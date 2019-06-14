[![Build Status](https://travis-ci.org/flxo/funksteckdose.svg)](https://travis-ci.org/flxo/funksteckdose)
[![crates.io](https://img.shields.io/crates/f/funksteckdose.svg)](https://crates.io/crates/funksteckdose)

# Funksteckdose

This program can be used to control 433MHz wireless power outlets. It implements a subset of
the functionality of [rc-switch](https://github.com/sui77/rc-switch/). Check the [wiki](https://github.com/sui77/rc-switch/wiki/List_KnownDevices) for a list of known and supported devices.

```rust
use funksteckdose::{wiringpi::WiringPiPin, Device, EncodingA, Protocol1, State};

fn main() {
    type Funksteckdose = funksteckdose::Funksteckdose<WiringPiPin, EncodingA, Protocol1>;
    let pin = WiringPiPin::new(0);
    let d: Funksteckdose = Funksteckdose::new(pin);
    d.send("10001", &Device::A, &State::On).expect("Failed to send");
}
```
