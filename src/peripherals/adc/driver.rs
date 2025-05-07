//! A dumbed-down driver of the TLA2528

use embedded_hal::i2c::I2c;

#[repr(u8)]
#[derive(Copy, Clone)]
enum Op {
    SingleRegRead = 0b0001_0000,
    SingleRegWrite = 0b0000_1000,
    SetBit = 0b0001_1000,
    ClearBit = 0b0010_0000,
}

#[repr(u8)]
#[derive(Copy, Clone)]
enum Reg {
    SystemStatus = 0x0,
    GeneralCfg = 0x1,
    PinCfg = 0x5,
    GpioCfg = 0x7,
    GpoDriveCfg = 0x9,
    GpoValue = 0xB,
    ChannelSel = 0x11,
}

#[derive(Copy, Clone)]
pub enum PinMode {
    Analog,
    DigitalOut,
}

pub struct Adc<I2C> {
    bus: I2C,
    addr: u8,
}

impl<I2C, E> Adc<I2C>
where
    I2C: I2c<Error = E>,
    E: embedded_hal::i2c::Error,
{
    pub fn new(bus: I2C, addr: u8) -> Self {
        Self { bus, addr }
    }

    #[allow(unused)]
    fn read_raw_reg(&mut self, reg: u8) -> Result<u8, E> {
        let mut byte = [0u8];
        self.bus
            .write_read(self.addr, &[Op::SingleRegRead as u8, reg], &mut byte)?;
        Ok(byte[0])
    }

    #[allow(unused)]
    fn read_reg(&mut self, reg: Reg) -> Result<u8, E> {
        self.read_raw_reg(reg as u8)
    }

    fn set_bit(&mut self, reg: Reg, bit: u8) -> Result<(), E> {
        self.bus
            .write(self.addr, &[Op::SetBit as u8, reg as u8, 1 << bit])?;
        Ok(())
    }

    fn clear_bit(&mut self, reg: Reg, bit: u8) -> Result<(), E> {
        self.bus
            .write(self.addr, &[Op::ClearBit as u8, reg as u8, 1 << bit])?;
        Ok(())
    }

    fn write_reg(&mut self, reg: Reg, byte: u8) -> Result<(), E> {
        self.bus
            .write(self.addr, &[Op::SingleRegWrite as u8, reg as u8, byte])?;
        Ok(())
    }

    /// Will attempt to read up to `n` averages (anything greater than 256 will default to 256)
    fn read_and_average(&mut self, n: usize) -> Result<u16, E> {
        let mut bytes = [0u8; 512];
        let trunc_n = if n <= 256 { n } else { 256 };
        let slice = &mut bytes[0..(trunc_n * 2)];
        self.bus.read(self.addr, slice)?;
        let mut res = [0u16; 256];
        slice.chunks(2).enumerate().for_each(|(i, chunk)| {
            res[i] = u16::from_be_bytes(chunk.try_into().unwrap()) >> 4;
        });
        Ok(integer_avg(&res[0..trunc_n]))
    }

    // ---- Higher-level stuff
    pub fn reset(&mut self) -> Result<(), E> {
        self.set_bit(Reg::GeneralCfg, 0)?;
        Ok(())
    }

    pub fn calibrate(&mut self) -> Result<(), E> {
        self.set_bit(Reg::SystemStatus, 0)?;
        Ok(())
    }

    pub fn set_pin_mode(&mut self, mode: PinMode, chan: u8) -> Result<(), E> {
        match mode {
            PinMode::Analog => {
                self.clear_bit(Reg::PinCfg, chan)?; // Analog
            }
            PinMode::DigitalOut => {
                self.set_bit(Reg::PinCfg, chan)?; // GPIO
                self.set_bit(Reg::GpioCfg, chan)?; // DigitalOut
                self.set_bit(Reg::GpoDriveCfg, chan)?; // PushPull
            }
        }
        Ok(())
    }

    /// Undefined behavior happens on pins that aren't analog (perhaps we should check)
    pub fn read_chan_with_average(&mut self, chan: u8, avgs: usize) -> Result<u16, E> {
        self.write_reg(Reg::ChannelSel, chan)?;
        self.read_and_average(avgs)
    }

    pub fn digital_write(&mut self, chan: u8, set: bool) -> Result<(), E> {
        if set {
            self.set_bit(Reg::GpoValue, chan)
        } else {
            self.clear_bit(Reg::GpoValue, chan)
        }
    }
}

/// Neat algo for integer averaging without overflow
fn integer_avg(nums: &[u16]) -> u16 {
    let n = nums.len() as u16;
    let mut avg = 0;
    let mut err = 0;
    for num in nums {
        err += num % n;
        avg += (num / n) + (err / n);
        err %= n;
    }
    avg
}
