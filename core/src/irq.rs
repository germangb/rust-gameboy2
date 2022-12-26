use crate::{
    device::Device,
    error::{ReadError, WriteError},
};
use bitflags::bitflags;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Flags: u8 {
        const VBLANK   = 0b00001;
        const LCD_STAT = 0b00010;
        const TIMER    = 0b00100;
        const SERIAL   = 0b01000;
        const JOYPAD   = 0b10000;
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IRQ {
    pub fi: Flags,
    pub ie: Flags,
}

impl Device for IRQ {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff0f => Ok(self.fi.bits),
                0xffff => Ok(self.ie.bits),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff0f => self.fi = Flags::from_bits(data).unwrap(),
                0xffff => self.ie = Flags::from_bits(data).unwrap(),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, LR35902};

    #[test]
    fn irq() {
        let mut emu = LR35902::new(NoCartridge);

        emu.write(0xff0f, 0b11111).unwrap();
        emu.write(0xffff, 0b11001).unwrap();

        assert_eq!([0b11111, 0b11001], [emu.irq.fi.bits, emu.irq.ie.bits]);
    }
}
