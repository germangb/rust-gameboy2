use crate::{cartridge::Cartridge, device::Device, error::Error};
use log::warn;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
    const DEBUG_NAME: &'static str = "ROM";

    fn read(&self, address: u16) -> Result<u8, Error> {
        match address {
            0x0000..=0x7fff => Ok(self.rom[address as usize]),
            0xa000..=0xbfff => Ok(self.ram[address as usize - 0xa000]),
            _ => Err(Error::InvalidAddr(address)),
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), Error> {
        match address {
            0x0000..=0x7fff => {
                warn!(
                    "Attempt to write to ROM, address: {:#04x}, data = {:#02x}",
                    address, data
                );
            }
            0xa000..=0xbfff => self.ram[address as usize - 0xa000] = data,
            _ => return Err(Error::InvalidAddr(address)),
        }

        Ok(())
    }
}
