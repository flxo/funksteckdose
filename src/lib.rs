/// Copyright © 2019 Felix Obenhuber
///
/// Permission is hereby granted, free of charge, to any person obtaining a copy
/// of this software and associated documentation files (the "Software"), to deal
/// in the Software without restriction, including without limitation the rights
/// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
/// copies of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be included in all
/// copies or substantial portions of the Software.
//
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
/// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
/// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
/// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
/// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
/// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
/// SOFTWARE.

use error::Error;
use log::debug;
use std::marker::PhantomData;
use std::str;

/// Error
pub mod error {
    use failure::Fail;

    #[derive(Debug, Fail)]
    pub enum Error {
        #[fail(display = "invalid group identifier: {}", _0)]
        InvalidGroup(String),
        #[fail(display = "invalid device identifier: {}", _0)]
        InvalidDevice(String),
        #[fail(display = "invalid state: {}. Try on, off, 1, 0, true, false", _0)]
        InvalidState(String),
    }
}

/// A Device
#[derive(Clone, Debug, PartialEq)]
pub enum Device {
    A,
    B,
    C,
    D,
    E,
}

impl From<Device> for u8 {
    fn from(d: Device) -> u8 {
        match d {
            Device::A => 1,
            Device::B => 2,
            Device::C => 3,
            Device::D => 4,
            Device::E => 5,
        }
    }
}

impl str::FromStr for Device {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" | "a" | "A" | "10000" => Ok(Device::A),
            "1" | "b" | "B" | "01000" => Ok(Device::B),
            "2" | "c" | "C" | "00100" => Ok(Device::C),
            "3" | "d" | "D" | "00010" => Ok(Device::D),
            "4" | "e" | "E" | "00001" => Ok(Device::E),
            _ => Err(Error::InvalidDevice(s.into())),
        }
    }
}

/// State to switch a socket to
#[derive(Clone, Debug, PartialEq)]
pub enum State {
    On,
    Off,
}

impl str::FromStr for State {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "On" | "on" | "1" | "true" => Ok(State::On),
            "Off" | "off" | "0" | "false" => Ok(State::Off),
            _ => Err(Error::InvalidState(s.into())),
        }
    }
}

/// Value to set a GPIO to
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Low,
    High,
}

/// Encoding
pub trait Encoding {
    fn encode(group: &str, device: &Device, state: &State) -> Result<Vec<u8>, Error>;
}

/// Encoding A - check [rc-switch](https://github.com/sui77/rc-switch/) for details
pub struct EncodingA;

impl Encoding for EncodingA {
    fn encode(group: &str, device: &Device, state: &State) -> Result<Vec<u8>, Error> {
        if group.len() != 5 || group.chars().any(|c| c != '0' && c != '1') {
            return Err(Error::InvalidGroup(group.into()));
        }
        let chars = group.chars();

        let device = match device {
            Device::A => "10000",
            Device::B => "01000",
            Device::C => "00100",
            Device::D => "00010",
            Device::E => "00001",
        };

        let chars = chars.chain(device.chars());

        let chars = match *state {
            State::On => chars.chain("10".chars()),
            State::Off => chars.chain("01".chars()),
        };

        Ok(chars
            .map(|c| match c {
                '0' => b'F',
                _ => b'0',
            })
            .collect())
    }
}

/// Encoding B - check [rc-switch](https://github.com/sui77/rc-switch/) for details
pub struct EncodingB;

impl Encoding for EncodingB {
    fn encode(_group: &str, _device: &Device, _state: &State) -> Result<Vec<u8>, Error> {
        unimplemented!()
    }
}

/// Encoding C - check [rc-switch](https://github.com/sui77/rc-switch/) for details
pub struct EncodingC;

impl Encoding for EncodingC {
    fn encode(_group: &str, _device: &Device, _state: &State) -> Result<Vec<u8>, Error> {
        unimplemented!()
    }
}

/// Interface for GPIO control
pub trait Pin {
    fn set(&self, value: &Value) -> Result<(), Error>;
}

/// Handle to a Funksteckdose system
#[derive(Debug)]
pub struct Funksteckdose<T: Pin, E: Encoding, P: Protocol> {
    pin: T,
    repeat_transmit: usize,
    protocol: PhantomData<P>,
    encoding: PhantomData<E>,
}

impl<T: Pin, E: Encoding, P: Protocol> Funksteckdose<T, E, P> {
    /// Create a new instance with a given pin and default protocol
    /// ```
    /// type Funksteckdose = funksteckdose::Funksteckdose<WiringPiPin, EncodingA, Protocol1>;
    /// let pin = WiringPiPin::new(0);
    /// let d: Funksteckdose = Funksteckdose::new(pin);
    /// ```
    pub fn new(pin: T) -> Funksteckdose<T, E, P> {
        Self::with_repeat_transmit(pin, 10)
    }

    /// Create a new instance with a given pin and transmit count
    /// ```
    /// type Funksteckdose = funksteckdose::Funksteckdose<WiringPiPin, EncodingA, Protocol1>;
    /// let pin = WiringPiPin::new(0);
    /// let d: Funksteckdose = Funksteckdose::with_repeat_transmit(pin, 5);
    /// ```
    pub fn with_repeat_transmit(pin: T, repeat_transmit: usize) -> Funksteckdose<T, E, P> {
        Funksteckdose {
            pin,
            repeat_transmit,
            protocol: PhantomData,
            encoding: PhantomData,
        }
    }

    /// Send a control sequence to give group and device.
    /// The group is coded like the dip switches in the devices e.g "10010"
    /// ```
    /// type Funksteckdose = funksteckdose::Funksteckdose<WiringPiPin, EncodingA, Protocol1>;
    /// let pin = WiringPiPin::new(0);
    /// let d: Funksteckdose = Funksteckdose::with_repeat_transmit(pin, 5);
    /// d.send("10001", &Device::A, &State::On).expect("Failed to send");
    /// ```
    pub fn send(&self, group: &str, device: &Device, state: &State) -> Result<(), Error> {
        let code_word = E::encode(group, device, state)?;
        self.send_tri_state(&code_word)
    }

    fn send_tri_state(&self, code_word: &[u8]) -> Result<(), Error> {
        let code = code_word.iter().fold(0u64, |mut code, c| {
            code <<= 2u64;
            match c {
                b'0' => (),           // bit pattern 00
                b'F' => code |= 1u64, // bit pattern 01
                b'1' => code |= 3u64, // bit pattern 11
                _ => unreachable!(),
            }
            code
        });

        // Transmit the first 'length' bits of the integer 'code'. The
        // bits are sent from MSB to LSB, i.e., first the bit at position length-1,
        // then the bit at position length-2, and so on, till finally the bit at position 0.
        let (first, second) = if P::values().inverted_signal {
            (Value::Low, Value::High)
        } else {
            (Value::High, Value::Low)
        };
        let length = code_word.len() * 2;
        for _ in 0..self.repeat_transmit {
            debug!("Sending code: {:#X} length: {}", code, length);
            let one = P::values().one;
            let zero = P::values().zero;
            for i in (0..length).rev() {
                let s = if code & (1 << i) != 0 { &one } else { &zero };
                self.transmit(s, &first, &second)?;
            }
            self.transmit(&P::values().sync_factor, &first, &second)?;
        }

        // Disable transmit after sending (i.e., for inverted protocols)
        self.pin.set(&Value::Low)?;
        Ok(())
    }

    fn transmit(&self, pulses: &HighLow, first: &Value, second: &Value) -> Result<(), Error> {
        self.pin.set(first)?;
        Self::delay((P::values().pulse_length * pulses.high) as u32);
        self.pin.set(second)?;
        Self::delay((P::values().pulse_length * pulses.low) as u32);
        Ok(())
    }

    fn delay(micros: u32) {
        if micros > 0 {
            let now = std::time::Instant::now();
            let micros = u128::from(micros);
            while now.elapsed().as_micros() < micros {}
        }
    }
}

/// Number of pulses
#[derive(Clone, Debug)]
pub struct HighLow {
    pub high: u64,
    pub low: u64,
}

impl HighLow {
    fn new(high: u64, low: u64) -> HighLow {
        HighLow { high, low }
    }
}

/// Format for protocol definitions
#[derive(Clone, Debug)]
pub struct ProtocolValues {
    pulse_length: u64,
    sync_factor: HighLow,
    zero: HighLow,
    one: HighLow,
    inverted_signal: bool,
}

/// A protocol definition
pub trait Protocol {
    fn values() -> ProtocolValues;
}

/// Protocol 1
pub struct Protocol1;

impl Protocol for Protocol1 {
    fn values() -> ProtocolValues {
        ProtocolValues {
            pulse_length: 350,
            sync_factor: HighLow::new(1, 31),
            zero: HighLow::new(1, 3),
            one: HighLow::new(3, 1),
            inverted_signal: false,
        }
    }
}

/// Protocol 2
pub struct Protocol2;

impl Protocol for Protocol2 {
    fn values() -> ProtocolValues {
        ProtocolValues {
            pulse_length: 650,
            sync_factor: HighLow::new(1, 10),
            zero: HighLow::new(1, 2),
            one: HighLow::new(2, 1),
            inverted_signal: false,
        }
    }
}

/// Protocol 3
pub struct Protocol3;

impl Protocol for Protocol3 {
    fn values() -> ProtocolValues {
        ProtocolValues {
            pulse_length: 100,
            sync_factor: HighLow::new(30, 71),
            zero: HighLow::new(4, 11),
            one: HighLow::new(9, 6),
            inverted_signal: false,
        }
    }
}

/// Protocol 4
pub struct Protocol4;

impl Protocol for Protocol4 {
    fn values() -> ProtocolValues {
        ProtocolValues {
            pulse_length: 380,
            sync_factor: HighLow::new(1, 6),
            zero: HighLow::new(1, 3),
            one: HighLow::new(3, 1),
            inverted_signal: false,
        }
    }
}

/// Protocol 5
pub struct Protocol5;

impl Protocol for Protocol5 {
    fn values() -> ProtocolValues {
        ProtocolValues {
            pulse_length: 500,
            sync_factor: HighLow::new(6, 14),
            zero: HighLow::new(1, 2),
            one: HighLow::new(2, 1),
            inverted_signal: false,
        }
    }
}


/// Protocol HT6P20B
pub struct ProtocolHT6P20B;

impl Protocol for ProtocolHT6P20B {
    fn values() -> ProtocolValues {
        ProtocolValues {
            pulse_length: 450,
            sync_factor: HighLow::new(23, 1),
            zero: HighLow::new(1, 2),
            one: HighLow::new(2, 1),
            inverted_signal: true,
        }
    }
}

/// Protocol HS2303-PT, i. e. used in AUKEY Remote
pub struct ProtocolHS2303;

impl Protocol for ProtocolHS2303 {
    fn values() -> ProtocolValues {
        ProtocolValues {
            pulse_length: 150,
            sync_factor: HighLow::new(2, 62),
            zero: HighLow::new(1, 6),
            one: HighLow::new(6, 1),
            inverted_signal: false,
        }
    }
}

/// A implementation of Pin to be used with wiringpi on a Raspberry
///
///```
/// let pin = WiringPiPin::new(0);
/// let funksteckdose = Funksteckdose::new(pin, 1).unwrap();
/// funksteckdose.send("10011", "10000", State::On);
///```
#[cfg(feature = "wiringpi")]
pub mod wiringpi {
    use super::{Error, Pin, Value};

    pub struct WiringPiPin {
        pin: wiringpi::pin::OutputPin<wiringpi::pin::WiringPi>,
    }

    impl WiringPiPin {
        pub fn new(pin: u16) -> WiringPiPin {
            let pi = wiringpi::setup();
            WiringPiPin {
                pin: pi.output_pin(pin),
            }
        }
    }

    impl Pin for WiringPiPin {
        fn set(&self, value: &Value) -> Result<(), Error> {
            match value {
                Value::High => self.pin.digital_write(wiringpi::pin::Value::High),
                Value::Low => self.pin.digital_write(wiringpi::pin::Value::Low),
            }
            Ok(())
        }
    }
}
