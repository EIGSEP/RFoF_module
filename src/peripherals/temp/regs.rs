//! Incomplete register map for the TMP117 for our use

use packed_struct::prelude::*;

pub(super) trait Addr {
    const ADDR: u8;
}

#[derive(PackedStruct, Debug, Default, PartialEq, Eq, Clone, Copy)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "2")]
pub(super) struct Temperature {
    #[packed_field(bits = "15..=0", endian = "msb")]
    pub(super) temp: i16,
}

impl Addr for Temperature {
    const ADDR: u8 = 0x00;
}

#[derive(PrimitiveEnum_u8, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConversionMode {
    #[default]
    Continuous = 0b00,
    Shutdown = 0b01,
    OneShot = 0b11,
}

#[derive(PrimitiveEnum_u8, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum AveragingMode {
    None = 0b00,
    #[default]
    _8 = 0b01,
    _32 = 0b10,
    _64 = 0b11,
}

#[derive(PackedStruct, Debug, Default, PartialEq, Eq, Clone, Copy)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "2")]
pub struct Configuration {
    #[packed_field(bits = "15")]
    pub(super) high_alert: bool,
    #[packed_field(bits = "14")]
    pub(super) low_alert: bool,
    #[packed_field(bits = "13")]
    pub(super) data_ready: bool,
    #[packed_field(bits = "12")]
    pub(super) eeprom_busy: bool,
    #[packed_field(bits = "11..=10", ty = "enum")]
    pub(super) mode: ConversionMode,
    #[packed_field(bits = "9..=7")]
    pub(super) conv: u8,
    #[packed_field(bits = "6..=5", ty = "enum")]
    pub(super) avg: AveragingMode,
    #[packed_field(bits = "4")]
    pub(super) t_na: bool,
    #[packed_field(bits = "3")]
    pub(super) pol: bool,
    #[packed_field(bits = "2")]
    pub(super) dr_alert: bool,
    #[packed_field(bits = "1")]
    pub(super) soft_reset: bool,
    #[packed_field(bits = "0")]
    pub(super) _res: ReservedZero<packed_bits::Bits<1>>,
}

impl Addr for Configuration {
    const ADDR: u8 = 0x01;
}

#[derive(PackedStruct, Debug, Default, PartialEq, Eq, Clone, Copy)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "2")]
pub(super) struct EEPROM1 {
    #[packed_field(bits = "15..=0", endian = "msb")]
    pub(super) data: u16,
}

impl Addr for EEPROM1 {
    const ADDR: u8 = 0x05;
}

#[derive(PackedStruct, Debug, Default, PartialEq, Eq, Clone, Copy)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "2")]
pub(super) struct EEPROM2 {
    #[packed_field(bits = "15..=0", endian = "msb")]
    pub(super) data: u16,
}

impl Addr for EEPROM2 {
    const ADDR: u8 = 0x06;
}

#[derive(PackedStruct, Debug, Default, PartialEq, Eq, Clone, Copy)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "2")]
pub(super) struct EEPROM3 {
    #[packed_field(bits = "15..=0", endian = "msb")]
    pub(super) data: u16,
}

impl Addr for EEPROM3 {
    const ADDR: u8 = 0x08;
}
