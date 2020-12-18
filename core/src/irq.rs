use crate::{
    device::{Device, Result},
    error::Error,
};
use bitflags::bitflags;
use log::info;
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
    const DEBUG_NAME: &'static str = "IRQ";

    fn read(&self, address: u16) -> Result<u8> {
        match address {
            0xff0f => Ok(self.fi.bits),
            0xffff => Ok(self.ie.bits),
            _ => Err(Error::InvalidAddr(address)),
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        match address {
            0xff0f => self.fi = Flags::from_bits(data).unwrap(),
            0xffff => {
                info!("IE: {:08b}", data);

                self.ie = Flags::from_bits(data).unwrap();
            }
            _ => return Err(Error::InvalidAddr(address)),
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Emulator};

    #[test]
    fn irq() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff0f, 0b11111).unwrap();
        emu.write(0xffff, 0b11001).unwrap();

        assert_eq!([0b11111, 0b11001], [emu.irq.fi.bits, emu.irq.ie.bits]);
    }
}
