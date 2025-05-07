//! Stripped-down implementation of the digipot laser current control

use embedded_hal::i2c::I2c;

const ADDR_BASE: u8 = 0b0101100;

/// AD5245-compatible 256-position digital potentiometer
#[derive(Debug)]
struct Cat5171<I2C> {
    addr: u8,
    bus: I2C,
}

impl<I2C, E> Cat5171<I2C>
where
    I2C: I2c<Error = E>,
    E: embedded_hal::i2c::Error,
{
    fn new(bus: I2C, ad0: bool) -> Self {
        Self {
            bus,
            addr: ADDR_BASE | ad0 as u8,
        }
    }

    fn set_state(&mut self, word: u8) -> Result<(), E> {
        // We don't care about reset to midscale or shutdown, so hard-code
        // the "instruction byte" to 0
        self.bus.write(self.addr, &[0, word])
    }

    fn get_state(&mut self) -> Result<u8, E> {
        let mut byte = [0u8; 1];
        self.bus.read(self.addr, &mut byte)?;
        Ok(byte[0])
    }
}

/// High-level laser current control struct
pub struct Digipot<I2C>(Cat5171<I2C>);

#[derive(Debug)]
pub enum Error<E> {
    /// Lower level bus error
    I2c(E),
    /// Requested current was out of range
    OutOfRange,
}

// Convert I2C errors into our higher-level error
impl<E> core::convert::From<E> for Error<E>
where
    E: embedded_hal::i2c::Error,
{
    fn from(value: E) -> Self {
        Error::I2c(value)
    }
}

impl<I2C, E> Digipot<I2C>
where
    I2C: I2c<Error = E>,
    E: embedded_hal::i2c::Error,
{
    pub fn new(bus: I2C, ad0: bool) -> Self {
        Self(Cat5171::new(bus, ad0))
    }

    pub fn set_raw(&mut self, word: u8) -> Result<(), Error<E>> {
        Ok(self.0.set_state(word)?)
    }

    /// Set the laser current source in mA
    /// This function will approximate the closest to the appropriate 256-bit word
    pub fn set(&mut self, current: f32) -> Result<(), Error<E>> {
        if !(0.0..=50.0).contains(&current) {
            return Err(Error::OutOfRange);
        }
        let raw = (current * 255.0 / 50.0) as u8;
        self.set_raw(raw)
    }

    /// Gets the state of the adjustable current soruce in mA
    pub fn get(&mut self) -> Result<f32, Error<E>> {
        let raw = self.0.get_state()?;
        Ok(raw as f32 * 50.0 / 255.0)
    }
}
