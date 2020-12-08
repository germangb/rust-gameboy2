use byteorder::{ByteOrder, LittleEndian};

pub type Address = u16;

/// Memory-mapped device you can read & write bytes from and to.
pub trait Device {
    /// Read byte
    fn read(&self, address: Address) -> u8;

    /// Write byte.
    fn write(&mut self, address: Address, data: u8);

    /// Read a word.
    fn read_word(&self, address: Address) -> u16 {
        let bytes = [self.read(address), self.read(address + 1)];
        LittleEndian::read_u16(&bytes[..])
    }

    /// Write a word.
    fn write_word(&mut self, address: Address, data: u16) {
        let mut bytes = [0; 2];
        LittleEndian::write_u16(&mut &mut bytes[..], data);
        self.write(address, bytes[0]);
        self.write(address + 1, bytes[1]);
    }
}

#[cfg(test)]
mod test {
    use super::Device;

    type TestDevice = Box<[u8; 0x10000]>;

    impl Device for TestDevice {
        fn read(&self, address: u16) -> u8 {
            self[address as usize]
        }

        fn write(&mut self, address: u16, data: u8) {
            self[address as usize] = data
        }
    }

    fn test_device() -> TestDevice {
        Box::new([0; 0x10000])
    }

    #[test]
    fn read_write_word() {
        let mut device = test_device();

        device.write_word(0x0000, 0x1234);
        device.write_word(0x0100, 0xabcd);

        assert_eq!(0x1234, device.read_word(0x0000));
        assert_eq!(0xabcd, device.read_word(0x0100));
    }
}
