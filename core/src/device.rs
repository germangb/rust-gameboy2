use crate::error::Error;
use byteorder::{ByteOrder, LittleEndian};

type Endianness = LittleEndian;

pub trait Device {
    const DEBUG_NAME: &'static str;

    fn read(&self, address: u16) -> Result<u8, Error>;

    fn write(&mut self, address: u16, data: u8) -> Result<(), Error>;

    fn read_word(&self, address: u16) -> Result<u16, Error> {
        let bytes = [self.read(address)?, self.read(address + 1)?];
        Ok(Endianness::read_u16(&bytes[..]))
    }

    fn write_word(&mut self, address: u16, data: u16) -> Result<(), Error> {
        let mut bytes = [0; 2];
        Endianness::write_u16(&mut &mut bytes[..], data);
        self.write(address, bytes[0])?;
        self.write(address + 1, bytes[1])?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Device;
    use crate::error::Error;

    type TestDevice = Box<[u8; 0x10000]>;

    impl Device for TestDevice {
        const DEBUG_NAME: &'static str = "Test";

        fn read(&self, address: u16) -> Result<u8, Error> {
            Ok(self[address as usize])
        }

        fn write(&mut self, address: u16, data: u8) -> Result<(), Error> {
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
