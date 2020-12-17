use crate::{device::Device, Update};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::error::Error;
pub use mbc1::MBC1;
pub use mbc3::MBC3;
pub use rom::ROM;

pub mod mbc1;
pub mod mbc3;
pub mod rom;

pub trait Cartridge: Device {}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NoCartridge;

impl Cartridge for NoCartridge {}

impl Device for NoCartridge {
    const DEBUG_NAME: &'static str = "No-Cartridge";

    fn read(&self, address: u16) -> Result<u8, Error> {
        if matches!(address, 0x0000..=0x7fff | 0xa000..=0xbfff) {
            Ok(0xff)
        } else {
            Err(Error::InvalidAddr(address))
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), Error> {
        if !matches!(address, 0x0000..=0x7fff | 0xa000..=0xbfff) {
            Err(Error::InvalidAddr(address))
        } else {
            Ok(())
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
