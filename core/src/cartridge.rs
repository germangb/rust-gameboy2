use crate::{
    device::{invalid_read, invalid_write, Device},
    Update,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod mbc1;
pub mod mbc3;
pub mod rom;

pub trait Cartridge: Device {}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NoCartridge;

impl Cartridge for NoCartridge {}

impl Device for NoCartridge {
    const DEBUG_NAME: &'static str = "No-Cartridge";

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

fn ram_banks(banks: u8) -> usize {
    match banks {
        0x00 => 0,
        0x01 | 0x02 => 1,
        0x03 => 4,
        0x04 => 16,
        _ => panic!(),
    }
}
