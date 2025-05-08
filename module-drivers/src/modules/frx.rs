//! The top-level FRX module driver

use crate::peripherals::temp::TemperataureSensor;
use crate::peripherals::{adc::frx::Adc, atten::Attenuator};
use embedded_hal::i2c::I2c;

// The fiber receiver module
pub struct Frx<I2C> {
    /// Digital step attenuator
    pub atten: Attenuator<I2C>,
    /// ADC
    pub adc: Adc<I2C>,
    /// Temperature sensor / ID
    pub temp: TemperataureSensor<I2C>,
}

#[derive(Debug)]
/// FRX Error types
pub enum Error<E> {
    /// Lower-level attenuator error
    Atten(crate::peripherals::atten::Error<E>),
    /// Lower-level adc error
    Adc(E),
    /// Lower-level temperature sensor error
    Temp(crate::peripherals::temp::Error<E>),
}

// convert::from impls to use `?` in drivers to convert to top-level error

impl<E> core::convert::From<crate::peripherals::atten::Error<E>> for Error<E>
where
    E: embedded_hal::i2c::Error,
{
    fn from(e: crate::peripherals::atten::Error<E>) -> Self {
        Error::Atten(e)
    }
}

impl<E> core::convert::From<crate::peripherals::temp::Error<E>> for Error<E>
where
    E: embedded_hal::i2c::Error,
{
    fn from(e: crate::peripherals::temp::Error<E>) -> Self {
        Error::Temp(e)
    }
}

/// Result type for FRX commands
pub type FrxResult<T, E> = Result<T, Error<E>>;

impl<I2C, E> Frx<I2C>
where
    I2C: I2c<Error = E>,
    E: embedded_hal::i2c::Error,
{
    /// Construct a new FRX instance.
    ///
    /// This requires ownership of separate I2C instances, but in reality these
    /// will share a bus using something like refcell or mutex (embedded_hal_bus)
    pub fn new(atten_bus: I2C, adc_bus: I2C, temp_bus: I2C) -> Self {
        // Attenuator address select is tied to ground
        let temp = TemperataureSensor::new(temp_bus, 0x48);
        let atten = Attenuator::new(atten_bus, false);
        let adc = Adc::<I2C>::new(adc_bus);
        Self { atten, adc, temp }
    }

    /// Initialize all the child peripherals
    pub fn init(&mut self) -> FrxResult<(), E> {
        self.atten.init()?;
        self.adc.init().map_err(|e| Error::Adc(e))?;
        self.temp.init()?;
        Ok(())
    }
}
