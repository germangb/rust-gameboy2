use crate::{
    device::{invalid_read, invalid_write, Device},
    Update,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub trait Cartridge: Device {}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NoCartridge;

impl Cartridge for NoCartridge {}

impl Device for NoCartridge {
    fn debug_name() -> &'static str {
        "NoCartridge"
    }

    fn read(&self, address: u16) -> u8 {
        if matches!(address, 0x0000..=0x7fff | 0xa000..=0xbfff) {
            0xff
        } else {
            invalid_read(address)
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        if !matches!(address, 0x0000..=0x7fff | 0xa000..=0xbfff) {
            invalid_write(address)
        }
    }
}

pub struct SingleBank;
pub struct MBC1;
pub struct MBC3;
pub struct MBC5;
