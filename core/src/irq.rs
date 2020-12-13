use crate::device::{invalid_read, invalid_write, Device};
use log::info;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug)]
pub struct Request {
    pub vblank: bool,
    pub lcd_stat: bool,
    pub joypad: bool,
    pub serial: bool,
    pub timer: bool,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IRQ {
    fi: u8,
    ie: u8,
}

impl Device for IRQ {
    const DEBUG_NAME: &'static str = "IRQ";

    fn read(&self, address: u16) -> u8 {
        match address {
            0xff0f => self.fi,
            0xffff => self.ie,
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0xff0f => self.fi = data,
            0xffff => {
                info!("IE = {:#08b}", data);

                self.ie = data
            }
            _ => invalid_write(address),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Emulator};

    #[test]
    fn irq() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff0f, 0x12);
        emu.write(0xffff, 0xbc);

        assert_eq!([0x12, 0xbc], [emu.irq.fi, emu.irq.ie]);
    }
}
