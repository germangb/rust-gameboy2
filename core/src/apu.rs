use crate::{
    device::Device,
    error::{Component, ReadError, WriteError},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Registers
#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct APU {
    // Sound Channel 1 - Tone & Sweep
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
    // Sound Channel 2 - Tone
    nr20: u8,
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
    // Sound Channel 3 - Wave Output
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
    wave_ram: [u8; 0x10],
    // Sound Channel 4 - Noise
    nr40: u8,
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
    // Sound Control Registers
    nr50: u8,
    // Bit 7 - Output sound 4 to SO2 terminal
    // Bit 6 - Output sound 3 to SO2 terminal
    // Bit 5 - Output sound 2 to SO2 terminal
    // Bit 4 - Output sound 1 to SO2 terminal
    // Bit 3 - Output sound 4 to SO1 terminal
    // Bit 2 - Output sound 3 to SO1 terminal
    // Bit 1 - Output sound 2 to SO1 terminal
    // Bit 0 - Output sound 1 to SO1 terminal
    nr51: u8,
    // Bit 7 - All sound on/off  (0: stop all sound circuits) (Read/Write)
    // Bit 3 - Sound 4 ON flag (Read Only)
    // Bit 2 - Sound 3 ON flag (Read Only)
    // Bit 1 - Sound 2 ON flag (Read Only)
    // Bit 0 - Sound 1 ON flag (Read Only)
    nr52: u8,
}

impl APU {
    fn clear_reg(&mut self) {
        self.nr10 = 0;
        self.nr11 = 0;
        self.nr12 = 0;
        self.nr13 = 0;
        self.nr14 = 0;

        self.nr20 = 0;
        self.nr21 = 0;
        self.nr22 = 0;
        self.nr23 = 0;
        self.nr24 = 0;

        self.nr30 = 0;
        self.nr31 = 0;
        self.nr32 = 0;
        self.nr33 = 0;
        self.nr34 = 0;

        self.nr40 = 0;
        self.nr41 = 0;
        self.nr42 = 0;
        self.nr43 = 0;
        self.nr44 = 0;

        self.nr50 = 0;
        self.nr51 = 0;
        self.nr52 = 0;
    }
}

impl Device for APU {
    #[allow(unused_variables)]
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                //      NRx0 NRx1 NRx2 NRx3 NRx4
                //     ---------------------------
                // NR1x  $80  $3F $00  $FF  $BF
                // NR2x  $FF  $3F $00  $FF  $BF
                // NR3x  $7F  $FF $9F  $FF  $BF
                // NR4x  $FF  $FF $00  $00  $BF
                // NR5x  $00  $00 $70
                //     ---------------------------
                // Sound Channel 1 - Tone & Sweep
                0xff10 => Ok(self.nr10 | 0x80),
                0xff11 => Ok(self.nr11 | 0x3f),
                0xff12 => Ok(self.nr12 | 0x00),
                0xff13 => Ok(self.nr13 | 0xff),
                0xff14 => Ok(self.nr14 | 0xbf),
                // Sound Channel 2 - Tone
                0xff15 => Ok(self.nr20 | 0xff),
                0xff16 => Ok(self.nr21 | 0x3f),
                0xff17 => Ok(self.nr22 | 0x00),
                0xff18 => Ok(self.nr23 | 0xff),
                0xff19 => Ok(self.nr24 | 0xbf),
                // Sound Channel 3 - Wave Output
                0xff1a => Ok(self.nr30 | 0x7f),
                0xff1b => Ok(self.nr31 | 0xff),
                0xff1c => Ok(self.nr32 | 0x9f),
                0xff1d => Ok(self.nr33 | 0xff),
                0xff1e => Ok(self.nr34 | 0xbf),
                // Sound Channel 4 - Noise
                0xff1f => Ok(self.nr40 | 0xff),
                0xff20 => Ok(self.nr41 | 0xff),
                0xff21 => Ok(self.nr42 | 0x00),
                0xff22 => Ok(self.nr43 | 0x00),
                0xff23 => Ok(self.nr44 | 0xbf),
                // Sound Control Registers
                0xff24 => Ok(self.nr50),
                0xff25 => Ok(self.nr51),
                0xff26 => Ok(self.nr52 | 0x70),
                // $FF27-$FF2F always read back as $FF
                0xff27..=0xff2f => Ok(0xff),
                0xff30..=0xff3f => Ok(self.wave_ram[address as usize - 0xff30]),
            }
        }
    }

    #[allow(unused_variables)]
    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        if self.nr52 & 0x80 != 0 {
            dev_write! {
                address, data {
                    // Channel 1 sweep
                    0xff10 => self.nr10 = data,
                    0xff11 => self.nr11 = data,
                    0xff12 => self.nr12 = data,
                    0xff13 => self.nr13 = data,
                    0xff14 => self.nr14 = data,
                    // Channel 2 - Tone
                    0xff15 => self.nr20 = data,
                    0xff16 => self.nr21 = data,
                    0xff17 => self.nr22 = data,
                    0xff18 => self.nr23 = data,
                    0xff19 => self.nr24 = data,
                    // Channel 3 - Wave RAM
                    0xff1a => self.nr30 = data,
                    0xff1b => self.nr31 = data,
                    0xff1c => self.nr32 = data,
                    0xff1d => self.nr33 = data,
                    0xff1e => self.nr34 = data,
                    // Channel 4 - Noise
                    0xff1f => self.nr40 = data,
                    0xff20 => self.nr41 = data,
                    0xff21 => self.nr42 = data,
                    0xff22 => self.nr43 = data,
                    0xff23 => self.nr44 = data,
                    // Sound Control Registers
                    0xff24 => self.nr50 = data,
                    0xff25 => self.nr51 = data,
                    0xff26 => { /* Handled below */ }
                    0xff27..=0xff2f => { /* Unused */ }
                    0xff30..=0xff3f => { /* Handled below */ }
                }
            }
        }

        // wave ram unaffected by power state
        if let 0xff30..=0xff3f = address {
            self.wave_ram[address as usize - 0xff30] = data;
        }

        // so is NR52
        if address == 0xff26 {
            self.nr52 &= 0x7f;
            self.nr52 |= data & 0x80;

            if self.nr52 & 0x80 == 0 {
                self.clear_reg();
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {}
