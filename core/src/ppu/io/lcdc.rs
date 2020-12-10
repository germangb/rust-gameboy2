use crate::dev::{invalid_read, invalid_write, Device};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LCDC {
    lcdc: u8,
}

impl LCDC {
    // Returns the address where the window map.
    // According to LCDC.6
    pub fn window_map_select(&self) -> u16 {
        todo!()
    }

    // Returns the address where the BG & Window data.
    // According to LCDC.4
    pub fn bg_window_data_select(&self) -> u16 {
        todo!()
    }

    // Returns the address where the window map.
    // According to LCDC.3
    pub fn bg_map_select(&self) -> u16 {
        todo!()
    }
}

impl Device for LCDC {
    fn debug_name() -> &'static str {
        "LCDC"
    }

    fn read(&self, address: u16) -> u8 {
        if address != 0xff40 {
            invalid_read(address);
        }

        self.lcdc
    }

    fn write(&mut self, address: u16, data: u8) {
        if address != 0xff40 {
            invalid_write(address);
        }

        self.lcdc = data
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn lcdc() {
        todo!()
    }
}
