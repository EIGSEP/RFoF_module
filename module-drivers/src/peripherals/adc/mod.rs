//! Generic ADC implementations

pub mod driver;
pub mod frx;
pub mod ftx;

const RF_AVGS: usize = 64;
const VOLTAGE_AVGS: usize = 64;

use embedded_hal::i2c::I2c;

pub trait Adc<I2C, E>
where
    I2C: I2c<Error = E>,
    E: embedded_hal::i2c::Error,
{
    /// ADC analog voltage reference
    const VREF: f32;

    fn inner_mut(&mut self) -> &mut driver::Adc<I2C>;

    /// Configure the ADC given a vector of channel/mode pairs
    fn configure(&mut self, pin_cfgs: &[(u8, driver::PinMode)]) -> Result<(), E> {
        for (chan, mode) in pin_cfgs {
            self.inner_mut().set_pin_mode(*mode, *chan)?;
        }
        Ok(())
    }

    /// Read an analog channel as a value from 0 to 1
    fn read_float_avgs(&mut self, chan: u8, avgs: usize) -> Result<f32, E> {
        Ok(self.inner_mut().read_chan_with_average(chan, avgs)? as f32 / 4095.0)
    }

    /// Read a current-sense channel given a `shunt` resistor and current-amplifier `gain`
    fn read_current_avgs(
        &mut self,
        chan: u8,
        shunt: f32,
        gain: f32,
        avgs: usize,
    ) -> Result<f32, E> {
        let raw: f32 = self.read_float_avgs(chan, avgs)?;
        Ok((raw * Self::VREF) / (gain * shunt))
    }

    /// Read a voltage-channel given a 'gain' implemented via an amplifier or resistor divider
    fn read_voltage_avgs(&mut self, chan: u8, gain: f32, avgs: usize) -> Result<f32, E> {
        let raw: f32 = self.read_float_avgs(chan, avgs)?;
        Ok((raw * Self::VREF) / gain)
    }
}
