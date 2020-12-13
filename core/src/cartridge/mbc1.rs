use crate::{
    cartridge::{ram_banks, Cartridge},
    device::{invalid_read, invalid_write, Device},
};
use log::info;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

enum Mode {
    Rom,
    Ram,
}

pub struct MBC1 {
    rom: Box<[u8]>,
    ram: Vec<[u8; 0x2000]>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    mode: Mode,
}

impl MBC1 {
    pub fn new(rom: Vec<u8>) -> Self {
        let ram_banks = ram_banks(rom[0x149]);
        Self {
            rom: rom.into_boxed_slice(),
            ram: vec![[0; 0x2000]; ram_banks],
            rom_bank: 0,
            ram_bank: 0,
            ram_enable: false,
            mode: Mode::Rom,
        }
    }

    fn rom_addr(&self, addr: u16) -> usize {
        0x4000 * self.rom_bank.max(1) + (addr as usize) - 0x4000
    }
}

impl Cartridge for MBC1 {}

impl Device for MBC1 {
    const DEBUG_NAME: &'static str = "ROM (MBC1)";

    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => self.rom.get(address as usize).copied().unwrap_or(0xff),
            0x4000..=0x7fff => {
                let addr = self.rom_addr(address);
                self.rom.get(addr).copied().unwrap_or(0)
            }
            0xa000..=0xbfff => {
                if self.ram_enable {
                    self.ram
                        .get(self.ram_bank)
                        .map(|bank| bank[address as usize - 0xa000])
                        .unwrap_or(0)
                } else {
                    0
                }
            }
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            // Before external RAM can be read or written, it must be enabled by writing to this
            // address space. It is recommended to disable external RAM after accessing it, in order
            // to protect its contents from damage during power down of the gameboy. Usually the
            // following values are used:
            0x0000..=0x1fff => self.ram_enable = data & 0xf == 0xa,
            // Writing to this address space selects the lower 5 bits of the ROM Bank Number (in
            // range 01-1Fh). When 00h is written, the MBC translates that to bank 01h also. That
            // doesn't harm so far, because ROM Bank 00h can be always directly accessed by reading
            // from 0000-3FFF.
            0x2000..=0x3fff => {
                self.rom_bank &= 0x60;
                self.rom_bank |= data as usize & 0x1f;

                info!("Selected ROM bank (lower 5 bits): {}", self.rom_bank);
            }
            // This 2bit register can be used to select a RAM Bank in range from 00-03h, or to
            // specify the upper two bits (Bit 5-6) of the ROM Bank number, depending on the current
            // ROM/RAM Mode. (See below.)
            0x4000..=0x5fff => match self.mode {
                Mode::Rom => {
                    info!("Selected ROM bank (upper 2 bits): {}", data);

                    self.rom_bank &= 0x1f;
                    self.rom_bank |= (data as usize & 0x3) << 5;
                }
                Mode::Ram => {
                    info!("Selected RAM bank (upper 2 bits): {}", data);

                    self.ram_bank = data as usize & 0x3;
                }
            },
            0x6000..=0x7fff => {
                self.mode = match data {
                    0x00 => Mode::Rom,
                    0x01 => Mode::Ram,
                    _ => panic!(),
                }
            }
            0xa000..=0xbfff => {
                if let Some(bank) = self.ram.get_mut(self.ram_bank) {
                    bank[address as usize - 0xa000] = data
                }
            }
            _ => invalid_write(address),
        }
    }
}
