# Drogue device drivers

Drogue-device contains device drivers for different boards and sensors. Drivers follow a common set of patterns that makes it easier to write new drivers. Device drivers can be written in different ways, but the common approach is to implement the following:

* An API that defines the API that a class of drivers implement. This may already exist (Examples are SPI and UART). This is located in `src/api/`.
* A driver that implements an API, either using a HAL or hardware directly. This is located in `src/driver`.
* (Optional) A HAL (Hardware Access Layer) for internal traits and types that a driver can use to support different hardware. This is located in `src/hal`.
* (Optional) Hardware-specific code that may implement a HAL. This is located in `src/port/<device class>`.

A driver is exposed as a `Package` that can be configure during application initialization. The `Package` exposes an `Actor` that can be used to interact with the driver.

# Writing the API

```rust
pub struct ReadValue;

pub trait MyTrait: Actor {
    fn read_value(self) -> Response<Self, u32>;
}

impl<A: MyTrait> RequestHandler<ReadValue> for A {
    type Response = u32;
    fn on_request(self, message: ReadValue) -> Response<Self, Self::Response> {
        self.read_value()
    }
}

impl<A: MyTrait> Address<A> {
    async fn read_value<A: RequestHandler<ReadValue>>(&self) -> u32 {
        self.request(ReadValue).await
    }
}
```

## Writing the driver

A driver is an implementation of the API that embedded applications use.

The simplest drivers are those that does not need to pass non-static references in its messages. You can also handle interrupts with an actor driver. This can be achieved by implementing the `Interrupt` trait as well as the actor trait. The common pattern is to use two actors, one for interrupts and one for handling commands. These actors may communicate either through a separate API or using the actor API.

Implementing the Package trait allows a driver to perform initial configuration across multiple sub-components, such as a separate IRQ actor that handles interrupts, and an API actor that handles requests. 

Here is an example driver that uses some struct to access the hardware, and it has some shared state between the actors, and it receives an initial configuration through implementing the `Package` trait.

```rust
pub struct MyDriver<T> {
    api: ActorContext<MyComponent>>,
    interrupt: InterruptContext<MyDevice<T>>,
    shared: AtomicU32,
}

pub struct MyComponent {
    shared: Option<&'static AtomicU32>
}

pub struct MyDevice {
    hardware: MyHardware,
    shared: Option<&'static AtomicU32>
}

impl MyDriver {
    fn new<IRQ: Nr>(hardware: MyHardware, irq: IRQ) -> Self {
       Self {
           api: ActorContext::new(MyComponent::new()),
           interrupt: InterruptContext::new(new MyDevice(hardware), irq),
           shared: AtomicU32::new(),
       }
    }
}
```

To become a `package`, the driver must implement the `Package` trait. The actors specify the type of configuration they expect, which is passed down from the driver. The package specifies its primary actor which is exposed to the user application.

```rust
impl Package for MyDriver {
    type Configuration = ();
    type Primary = MyComponent;
    fn on_mount(&'static self, config: Self::Configuration, supervisor: &mut Supervisor) {
        self.api.mount(&self.shared, supervisor);
        self.support.mount(&self.shared, supervisor)
    }
}

impl Actor for MyComponent {
    type Configuration = &'static AtomicU32;
    fn configure(&mut self, config: Self::Configuration) {
        self.shared.replace(config);
    }
}

impl Actor for MyDevice {
    type Configuration = &'static AtomicU32;
    fn configure(&mut self, config: Self::Configuration) {
        self.shared.replace(config);
    }
}
```

The `MyDevice` actor handles interrupts, reads the value of the device and stores it in the shared state:

```rust
impl Interrupt for MyDevice {
    fn on_interrupt(&mut self) {
        if let Some(value) = &self.shared {
            value.store(self.hardware.read_value(), Ordering::SeqCst);
        }
    }
}
```

The `MyTrait` implementation ensures that it conforms to the expected signature:

```rust
impl MyTrait for MyComponent {
    fn read_value(self) -> Response<Self, u32> {
        let value = self.load(Ordering::SeqCst);
        Response::immediate(self, value)
    }
}
```

Any actor that implements the `MyTrait` is considered a valid driver from the API POV.
