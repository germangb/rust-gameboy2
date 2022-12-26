use crate::device::Device;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// re-exports
use crate::error::{ReadError, WriteError};
#[cfg(feature = "mbc1")]
pub use mbc1::MBC1;
#[cfg(feature = "mbc3")]
pub use mbc3::MBC3;
#[cfg(feature = "mbc5")]
pub use mbc5::MBC5;

#[cfg(feature = "mbc1")]
mod mbc1;
#[cfg(feature = "mbc2")]
mod mbc2;
#[cfg(feature = "mbc3")]
mod mbc3;
#[cfg(feature = "mbc5")]
mod mbc5;

fn decode_ram_banks(banks: u8) -> usize {
    match banks {
        0x00 => 0,
        0x01 | 0x02 => 1,
        0x03 => 4,
        0x04 => 16,
        _ => panic!(),
    }
}

/// An empty touple represents the absence of cartride.
pub trait Cartridge: Device {}

impl Device for () {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0x0000..=0x7fff => Ok(0xff),
                0xa000..=0xbfff => Ok(0xff),
            }
        }
    }

    #[allow(unused_variables)]
    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0x0000..=0x7fff => {},
                0xa000..=0xbfff => {},
            }
        }
        Ok(())
    }
}

impl Cartridge for () {}

/// Non-switchable ROM and RAM banks.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ROM {
    rom: Box<[u8]>,
    ram: Box<[u8]>,
}

impl ROM {
    pub fn new(rom: Box<[u8]>) -> Self {
        Self {
            rom,
            ram: vec![0; 0x2000].into_boxed_slice(),
        }
    }
}

impl Cartridge for ROM {}

impl Device for ROM {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0x0000..=0x7fff => Ok(self.rom[address as usize]),
                0xa000..=0xbfff => Ok(self.ram[(address as usize) - 0xa000]),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0x0000..=0x7fff => return Err(WriteError::ROMAddress(address, data)),
                0xa000..=0xbfff => self.ram[address as usize - 0xa000] = data,
            }
        }

        Ok(())
    }
}
