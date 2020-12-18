use crate::error::Error;
use byteorder::{ByteOrder, LittleEndian};

type Endianness = LittleEndian;

pub type Result<T> = std::result::Result<T, Error>;

pub trait Device {
    fn read(&self, address: u16) -> Result<u8>;

    fn write(&mut self, address: u16, data: u8) -> Result<()>;

    fn read_word(&self, address: u16) -> Result<u16> {
        let bytes = [self.read(address)?, self.read(address + 1)?];
        Ok(Endianness::read_u16(&bytes[..]))
    }

    fn write_word(&mut self, address: u16, data: u16) -> Result<()> {
        let mut bytes = [0; 2];
        Endianness::write_u16(&mut &mut bytes[..], data);
        self.write(address, bytes[0])?;
        self.write(address + 1, bytes[1])?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{Device, Result};
    use crate::error::Error;

    type TestDevice = Box<[u8; 0x10000]>;

    impl Device for TestDevice {
        fn read(&self, address: u16) -> Result<u8> {
            Ok(self[address as usize])
        }

        fn write(&mut self, address: u16, data: u8) -> Result<()> {
            self[address as usize] = data;

            Ok(())
        }
    }

    fn test_device() -> TestDevice {
        Box::new([0; 0x10000])
    }

    #[test]
    fn read_write_word() {
        let mut device = test_device();

        device.write_word(0x0000, 0x1234).unwrap();
        device.write_word(0x0100, 0xabcd).unwrap();

        assert_eq!(0x1234, device.read_word(0x0000).unwrap());
        assert_eq!(0xabcd, device.read_word(0x0100).unwrap());
    }
}
