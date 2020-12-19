use crate::device::{Device, Result};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WorkRAM {
    // serde doesn't support big arrays so use a boxed slice instead of a boxed big array :(
    data: Box<[u8]>,
    bank: usize,
}

impl Default for WorkRAM {
    fn default() -> Self {
        Self {
            data: vec![0; 0x1000 * 8].into_boxed_slice(),
            bank: 0,
        }
    }
}

impl WorkRAM {
    fn bank_address(&self, address: u16) -> usize {
        let bank = self.bank.max(1);
        bank * 0x1000 + (address as usize) - 0xd000
    }
}

impl Device for WorkRAM {
    fn read(&self, address: u16) -> Result<u8> {
        device_match! {
            address {
                0xc000..=0xcfff => Ok(self.data[(address as usize) - 0xc000]),
                0xd000..=0xdfff => Ok(self.data[self.bank_address(address)]),
                0xff70 => {
                    Ok((self.bank & 0b111) as _)
                }
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        device_match! {
            address {
                0xc000..=0xcfff => self.data[(address as usize) - 0xc000] = data,
                0xd000..=0xdfff => self.data[self.bank_address(address)] = data,
                0xff70 => {
                    self.bank = (data & 0b111) as _
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Emulator};

    #[test]
    fn work_ram() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xc000, 1).unwrap();
        emu.write(0xdfff, 2).unwrap();

        assert_eq!(
            [1, 2],
            [
                emu.work_ram.read(0xc000).unwrap(),
                emu.work_ram.read(0xdfff).unwrap()
            ]
        );
    }

    #[test]
    fn work_ram_bank() {
        let mut emu = Emulator::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xc000, 0x1).unwrap();
        emu.write(0xcfff, 0x2).unwrap();

        emu.write(0xff70, 1).unwrap();
        emu.write(0xd000, 0xa).unwrap();
        emu.write(0xdfff, 0xb).unwrap();
        emu.write(0xff70, 2).unwrap();
        emu.write(0xd000, 0xc).unwrap();
        emu.write(0xdfff, 0xd).unwrap();
        emu.write(0xff70, 3).unwrap();
        emu.write(0xd000, 0xe).unwrap();
        emu.write(0xdfff, 0xf).unwrap();

        emu.write(0xff70, 0).unwrap();
        states.push(emu.read(0xd000).unwrap());
        states.push(emu.read(0xdfff).unwrap());
        emu.write(0xff70, 1).unwrap();
        states.push(emu.read(0xd000).unwrap());
        states.push(emu.read(0xdfff).unwrap());
        emu.write(0xff70, 2).unwrap();
        states.push(emu.read(0xd000).unwrap());
        states.push(emu.read(0xdfff).unwrap());
        emu.write(0xff70, 3).unwrap();
        states.push(emu.read(0xd000).unwrap());
        states.push(emu.read(0xdfff).unwrap());

        assert_eq!(vec![0xa, 0xb, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf], states)
    }
}
