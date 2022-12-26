use crate::{
    cartridge::{decode_ram_banks, Cartridge},
    device::Device,
    error::{ReadError, WriteError},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// MBC5 controller.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MBC5 {
    rom: Box<[u8]>,
    ram: Box<[u8]>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
}

impl MBC5 {
    pub fn new(rom: Box<[u8]>) -> Self {
        //let ram_banks = decode_ram_banks(rom[0x149]);
        let ram_banks = 16;
        Self {
            rom,
            ram: vec![0; 0x2000 * ram_banks].into_boxed_slice(),
            rom_bank: 0,
            ram_bank: 0,
            ram_enabled: true,
        }
    }

    fn rom_bank_address(&self, address: u16) -> usize {
        0x4000 * self.rom_bank + (address as usize) - 0x4000
    }

    fn ram_bank_address(&self, address: u16) -> usize {
        0x2000 * self.ram_bank + (address as usize) - 0xa000
    }
}

impl Cartridge for MBC5 {}

impl Device for MBC5 {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0x0000..=0x3fff => Ok(self.rom[address as usize]),
                0x4000..=0x7fff => {
                    Ok(self.rom[self.rom_bank_address(address)])
                }
                0xa000..=0xbfff => {
                    if self.ram_enabled {
                        Ok(self.ram[self.ram_bank_address(address)])
                    } else {
                        Ok(0)
                    }
                }
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0x0000..=0x1fff => self.ram_enabled = (data & 0xf) == 0xa,
                0x2000..=0x2fff => {
                    self.rom_bank &= !0xff;
                    self.rom_bank |= data as usize;
                }
                0x3000..=0x3fff => {
                    self.rom_bank &= 0xff;
                    self.rom_bank |= (data as usize & 0x1) << 8;
                }
                0x4000..=0x5fff => self.ram_bank = (data & 0xf) as usize,
                0x6000..=0x7fff => { /* read-only */ }
                0xa000..=0xbfff => {
                    if self.ram_enabled {
                        self.ram[self.ram_bank_address(address)] = data
                    }
                }
            }
        }

        Ok(())
    }
}
