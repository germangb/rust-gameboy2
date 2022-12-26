use crate::{
    device::Device,
    error::{ReadError, WriteError},
    irq,
    ppu::LCD_HEIGHT,
};
use log::warn;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const SEARCH_DOTS: u64 = 80; // 80 dots (19 us)
const PIXELS_DOTS: u64 = 200; // 168 to 291 cycles (40 to 60 us) depending on sprite count
const HBLANK_DOTS: u64 = 85 + (291 - PIXELS_DOTS); // 85 to 208 dots (20 to 49 us) depending on previous mode 3 duration
const VBLANK_DOTS: u64 = 456; // 4560 dots (1087 us, 10 scanlines)

#[repr(u8)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Mode {
    HBLANK = 0,
    VBLANK = 1,
    SEARCH = 2,
    PIXEL0 = 3,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct STAT {
    dots: u64,
    stat: u8,
    ly: u8,
    lyc: u8,
    // TODO better name
    pub lyc_hist: Box<[bool; LCD_HEIGHT]>,
}

impl Default for STAT {
    fn default() -> Self {
        Self {
            dots: SEARCH_DOTS,
            stat: 0b10,
            ly: 0,
            lyc: 0,
            lyc_hist: Box::new([false; LCD_HEIGHT]),
        }
    }
}

impl STAT {
    pub fn reset(&mut self) {
        self.dots = SEARCH_DOTS;
        self.ly = 0;
        self.update_lyc_flag();
        self.set_mode(Mode::SEARCH);
    }

    pub fn ly(&self) -> u8 {
        self.ly
    }

    pub fn lyc(&self) -> u8 {
        self.lyc
    }

    #[cfg(todo)]
    pub fn stat(&self) -> u8 {
        self.stat
    }

    pub fn mode(&self) -> Mode {
        unsafe { std::mem::transmute(self.stat & 0x3) }
    }

    /// Return number of dots emulated on the current scanline.
    pub fn dots(&self) -> u64 {
        // TODO(german) fix the lie in the comment above. This is actually the number of
        // dots _remaining_, which is how the current (buggy) emulation works
        self.dots
    }

    fn lyc_int(&self) -> bool {
        (self.stat & 0b0100_0000) != 0
    }

    fn oam_int(&self) -> bool {
        (self.stat & 0b0010_0000) != 0
    }

    fn vblank_int(&self) -> bool {
        (self.stat & 0b0001_0000) != 0
    }

    fn hblank_int(&self) -> bool {
        (self.stat & 0b0000_1000) != 0
    }

    fn set_mode(&mut self, mode: Mode) {
        let mode = match mode {
            Mode::HBLANK => 0,
            Mode::VBLANK => 1,
            Mode::SEARCH => 2,
            Mode::PIXEL0 => 3,
        };
        self.stat &= 0b1111_1100;
        self.stat |= mode;
    }

    // OAM search state
    fn search(&mut self, ticks: u64) {
        if ticks > self.dots {
            self.dots = PIXELS_DOTS - (ticks - self.dots);
        } else {
            self.dots -= ticks;
            return;
        }

        self.set_mode(Mode::PIXEL0);
    }

    // transfer data to LCD state
    fn pixel0(&mut self, ticks: u64, flags: &mut irq::Flags) {
        if ticks > self.dots {
            self.dots = HBLANK_DOTS - (ticks - self.dots);
        } else {
            self.dots -= ticks;
            return;
        }
        if self.hblank_int() {
            flags.set(irq::Flags::LCD_STAT, true);
        }
        self.set_mode(Mode::HBLANK);
    }

    // horizontal blank state
    fn hblank(&mut self, ticks: u64, flags: &mut irq::Flags) {
        if ticks <= self.dots {
            self.dots -= ticks;
            return;
        }

        // increment line
        self.ly += 1;

        if self.ly == 144 {
            if self.vblank_int() {
                flags.set(irq::Flags::LCD_STAT, true);
            }
            flags.set(irq::Flags::VBLANK, true);
            self.dots = VBLANK_DOTS - (ticks - self.dots);
            self.set_mode(Mode::VBLANK);
        } else {
            if self.oam_int() {
                flags.set(irq::Flags::LCD_STAT, true);
            }
            self.dots = SEARCH_DOTS - (ticks - self.dots);
            self.set_mode(Mode::SEARCH);
        }
    }

    // vertical vblank state
    fn vblank(&mut self, ticks: u64, flags: &mut irq::Flags) {
        if ticks <= self.dots {
            self.dots -= ticks;
            return;
        }

        // increment line
        self.ly += 1;

        // vblank has ended, switch to fist OAM search
        if self.ly == 154 {
            if self.oam_int() {
                flags.set(irq::Flags::LCD_STAT, true);
            }
            self.ly = 0;
            self.dots = SEARCH_DOTS - (ticks - self.dots);
            self.set_mode(Mode::SEARCH);
            let _ = std::mem::replace(self.lyc_hist.as_mut(), [false; LCD_HEIGHT]);
        } else {
            self.dots = VBLANK_DOTS - (ticks - self.dots);
        }
    }

    fn update_lyc_flag(&mut self) {
        if self.ly == self.lyc {
            self.stat |= 0b0000_0100;
        } else {
            self.stat &= 0b1111_1011;
        }
    }

    pub fn update(&mut self, ticks: u64, flags: &mut irq::Flags) {
        //self.dots += ticks;
        let ly = self.ly;
        match self.mode() {
            Mode::SEARCH => self.search(ticks),
            Mode::PIXEL0 => self.pixel0(ticks, flags),
            Mode::HBLANK => self.hblank(ticks, flags),
            Mode::VBLANK => self.vblank(ticks, flags),
        }
        self.update_lyc_flag();

        // check if ly == (lyc register)
        // if that's the case, the ly interrupt is triggered
        if self.lyc_int() && self.ly == self.lyc && self.ly != ly {
            flags.set(irq::Flags::LCD_STAT, true);

            // keep track ot where the ly==lyc interrupt has been requested
            if (self.lyc as usize) < LCD_HEIGHT {
                self.lyc_hist[self.lyc as usize] = true;
            }
        }
    }
}

impl Device for STAT {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff41 => Ok(self.stat),
                0xff44 => Ok(self.ly),
                0xff45 => Ok(self.lyc),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff41 => {
                    self.stat &= 0x7;
                    self.stat |= data & 0xf8;
                }
                0xff44 => {
                    warn!("WRITE to LY is undefined.");
                }
                0xff45 => {
                    self.lyc = data;
                    self.update_lyc_flag();
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {}
