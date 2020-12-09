use crate::dev::Device;
use log::{info, warn};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::mem;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
enum Select {
    Button = 0b0010_0000,
    Direction = 0b0001_0000,
    Undefined = 0b0000_0000,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Button {
    A = 0b0000_0001,
    B = 0b0000_0010,
    Select = 0b0000_0100,
    Start = 0b0000_1000,
    Right = 0b0001_0000,
    Left = 0b0010_0000,
    Up = 0b0100_0000,
    Down = 0b1000_0000,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Joypad {
    select: Select,
    matrix: u8,
}

impl Joypad {
    fn press(&mut self, button: Button) {
        info!("Button press: {:?}", button);
        self.matrix |= unsafe { mem::transmute::<_, u8>(button) };
    }

    fn release(&mut self, button: Button) {
        info!("Button release: {:?}", button);
        self.matrix &= !unsafe { mem::transmute::<_, u8>(button) };
    }
}

impl Device for Joypad {
    fn read(&self, address: u16) -> u8 {
        assert_eq!(0xff00, address);

        let data = match self.select {
            Select::Button => (self.matrix & 0xf) | 0b0010_0000,
            Select::Direction => (self.matrix >> 4) | 0b0001_0000,
            Select::Undefined => unreachable!(),
        };

        // we're swapping the meaning of 0 and 1 internally
        // so we need to invert the data bits
        !data & 0b0011_1111
    }

    fn write(&mut self, address: u16, mut data: u8) {
        assert_eq!(0xff00, address);

        // we're swapping the meaning of 0 and 1 internally
        // so we need to invert the data bits
        data = !data;
        data &= 0b0011_0000;

        let data: Select = unsafe { mem::transmute(data) };

        match data {
            Select::Undefined => warn!("Undefined select mode"),
            s => self.select = s,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Button, Joypad, Select};
    use crate::dev::Device;

    fn joypad() -> Joypad {
        Joypad {
            select: Select::Button,
            matrix: 0,
        }
    }

    #[test]
    fn button_select() {
        let mut joypad = joypad();

        joypad.press(Button::A);
        joypad.press(Button::Select);

        // select button row
        joypad.write(0xff00, 0b0001_0000);

        assert_eq!(0b0001_1010, joypad.read(0xff00));
    }

    #[test]
    fn direction_select() {
        let mut joypad = joypad();

        joypad.press(Button::Right);
        joypad.press(Button::Up);

        // select direction row
        joypad.write(0xff00, 0b0010_0000);

        assert_eq!(0b0010_1010, joypad.read(0xff00));
    }
}
