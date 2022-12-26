use crate::error::{Component, ReadError, WriteError};
use byteorder::{ByteOrder, LittleEndian};

pub trait Device {
    /// Read byte from given address.
    /// May return Err if `address` is not mapped to the device.
    fn read(&self, address: u16) -> Result<u8, ReadError>;

    /// Write a byte to given address.
    /// May return Err if `address` is not mapped to the device.
    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError>;

    /// Read a exact amount of bytes, starting at the given address, into the
    /// given output buffer. Device implementors may re-implement this method
    /// for speed.
    fn read_exact(&self, address: u16, buf: &mut [u8]) -> Result<(), ReadError> {
        self.read_exact_fallback(address, buf)
    }

    /// Fallback method for read_exact.
    fn read_exact_fallback(&self, address: u16, buf: &mut [u8]) -> Result<(), ReadError> {
        for (i, out) in buf.iter_mut().enumerate() {
            *out = self.read(address + i as u16)?;
        }
        Ok(())
    }

    /// Write the exact amount of bytes in the slice, starting at the given
    /// address. Device implementors may re-define this method for speed.
    fn write_exact(&mut self, address: u16, buf: &[u8]) -> Result<(), WriteError> {
        self.write_exact_fallback(address, buf)
    }

    /// Fallback method for write_exact.
    fn write_exact_fallback(&mut self, address: u16, buf: &[u8]) -> Result<(), WriteError> {
        for (i, data) in buf.iter().enumerate() {
            self.write(address + i as u16, *data)?;
        }
        Ok(())
    }
}

/// Main device used by the CPU when running the emulation.
///
/// The only purpose of this trait is to provide a method to read LE u16 values
/// as required by some CPU instructions, as well as some basic logging for
/// buggy and/or missing reads and writes.
pub trait MainDevice: Device {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        match <Self as Device>::read(self, address) {
            Ok(b) => Ok(b),
            Err(err) => {
                match err {
                    ReadError::InvalidAddress(_) => log::error!("{err}"),
                    ReadError::AddrNotImpl(_, Some(Component::APU)) => {}
                    ReadError::AddrNotImpl(_, Some(Component::Serial)) => {}
                    _ => log::warn!("{err}"),
                }
                Ok(0xff)
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        match <Self as Device>::write(self, address, data) {
            Ok(_) => Ok(()),
            Err(err) => {
                match err {
                    WriteError::InvalidAddress(_, _) => log::error!("{err}"),
                    WriteError::AddrNotImpl(_, _, Some(Component::APU)) => {}
                    WriteError::AddrNotImpl(_, _, Some(Component::Serial)) => {}
                    _ => log::warn!("{err}"),
                }
                Ok(())
            }
        }
    }

    /// Read little-endian u16 word from given address.
    /// May return Err if `address` or `address + 1` are not mapped to the
    /// device.
    fn read_word(&self, address: u16) -> Result<u16, ReadError> {
        let bytes = [
            <Self as MainDevice>::read(self, address)?,
            <Self as MainDevice>::read(self, address + 1)?,
        ];
        Ok(LittleEndian::read_u16(&bytes[..]))
    }

    /// Write little-endian u16 word from given address.
    /// May return Err if `address` or `address + 1` are not mapped to the
    /// device.
    fn write_word(&mut self, address: u16, data: u16) -> Result<(), WriteError> {
        let mut bytes = [0; 2];
        LittleEndian::write_u16(&mut bytes[..], data);
        <Self as MainDevice>::write(self, address, bytes[0])?;
        <Self as MainDevice>::write(self, address + 1, bytes[1])?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{Device, Result};

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
