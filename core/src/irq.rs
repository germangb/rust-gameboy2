use crate::{device::Device, error::Error};
use log::info;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IRQ {
    fi: u8,
    ie: u8,
}

impl Device for IRQ {
    const DEBUG_NAME: &'static str = "IRQ";

    fn read(&self, address: u16) -> Result<u8, Error> {
        match address {
            0xff0f => Ok(self.fi),
            0xffff => Ok(self.ie),
            _ => Err(Error::InvalidAddr(address)),
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), Error> {
        match address {
            0xff0f => self.fi = data,
            0xffff => {
                info!("IE = {:#08b}", data);

                self.ie = data
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

        emu.write(0xff0f, 0x12).unwrap();
        emu.write(0xffff, 0xbc).unwrap();

        assert_eq!([0x12, 0xbc], [emu.irq.fi, emu.irq.ie]);
    }
}
