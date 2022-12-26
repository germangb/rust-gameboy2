use crate::{
    cartridge::{decode_ram_banks, Cartridge},
    device::Device,
    error::{ReadError, WriteError},
};
use log::info;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
enum Mode {
    Ram,
    Rtc,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MBC3 {
    rom: Box<[u8]>,
    ram: Box<[u8]>,
    // The Clock Counter Registers
    // 08h  RTC S   Seconds   0-59 (0-3Bh)
    // 09h  RTC M   Minutes   0-59 (0-3Bh)
    // 0Ah  RTC H   Hours     0-23 (0-17h)
    // 0Bh  RTC DL  Lower 8 bits of Day Counter (0-FFh)
    // 0Ch  RTC DH  Upper 1 bit of Day Counter, Carry Bit, Halt Flag
    //         Bit 0  Most significant bit of Day Counter (Bit 8)
    //         Bit 6  Halt (0=Active, 1=Stop Timer)
    //         Bit 7  Day Counter Carry Bit (1=Counter Overflow)
    rtc: [u8; 5],
    rtc_select: usize,
    rom_bank: usize,
    ram_bank: usize,
    ram_timer_enabled: bool,
    mode: Mode,
}

impl MBC3 {
    pub fn new(rom: Box<[u8]>) -> Self {
        let ram_banks = decode_ram_banks(rom[0x149]);
        let ram_banks = 8;
        Self {
            rom,
            ram: vec![0; 0x2000 * ram_banks].into_boxed_slice(),
            rtc: [0; 5],
            rtc_select: 0,
            rom_bank: 0,
            ram_bank: 0,
            ram_timer_enabled: false,
            mode: Mode::Ram,
        }
    }

    fn rom_bank_address(&self, address: u16) -> usize {
        0x4000 * self.rom_bank.max(1) + (address as usize) - 0x4000
    }

    fn ram_bank_address(&self, address: u16) -> usize {
        0x2000 * self.ram_bank + (address as usize) - 0xa000
    }
}

impl Cartridge for MBC3 {}

impl Device for MBC3 {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0x0000..=0x3fff => Ok(self.rom[address as usize]),
                0x4000..=0x7fff => Ok(self.rom[self.rom_bank_address(address)]),
                0xa000..=0xbfff if self.ram_timer_enabled => match self.mode {
                    Mode::Ram => Ok(self.ram[self.ram_bank_address(address)]),
                    Mode::Rtc => Ok(self.rtc[self.rtc_select]),
                },
                0xa000..=0xbfff => Ok(0xff),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0x0000..=0x1fff => self.ram_timer_enabled = (data & 0xf) == 0xa,
                0x2000..=0x3fff => {
                    self.rom_bank = data as usize;

                    info!("Selected ROM bank: {}", self.rom_bank);
                },
                0x4000..=0x5fff => match data {
                    0x00..=0x03 => {
                        self.mode = Mode::Ram;
                        self.ram_bank = data as usize
                    }
                    0x08..=0x0c => {
                        self.mode = Mode::Rtc;
                        self.rtc_select = (data as usize) - 0x08
                    }
                    _ => panic!(),
                },
                0x6000..=0x7fff => {}
                0xa000..=0xbfff if self.ram_timer_enabled => match self.mode {
                    Mode::Ram => self.ram[self.ram_bank_address(address)] = data,
                    Mode::Rtc => self.rtc[self.rtc_select] = data,
                },
                0xa000..=0xbfff => {}
            }
        }

        Ok(())
    }
}
