use crate::api::arbitrator::Arbitrator;
use crate::api::spi::{SpiBus, SpiError, SpiTransfer};
use crate::prelude::*;
use core::fmt::Debug;
use core::marker::PhantomData;
use embedded_hal::blocking::spi::Transfer;

// ------------------------------------------------------------------------
// ------------------------------------------------------------------------

pub struct Spi<SPI, W>
where
    SPI: Transfer<W> + 'static,
    SPI::Error: Into<SpiError>,
    W: 'static,
{
    arbitrator: Arbitrator<SpiController<SPI, W>>,
    controller: ActorContext<SpiController<SPI, W>>,
}

impl<SPI, W> Spi<SPI, W>
where
    SPI: Transfer<W>,
    SPI::Error: Into<SpiError>,
{
    pub fn new(spi: SPI) -> Self {
        Self {
            arbitrator: Arbitrator::new(),
            controller: ActorContext::new(SpiController::new(spi)),
        }
    }
}

impl<SPI, W> Package for Spi<SPI, W>
where
    SPI: Transfer<W>,
    SPI::Error: Into<SpiError>,
{
    type Primary = <Arbitrator<SpiController<SPI, W>> as Package>::Primary;
    type Configuration = ();

    fn mount(
        &'static self,
        config: Self::Configuration,
        supervisor: &mut Supervisor,
    ) -> Address<Self::Primary> {
        let controller_addr = self.controller.mount((), supervisor);
        self.arbitrator.mount(controller_addr, supervisor)
    }

    fn primary(&'static self) -> Address<Self::Primary> {
        self.arbitrator.primary()
    }
}

pub struct SpiController<SPI: Transfer<W>, W> {
    spi: SPI,
    _marker: PhantomData<W>,
}

impl<SPI, W> SpiController<SPI, W>
where
    SPI: Transfer<W>,
{
    fn new(spi: SPI) -> Self {
        Self {
            spi,
            _marker: PhantomData,
        }
    }
}

impl<SPI, W> Actor for SpiController<SPI, W>
where
    SPI: Transfer<W>,
{
    type Configuration = ();
}

impl<SPI, W> SpiBus for SpiController<SPI, W>
where
    SPI::Error: Into<SpiError>,
    SPI: Transfer<W>,
    W: Debug,
{
    type Word = W;

    fn transfer(
        mut self,
        transfer: SpiTransfer<Self::Word>,
    ) -> Response<Self, Result<(), SpiError>> {
        let result = self.spi.transfer(transfer.0).map_err(|e| e.into());
        Response::immediate(self, result.map(|_| ()))
    }
}
