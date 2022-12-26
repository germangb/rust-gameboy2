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
    rom_bank: usize,
    ram: Box<[u8]>,
    ram_bank: usize,
    ram_enabled: bool,
}

impl MBC5 {
    pub fn new(rom: Box<[u8]>) -> Self {
        let ram_banks = decode_ram_banks(rom[0x149]);
        Self {
            rom,
            rom_bank: 0,
            ram: vec![0x00; 0x2000 * ram_banks].into_boxed_slice(),
            ram_bank: 0,
            ram_enabled: true,
        }
    }

    fn rom_bank_addr(&self, address: u16) -> usize {
        (address as usize) - 0x4000 + (0x4000 * self.rom_bank)
    }

    fn ram_bank_addr(&self, address: u16) -> usize {
        (address as usize) - 0xa000 + (0x2000 * self.ram_bank)
    }
}

impl Cartridge for MBC5 {}

impl Device for MBC5 {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0x0000..=0x3fff => Ok(self.rom[address as usize]),
                0x4000..=0x7fff => {
                    let address = self.rom_bank_addr(address);
                    Ok(self.rom.get(address).copied().unwrap_or(0x00))
                }
                0xa000..=0xbfff => {
                    if self.ram_enabled {
                        let addr = self.ram_bank_addr(address);
                        let data = self.ram.get(addr).copied();
                        Ok(data.unwrap_or(0x00))
                    } else {
                        Ok(0x00)
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
                    //println!("ROM BANK {}", self.rom_bank);
                }
                0x3000..=0x3fff => {
                    self.rom_bank &= 0xff;
                    self.rom_bank |= (data as usize & 0x1) << 8;
                    //println!(">ROM BANK {}", self.rom_bank);
                }
                0x4000..=0x5fff => {
                    self.ram_bank = (data & 0xf) as usize;
                    //println!("RAM BANK {}", self.ram_bank);
                },
                0x6000..=0x7fff => { /* read-only */ }
                0xa000..=0xbfff => {
                    let banks = self.ram.len() / 0x2000;
                    if self.ram_enabled && self.ram_bank < banks {
                        self.ram[self.ram_bank_addr(address)] = data
                    }
                }
            }
        }

        Ok(())
    }
}
