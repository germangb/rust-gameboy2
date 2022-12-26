use crate::{
    device::Device,
    error::{ReadError, WriteError},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::mem;
#[cfg(feature = "wasm-bindgen")]
use wasm_bindgen::prelude::*;

const BUTTON: u8 = 0b0010_0000;
const DIRECTION: u8 = 0b0001_0000;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[rustfmt::skip]
enum Select {
    Button    = BUTTON,
    Direction = DIRECTION,
}

#[repr(u8)]
#[cfg_attr(feature = "wasm-bindgen", wasm_bindgen)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[rustfmt::skip]
pub enum Button {
    A      = 0b0000_0001,
    B      = 0b0000_0010,
    Select = 0b0000_0100,
    Start  = 0b0000_1000,
    Right  = 0b0001_0000,
    Left   = 0b0010_0000,
    Up     = 0b0100_0000,
    Down   = 0b1000_0000,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(super) struct Joypad {
    select: Select,
    matrix: u8,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            select: Select::Button,
            matrix: 0,
        }
    }
}

impl Joypad {
    pub fn press(&mut self, button: &Button) {
        log::trace!("button press: {button:?}");
        self.matrix |= unsafe { mem::transmute::<_, u8>(*button) };
    }

    pub fn release(&mut self, button: &Button) {
        log::trace!("button release: {button:?}");
        self.matrix &= !unsafe { mem::transmute::<_, u8>(*button) };
    }
}

impl Device for Joypad {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff00 => {
                    let data = match self.select {
                        Select::Button => (self.matrix & 0xf) | BUTTON,
                        Select::Direction => (self.matrix >> 4) | DIRECTION,
                    };

                    // we're swapping the meaning of 0 and 1 internally
                    // so we need to invert the data bits
                    Ok(!data & 0b0011_1111)
                }
            }
        }
    }

    fn write(&mut self, address: u16, mut data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff00 => {
                    // we're swapping the meaning of 0 and 1 internally
                    // so we need to invert the data bits
                    data = !data;
                    data &= 0b0011_0000;

                    match data {
                        BUTTON => self.select = Select::Button,
                        DIRECTION => self.select = Select::Direction,
                        _ => {},
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Button;
    use crate::{cartridge::NoCartridge, device::Device, LR35902};

    #[test]
    fn button_select() {
        let mut emu = LR35902::new(NoCartridge);

        emu.joypad.press(&Button::A);
        emu.joypad.press(&Button::Select);

        // select button row
        emu.write(0xff00, 0b0001_0000).unwrap();

        assert_eq!(Ok(0b0001_1010), emu.joypad.read(0xff00));
    }

    #[test]
    fn direction_select() {
        let mut emu = LR35902::new(NoCartridge);

        emu.joypad.press(&Button::Right);
        emu.joypad.press(&Button::Up);

        // select direction row
        emu.write(0xff00, 0b0010_0000).unwrap();

        assert_eq!(Ok(0b0010_1010), emu.joypad.read(0xff00));
    }
}
