use crate::{
    cartridge::{decode_ram_banks, Cartridge},
    device::Device,
    error::{ReadError, WriteError},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
enum Mode {
    Rom,
    Ram,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct MBC1 {
    rom: Box<[u8]>,
    ram: Box<[u8]>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    mode: Mode,
}

impl MBC1 {
    pub fn new(rom: Box<[u8]>) -> Self {
        let ram_banks = decode_ram_banks(rom[0x149]);
        let ram_banks = 42; //decode_ram_banks(rom[0x149]);
        Self {
            rom,
            ram: vec![0u8; 0x2000 * ram_banks].into_boxed_slice(),
            rom_bank: 0,
            ram_bank: 0,
            ram_enable: false,
            mode: Mode::Rom,
        }
    }

    fn rom_bank_address(&self, address: u16) -> usize {
        0x4000 * self.rom_bank.max(1) + (address as usize) - 0x4000
    }

    fn ram_bank_address(&self, address: u16) -> usize {
        0x2000 * self.ram_bank + (address as usize) - 0xa000
    }
}

impl Cartridge for MBC1 {}

impl Device for MBC1 {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0x0000..=0x3fff => Ok(self.rom[address as usize]),
                0x4000..=0x7fff => Ok(self.rom[self.rom_bank_address(address)]),
                0xa000..=0xbfff if self.ram_enable => Ok(self.ram[self.ram_bank_address(address)]),
                0xa000..=0xbfff => Ok(0xff),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0x0000..=0x1fff => self.ram_enable = data & 0xf == 0xa,
                0x2000..=0x3fff => {
                    self.rom_bank &= 0x60;
                    self.rom_bank |= data as usize & 0x1f;
                }
                0x4000..=0x5fff => match self.mode {
                    Mode::Rom => {
                        self.rom_bank &= 0x1f;
                        self.rom_bank |= (data as usize & 0x3) << 5;
                    }
                    Mode::Ram => self.ram_bank = data as usize & 0x3,
                },
                0x6000..=0x7fff => {
                    self.mode = match data {
                        0x00 => Mode::Rom,
                        0x01 => Mode::Ram,
                        _ => panic!(),
                    }
                }
                0xa000..=0xbfff => self.ram[self.ram_bank_address(address)] = data,
            }
        }

        Ok(())
    }
}
