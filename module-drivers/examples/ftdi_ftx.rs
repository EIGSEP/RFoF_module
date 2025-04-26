use embedded_hal_bus::i2c::RefCellDevice;
use ftdi_embedded_hal::libftd2xx::{self};
use ftdi_embedded_hal::{self as hal};
use rfof::{modules::ftx::Ftx, peripherals::atten::Attenuation};
use std::cell::RefCell;

fn main() {
    let device = libftd2xx::Ft4232ha::with_description("Quad RS232-HS B").unwrap();

    let hal = hal::FtHal::init_freq(device, 100_000).unwrap(); // 100 kHz
    let bus = RefCell::new(hal.i2c().unwrap());

    let mut ftx = Ftx::new(
        RefCellDevice::new(&bus),
        RefCellDevice::new(&bus),
        RefCellDevice::new(&bus),
        RefCellDevice::new(&bus),
    );

    ftx.init().unwrap();

    // Control example
    ftx.atten.set(Attenuation::_1_25).unwrap();
    ftx.adc.enable_lna(false).unwrap();
    ftx.digipot.set(25.0).unwrap();

    // Monitor example
    println!("Temperature: {:.2} C", ftx.temp.temp().unwrap());
    println!("Unique ID: {:#x}", ftx.temp.uid().unwrap());
    println!("Attenuation: {:#?}", ftx.atten.get().unwrap());
    println!("PD Current: {:.2} uA", ftx.adc.pd_current().unwrap());
    println!("LD Current: {:.2} mA", ftx.adc.ld_current().unwrap());
    println!("LD Setpoint Current: {:.2} mA", ftx.digipot.get().unwrap());
    println!("LNA Current: {:.2} mA", ftx.adc.lna_current().unwrap());
    println!("RF Power: {:.2} dBm", ftx.adc.rf_power().unwrap());
    println!("VDDA: {:.2} V", ftx.adc.analog_voltage().unwrap());
    println!("VDD: {:.2} V", ftx.adc.digital_voltage().unwrap());
    println!("VLNA: {:.2} V", ftx.adc.lna_voltage().unwrap());
}
