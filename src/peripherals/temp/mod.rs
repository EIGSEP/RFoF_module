//! Minimal driver and high-level wrapper for the TMP117 temperature
//! sensor and ID
//!
//! Use all the defaults for now, if we want to muck with averaging and conversions,
//! we'll want to manage internal state and be a bit more careful

mod regs;

use embedded_hal::i2c::{I2c, Operation};
use packed_struct::PackedStruct;
use regs::{
    Addr, AveragingMode, Configuration, ConversionMode, Temperature, EEPROM1, EEPROM2, EEPROM3,
};

const SCALE_C: f32 = 7.8125e-3;

struct Tmp117<I2C> {
    bus: I2C,
    addr: u8,
}

impl<I2C, E> Tmp117<I2C>
where
    I2C: I2c<Error = E>,
    E: embedded_hal::i2c::Error,
{
    fn new(bus: I2C, addr: u8) -> Self {
        Self { bus, addr }
    }

    fn read_reg<R, const N: usize>(&mut self) -> Result<R, E>
    where
        R: Addr + PackedStruct<ByteArray = [u8; N]>,
    {
        let mut raw = [0u8; N];
        self.bus.write_read(self.addr, &[R::ADDR], &mut raw)?;
        let unpacked = R::unpack(&raw).unwrap();
        Ok(unpacked)
    }

    fn write_reg<R, const N: usize>(&mut self, reg: R) -> Result<(), E>
    where
        R: Addr + PackedStruct<ByteArray = [u8; N]>,
    {
        let bytes = reg.pack().unwrap();
        let mut operations = [Operation::Write(&[R::ADDR]), Operation::Write(&bytes)];
        self.bus.transaction(self.addr, &mut operations)?;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), E> {
        let con = Configuration {
            soft_reset: true,
            ..Default::default()
        };
        self.write_reg(con)?;
        Ok(())
    }

    fn set_cc(&mut self, cc: u8) -> Result<(), E> {
        let mut conf: Configuration = self.read_reg()?;
        conf.conv = cc;
        self.write_reg(conf)?;
        Ok(())
    }

    fn set_avg(&mut self, avg: AveragingMode) -> Result<(), E> {
        let mut conf: Configuration = self.read_reg()?;
        conf.avg = avg;
        self.write_reg(conf)?;
        Ok(())
    }

    fn set_mod(&mut self, mode: ConversionMode) -> Result<(), E> {
        let mut conf: Configuration = self.read_reg()?;
        conf.mode = mode;
        self.write_reg(conf)?;
        Ok(())
    }
}

/// High-level temperature sensor struct
pub struct TemperataureSensor<I2C>(Tmp117<I2C>);

#[derive(Debug)]
pub enum Error<E> {
    /// Lower level bus error
    I2c(E),
    /// Timeout while waiting for a conversion
    Timeout,
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

pub type TempResult<T, E> = Result<T, Error<E>>;

impl<I2C, E> TemperataureSensor<I2C>
where
    I2C: I2c<Error = E>,
    E: embedded_hal::i2c::Error,
{
    pub fn new(bus: I2C, addr: u8) -> Self {
        Self(Tmp117::new(bus, addr))
    }

    /// Initialize the temperature sensor
    pub fn init(&mut self) -> TempResult<(), E> {
        self.0.reset()?;
        // Polling is broken due to a silicon bug, so we'll just have the sensor free-run reading temperatures
        self.0.set_mod(ConversionMode::Continuous)?;
        // Might as well go full averaging (64) and minimum cycle time (so 1s between new samples)
        self.0.set_cc(0)?;
        self.0.set_avg(AveragingMode::_64)?;
        Ok(())
    }

    /// Get the unique ID
    pub fn uid(&mut self) -> TempResult<u64, E> {
        let uid1: EEPROM1 = self.0.read_reg()?;
        let uid2: EEPROM2 = self.0.read_reg()?;
        let uid3: EEPROM3 = self.0.read_reg()?;
        // I don't think we really care about the endianness here, as long as we're consistent
        Ok(((uid1.data as u64) << 32) | ((uid2.data as u64) << 16) | (uid3.data as u64))
    }

    /// Get the last conversion temperature in C
    pub fn temp(&mut self) -> TempResult<f32, E> {
        // So fun thing: TI fucked this up and ocassionally the data ready pin doesn't assert, so that's cool I guess. This is actually a silicon bug. Great.
        // See: https://e2e.ti.com/support/sensors-group/sensors/f/sensors-forum/1019457/tmp117-data_ready-flag-cleared-incorrectly-if-data-becomes-ready-during-read-of-configuration-register
        // And also: https://e2e.ti.com/support/sensors-group/sensors/f/sensors-forum/822036/tmp116-polling-the-data-ready-flag-seems-to-clear-it-inadvertently
        // And also: https://e2e.ti.com/support/sensors-group/sensors/f/sensors-forum/909104/tmp117-polling-the-data-ready-flag-seems-to-clear-it-inadvertently-when-using-1-shot-mode

        // So instead, we have the chip free run, populating the result register with updates every second, and we will just immediately read the most recent result
        let raw: Temperature = self.0.read_reg()?;
        Ok(SCALE_C * raw.temp as f32)
    }
}
