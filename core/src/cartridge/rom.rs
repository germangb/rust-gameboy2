use crate::{
    cartridge::Cartridge,
    device::{invalid_read, invalid_write, Device},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rom {
    rom: Box<[u8]>,
    ram: Box<[u8]>,
}

impl Rom {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            rom: data.into_boxed_slice(),
            ram: vec![0; 0x2000].into_boxed_slice(),
        }
    }
}

impl Cartridge for Rom {}

impl Device for Rom {
    const DEBUG_NAME: &'static str = "ROM";

    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7fff => self.rom[address as usize],
            0xa000..=0xbfff => self.ram[address as usize - 0xa000],
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x7fff => self.rom[address as usize] = data,
            0xa000..=0xbfff => self.ram[address as usize - 0xa000] = data,
            _ => invalid_write(address),
        }
    }
}
