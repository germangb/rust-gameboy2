use crate::device::{Device, Result};
use log::warn;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// re-exports
pub use mbc1::MBC1;
pub use mbc3::MBC3;

pub mod mbc1;
pub mod mbc2;
pub mod mbc3;

fn ram_banks(banks: u8) -> usize {
    match banks {
        0x00 => 0,
        0x01 | 0x02 => 1,
        0x03 => 4,
        0x04 => 16,
        _ => panic!(),
    }
}

pub trait Cartridge: Device {}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NoCartridge;

impl Cartridge for NoCartridge {}

impl Device for NoCartridge {
    fn read(&self, address: u16) -> Result<u8> {
        device_match! {
            address {
                0x0000..=0x7fff => Ok(0xff),
                0xa000..=0xbfff => Ok(0xff),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        device_match! {
            address {
                0x0000..=0x7fff => {},
                0xa000..=0xbfff => {},
            }
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ROM {
    rom: Box<[u8]>,
    ram: Box<[u8]>,
}

impl ROM {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            rom: data.into_boxed_slice(),
            ram: vec![0; 0x2000].into_boxed_slice(),
        }
    }
}

impl Cartridge for ROM {}

impl Device for ROM {
    fn read(&self, address: u16) -> Result<u8> {
        device_match! {
            address {
                0x0000..=0x7fff => Ok(self.rom[address as usize]),
                0xa000..=0xbfff => Ok(self.ram[(address as usize) - 0xa000]),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        device_match! {
            address {
                0x0000..=0x7fff => warn!("[WRITE] ROM address."),
                0xa000..=0xbfff => self.ram[address as usize - 0xa000] = data,
            }
        }

        Ok(())
    }
}
