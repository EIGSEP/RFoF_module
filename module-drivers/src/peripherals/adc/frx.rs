//! FRX-Specific ADC implementation

use super::driver::{Adc as RawAdc, PinMode};
use super::{Adc as AdcTrait, RF_AVGS, VOLTAGE_AVGS};
use embedded_hal::i2c::I2c;

/// High-level ADC interface for the FRX
pub struct Adc<I2C>(RawAdc<I2C>);

impl<I2C, E> Adc<I2C>
where
    I2C: I2c<Error = E>,
    E: embedded_hal::i2c::Error,
{
    pub fn new(bus: I2C) -> Adc<I2C> {
        // FTX has hard-coded address of 0x10
        Adc(RawAdc::new(bus, 0x10))
    }
}

impl<I2C, E> AdcTrait<I2C, E> for Adc<I2C>
where
    I2C: I2c<Error = E>,
    E: embedded_hal::i2c::Error,
{
    const VREF: f32 = 5.0;

    fn inner_mut(&mut self) -> &mut RawAdc<I2C> {
        &mut self.0
    }
}

/// The meat of the implementation

/// Photodiode DC current monitor resistor value
const PDI_SHUNT: f32 = 5.1;

/// Current-sense amplifier gain
const GAIN: f32 = 100.0;

/// RF power monitoring channel
const RF: u8 = 0;

/// Photodiode current monitoring channel
const PDI: u8 = 1;

impl<I2C, E> Adc<I2C>
where
    I2C: I2c<Error = E>,
    E: embedded_hal::i2c::Error,
{
    /// Initialize and setup the ADC
    pub fn init(&mut self) -> Result<(), E> {
        self.0.reset()?;
        self.0.calibrate()?;
        self.configure(&[(RF, PinMode::Analog), (PDI, PinMode::Analog)])?;
        Ok(())
    }

    /// Get the DC photodiode current (in mA)
    pub fn pd_current(&mut self) -> Result<f32, E> {
        Ok(self.read_current_avgs(PDI, PDI_SHUNT, GAIN, VOLTAGE_AVGS)? * 1000.0)
    }

    /// Get the RF power (in dBm)
    pub fn rf_power(&mut self) -> Result<f32, E> {
        let raw = self.read_float_avgs(RF, RF_AVGS)?;
        Ok(17.74 * (raw * 5.0) - 55.0)
    }
}
