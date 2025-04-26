//! FTX-Specific ADC implementation

use super::driver::{Adc as RawAdc, PinMode};
use super::{Adc as AdcTrait, RF_AVGS, VOLTAGE_AVGS};
use embedded_hal::i2c::I2c;

/// High-level ADC interface for the FTX
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

// Gains and scaling factors
const LDI_SHUNT: f32 = 1.0;
const PDI_SHUNT: f32 = 100.0;
const LNAI_SHUNT: f32 = 0.5;

const LDI_GAIN: f32 = 100.0;
const PDI_GAIN: f32 = 100.0;
const LNAI_GAIN: f32 = 100.0;

const VDDA_GAIN: f32 = 0.5;
const VLNA_GAIN: f32 = 0.25;
const VDD_GAIN: f32 = 0.5;

// Channels
const VDDA: u8 = 0;
const PDI: u8 = 1;
const RF: u8 = 2;
const LNAI: u8 = 3;
const LDI: u8 = 4;
const VLNA: u8 = 5;
const LNA_EN: u8 = 6;
const VDD: u8 = 7;

impl<I2C, E> Adc<I2C>
where
    I2C: I2c<Error = E>,
    E: embedded_hal::i2c::Error,
{
    /// Initialize and setup the ADC
    pub fn init(&mut self) -> Result<(), E> {
        self.0.reset()?;
        self.0.calibrate()?;
        self.configure(&[
            (VDDA, PinMode::Analog),
            (PDI, PinMode::Analog),
            (RF, PinMode::Analog),
            (LNAI, PinMode::Analog),
            (LDI, PinMode::Analog),
            (VLNA, PinMode::Analog),
            (LNA_EN, PinMode::DigitalOut),
            (VDD, PinMode::Analog),
        ])?;
        Ok(())
    }

    /// Get the analog supply (VDDA) voltage (in V)
    pub fn analog_voltage(&mut self) -> Result<f32, E> {
        self.read_voltage_avgs(VDDA, VDDA_GAIN, VOLTAGE_AVGS)
    }

    /// Get the DC monitor photodiode current (in uA)
    pub fn pd_current(&mut self) -> Result<f32, E> {
        Ok(self.read_current_avgs(PDI, PDI_SHUNT, PDI_GAIN, VOLTAGE_AVGS)? * 1e6)
    }

    /// Get the RF power (in dBm)
    pub fn rf_power(&mut self) -> Result<f32, E> {
        let raw = self.read_float_avgs(RF, RF_AVGS)?;
        Ok(17.74 * (raw * 5.0) - 55.0)
    }

    /// Get the LNA current (in mA)
    pub fn lna_current(&mut self) -> Result<f32, E> {
        Ok(self.read_current_avgs(LNAI, LNAI_SHUNT, LNAI_GAIN, VOLTAGE_AVGS)? * 1000.0)
    }

    /// Get the DC monitor photodiode current (in mA)
    pub fn ld_current(&mut self) -> Result<f32, E> {
        Ok(self.read_current_avgs(LDI, LDI_SHUNT, LDI_GAIN, VOLTAGE_AVGS)? * 1000.0)
    }

    /// Get the LNA voltage (in V)
    pub fn lna_voltage(&mut self) -> Result<f32, E> {
        self.read_voltage_avgs(VLNA, VLNA_GAIN, VOLTAGE_AVGS)
    }

    /// Get the digital supply (VDD) voltage (in V)
    pub fn digital_voltage(&mut self) -> Result<f32, E> {
        self.read_voltage_avgs(VDD, VDD_GAIN, VOLTAGE_AVGS)
    }

    /// Set the state of the LNA bias
    pub fn enable_lna(&mut self, enable: bool) -> Result<(), E> {
        self.0.digital_write(LNA_EN, enable)
    }
}
