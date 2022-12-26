use crate::{
    cartridge::Cartridge,
    device::Device,
    error::{ReadError, WriteError},
};

pub struct MBC2 {
    rom: Box<[u8]>,
    ram: Box<[u8]>,
    rom_bank: usize,
    ram_enabled: bool,
}

impl MBC2 {
    pub fn new(rom: Box<[u8]>) -> Self {
        Self {
            rom,
            ram: vec![0u8; 0x200].into_boxed_slice(),
            rom_bank: 0,
            ram_enabled: false,
        }
    }

    fn rom_bank_address(&self, address: u16) -> usize {
        0x4000 * self.rom_bank.max(1) + (address as usize) - 0x4000
    }
}

impl Cartridge for MBC2 {}

impl Device for MBC2 {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0x0000..=0x3fff => Ok(self.rom[address as usize]),
                0x4000..=0x7fff => Ok(self.rom[self.rom_bank_address(address)]),
                0xa000..=0xa1ff => Ok(self.ram[(address - 0xa000) as usize] & 0x0f),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0x0000..=0x1fff => { /*???*/ },
                0x2000..=0x3fff => self.rom_bank = (data & 0x0f) as usize,
                0xa000..=0xa1ff => self.ram[(address - 0xa000) as usize] = (data & 0x0f),
            }
        }

        Ok(())
    }
}
