//! Stripped-down implementation of the TCA6408 bus expander
//! driver to only operate in output mode to control our digital
//! attenuator.

use embedded_hal::i2c::I2c;

const ADDR_PREAMBLE: u8 = 0b0100000;

#[derive(Debug)]
/// Low-level implementation of the TCA6408A bus expander
struct Tca6408A<I2C> {
    bus: I2C,
    addr: u8,
}

#[derive(Debug)]
#[repr(u8)]
enum Register {
    OutputPort = 0x01,
    Configuration = 0x03,
}

impl<I2C, E> Tca6408A<I2C>
where
    I2C: I2c<Error = E>,
    E: embedded_hal::i2c::Error,
{
    fn new(bus: I2C, addr_bit: bool) -> Self {
        Self {
            bus,
            addr: ADDR_PREAMBLE | addr_bit as u8,
        }
    }

    /// Set all pins to outputs
    fn configure_outputs(&mut self) -> Result<(), E> {
        // Write to the configuration register `0` to set to output
        self.bus
            .write(self.addr, &[Register::Configuration as u8, 0])?;
        Ok(())
    }

    /// Write a word to the bus expander
    fn write_word(&mut self, word: u8) -> Result<(), E> {
        self.bus
            .write(self.addr, &[Register::OutputPort as u8, word])?;
        Ok(())
    }

    /// Read a word from the bus expander
    fn read_word(&mut self) -> Result<u8, E> {
        let mut byte = [0u8; 1];
        self.bus
            .write_read(self.addr, &[Register::OutputPort as u8], &mut byte)?;
        Ok(byte[0])
    }
}

/// High-level attenuator struct
pub struct Attenuator<I2C>(Tca6408A<I2C>);

macro_rules! attenuation_variants {
    ($($value:ident),*) => {
        #[repr(u8)]
        #[derive(Debug)]
        /// Valid attenuation values for the F1958 digital step attentuator
        pub enum Attenuation {
            $(
                #[allow(non_camel_case_types)]
                $value,
            )*
        }
    };
}

// Valid attenuation states
macro_rules! generate_attenuation_enum {
    () => {
        attenuation_variants!(
            _0, _0_25, _0_5, _0_75, _1_0, _1_25, _1_5, _1_75, _2_0, _2_25, _2_5, _2_75, _3_0,
            _3_25, _3_5, _3_75, _4_0, _4_25, _4_5, _4_75, _5_0, _5_25, _5_5, _5_75, _6_0, _6_25,
            _6_5, _6_75, _7_0, _7_25, _7_5, _7_75, _8_0, _8_25, _8_5, _8_75, _9_0, _9_25, _9_5,
            _9_75, _10_0, _10_25, _10_5, _10_75, _11_0, _11_25, _11_5, _11_75, _12_0, _12_25,
            _12_5, _12_75, _13_0, _13_25, _13_5, _13_75, _14_0, _14_25, _14_5, _14_75, _15_0,
            _15_25, _15_5, _15_75, _16_0, _16_25, _16_5, _16_75, _17_0, _17_25, _17_5, _17_75,
            _18_0, _18_25, _18_5, _18_75, _19_0, _19_25, _19_5, _19_75, _20_0, _20_25, _20_5,
            _20_75, _21_0, _21_25, _21_5, _21_75, _22_0, _22_25, _22_5, _22_75, _23_0, _23_25,
            _23_5, _23_75, _24_0, _24_25, _24_5, _24_75, _25_0, _25_25, _25_5, _25_75, _26_0,
            _26_25, _26_5, _26_75, _27_0, _27_25, _27_5, _27_75, _28_0, _28_25, _28_5, _28_75,
            _29_0, _29_25, _29_5, _29_75, _30_0, _30_25, _30_5, _30_75, _31_0, _31_25, _31_5,
            _31_75
        );
    };
}

generate_attenuation_enum!();

#[derive(Debug)]
pub enum Error<E> {
    /// Lower level bus error
    I2c(E),
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

impl<I2C, E> Attenuator<I2C>
where
    I2C: I2c<Error = E>,
    E: embedded_hal::i2c::Error,
{
    pub fn new(bus: I2C, addr_bit: bool) -> Self {
        Self(Tca6408A::new(bus, addr_bit))
    }

    /// Setup the outputs
    pub fn init(&mut self) -> Result<(), Error<E>> {
        // Configure as outputs and start in minimum atten
        self.0.configure_outputs()?;
        self.set(Attenuation::_0)?;
        Ok(())
    }

    /// Sets the raw attenuation word
    pub fn set_raw(&mut self, atten: u8) -> Result<(), Error<E>> {
        // Reminder: The LE pin (bit 8) needs to be set high
        let word = atten | 128;
        self.0.write_word(word)?;
        Ok(())
    }

    /// Sets the attenuation
    pub fn set(&mut self, atten: Attenuation) -> Result<(), Error<E>> {
        self.set_raw(atten as u8)
    }

    pub fn get(&mut self) -> Result<Attenuation, Error<E>> {
        let word = self.0.read_word()?;
        // Safety: Every value from 0-127 is a valid enum vaue
        Ok(unsafe { core::mem::transmute::<u8, Attenuation>(word & 127) })
    }
}
