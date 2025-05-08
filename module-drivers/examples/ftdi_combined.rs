use embedded_hal_bus::i2c::RefCellDevice;
use ftdi_embedded_hal::libftd2xx::{self};
use ftdi_embedded_hal::{self as hal};
use rfof::{
    modules::{frx::Frx, ftx::Ftx},
    peripherals::atten::Attenuation,
};
use std::cell::RefCell;

fn main() {
    for d in libftd2xx::list_devices().expect("Could not poll devices") {
        dbg!(d);
    }
    let frx_device = libftd2xx::Ft4232ha::with_description("Quad RS232-HS A").unwrap();
    let ftx_device = libftd2xx::Ft4232ha::with_description("Quad RS232-HS B").unwrap();

    let frx_hal = hal::FtHal::init_freq(frx_device, 100_000).unwrap();
    let ftx_hal = hal::FtHal::init_freq(ftx_device, 100_000).unwrap();

    let frx_bus = RefCell::new(frx_hal.i2c().unwrap());
    let ftx_bus = RefCell::new(ftx_hal.i2c().unwrap());

    let mut frx = Frx::new(
        RefCellDevice::new(&frx_bus),
        RefCellDevice::new(&frx_bus),
        RefCellDevice::new(&frx_bus),
    );

    let mut ftx: Ftx<RefCellDevice<'_, ftdi_embedded_hal::I2c<libftd2xx::Ft4232ha>>> = Ftx::new(
        RefCellDevice::new(&ftx_bus),
        RefCellDevice::new(&ftx_bus),
        RefCellDevice::new(&ftx_bus),
        RefCellDevice::new(&ftx_bus),
    );

    ftx.init().unwrap();
    frx.init().unwrap();

    // Control example
    frx.atten.set(Attenuation::_15_25).unwrap();
    ftx.atten.set(Attenuation::_15_25).unwrap();
    ftx.adc.enable_lna(true).unwrap();
    ftx.digipot.set(25.0).unwrap();

    // Monitor example
    println!("---FTX---");
    println!("Temperature: {:.2} C", ftx.temp.temp().unwrap());
    println!("Unique ID: {:#x}", ftx.temp.uid().unwrap());
    println!("PD Current: {:.2} uA", ftx.adc.pd_current().unwrap());
    println!("LD Current: {:.2} mA", ftx.adc.ld_current().unwrap());
    println!("LD Setpoint Current: {:.2} mA", ftx.digipot.get().unwrap());
    println!("LNA Current: {:.2} mA", ftx.adc.lna_current().unwrap());
    println!("RF Power: {:.2} dBm", ftx.adc.rf_power().unwrap());
    println!("VDDA: {:.2} V", ftx.adc.analog_voltage().unwrap());
    println!("VDD: {:.2} V", ftx.adc.digital_voltage().unwrap());
    println!("VLNA: {:.2} V", ftx.adc.lna_voltage().unwrap());

    println!("\n---FRX---");
    println!("Temperature: {:.2} C", frx.temp.temp().unwrap());
    println!("Unique ID: {:#x}", frx.temp.uid().unwrap());
    println!("RF Power: {:.2} dBm", frx.adc.rf_power().unwrap());
    println!("PD Current: {:.2} mA", frx.adc.pd_current().unwrap());
}
