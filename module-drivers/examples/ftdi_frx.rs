use embedded_hal_bus::i2c::RefCellDevice;
use ftdi_embedded_hal::libftd2xx::{self};
use ftdi_embedded_hal::{self as hal};
use rfof::{modules::frx::Frx, peripherals::atten::Attenuation};
use std::cell::RefCell;

fn main() {
    let device = libftd2xx::Ft4232ha::with_description("Quad RS232-HS A").unwrap();

    let hal = hal::FtHal::init_freq(device, 100_000).unwrap();
    let bus = RefCell::new(hal.i2c().unwrap());

    let mut frx = Frx::new(
        RefCellDevice::new(&bus),
        RefCellDevice::new(&bus),
        RefCellDevice::new(&bus),
    );

    frx.init().unwrap();

    // Control example
    frx.atten.set(Attenuation::_15_25).unwrap();

    // Monitor example
    println!("RF Power: {:.2} dBm", frx.adc.rf_power().unwrap());
    println!("PD Current: {:.2} mA", frx.adc.pd_current().unwrap());
    println!("Temperature: {:.2} C", frx.temp.temp().unwrap());
    println!("Unique ID: {:#x}", frx.temp.uid().unwrap());
}
