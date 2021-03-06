# microbit-rak811 drogue-device example

This example application runs out of the box on the BBC micro:bit v2.0. It relies on a RAK811 LoRa shield to which it communicates over UART.

## Prerequisites

### Hardware

* BBC micro:bit v2.0

### Software

To build and flash the example, you need to have [Rust](https://rustup.rs/), [cargo-embed](https://crates.io/crates/cargo-embed) installed. In pratice you can use whatever tool you want to flash the device, but this guide will assume cargo-embed is used.

## Building

Make sure you have the correct target architecture supported in rust:

```
rustup target add thumbv7em-none-eabihf
```

To build the firmware:

```
cargo build --release
```

## Flashing

Flashing the firmware uses the configuration from the [Embed.toml](Embed.toml) file, which auto-detects the probe connected to your device. If you're experiencing problems, try setting the `usb_vid` and `usb_pid` values to that of your probe (you can find that from lsusb once your board is powered).

The following command will build and flash the firmware and open the debugger console so you can see the console debug output.

```
cargo embed --release
```
