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

#[cfg(not(target_os = "arm-unknown-linux-gnueabihf"))]
fn main() {
    println!("nop - sorry, this examples requires wiringpi...");
}

#[cfg(all(target_os = "arm-unknown-linux-gnueabihf", cfg("wiringpi")))]
fn main() {
    use funksteckdose::{wiringpi::WiringPiPin, Device, EncodingA, Protocol1, State};
    use std::str::FromStr;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(name = "funksteckdose", about = "Control 433Mhz wireless sockets")]
    struct Opt {
        /// Select group according to dip switches e.g "10011"
        #[structopt(short = "g", long = "group")]
        group: String,
        /// Select device according to dip switches e.g "10000" or "A" or "0"
        #[structopt(short = "d", long = "device", parse(try_from_str = "Device::from_str"))]
        device: Device,
        /// Send command: on, off, true, false, 1, 0
        #[structopt(short = "s", long = "send", parse(try_from_str = "State::from_str"))]
        send: State,
        /// Select WiringPI pin. Default: 0
        #[structopt(short = "p", long = "pin")]
        pin: Option<u16>,
    }

    let opt = Opt::from_args();

    // Use wiringpi pin 0. See http://wiringpi.com/pins/
    type Funksteckdose = funksteckdose::Funksteckdose<WiringPiPin, EncodingA, Protocol1>;
    let pin = WiringPiPin::new(opt.pin.unwrap_or(0));
    let d: Funksteckdose = Funksteckdose::new(pin);
    d.send(&opt.group, &opt.device, &opt.send).expect("Failed to send");
}
