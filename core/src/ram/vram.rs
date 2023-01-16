use crate::{
    device::Device,
    error::{ReadError, WriteError},
};
use bitflags::bitflags;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub(crate) struct Attributes: u8 {
        const PALETTE         = 0b00000111;
        const TILE_VRAM_BANK  = 0b00001000;
        const UNUSED          = 0b00010000;
        const HORIZONTAL_FLIP = 0b00100000;
        const VERTICAL_FLIP   = 0b01000000;
        const BG_OAM_PRIPRITY = 0b10000000;
    }
}

impl Attributes {
    #[cfg(feature = "cgb")]
    pub fn palette(&self) -> usize {
        (self.bits & 0b111) as _
    }

    #[cfg(feature = "cgb")]
    pub fn bank(&self) -> usize {
        if self.contains(Attributes::TILE_VRAM_BANK) {
            1
        } else {
            0
        }
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum ColorID {
    C0 = 0xff_000000,
    C1 = 0xff_555555,
    C2 = 0xff_aaaaaa,
    C3 = 0xff_ffffff,
}

#[derive(Debug)]
pub struct TileDataCache {
    #[cfg(not(feature = "cgb"))]
    pub cache: Box<[ColorID; Self::CACHE_BANK_SIZE]>,
    #[cfg(feature = "cgb")]
    pub(crate) cache: Box<[ColorID; Self::CACHE_BANK_SIZE * 2]>,
}

impl TileDataCache {
    const CACHE_TILE_COLS: usize = 24;
    const CACHE_TILE_ROWS: usize = 16;
    const CACHE_TILE_DOTS: usize = 8 * 8;
    const CACHE_BANK_SIZE: usize =
        Self::CACHE_TILE_COLS * Self::CACHE_TILE_ROWS * Self::CACHE_TILE_DOTS;

    /// Number of columns of the cache image.
    pub const DOTS_WIDTH: usize = Self::CACHE_TILE_COLS * 8;
    /// Number of rows of the cache image.
    pub const DOTS_HEIGHT: usize = Self::CACHE_TILE_ROWS * 8;

    fn new() -> Self {
        Self {
            #[cfg(not(feature = "cgb"))]
            cache: Box::new([ColorID::C0; Self::CACHE_BANK_SIZE]),
            #[cfg(feature = "cgb")]
            cache: Box::new([ColorID::C0; Self::CACHE_BANK_SIZE * 2]),
        }
    }

    pub fn as_slice(&self) -> &[ColorID] {
        &self.cache[..]
    }

    /// Compute the index where a given tile's pixel (indexed by bank, tile
    /// index, tile row, and tile column) is stored within the cache.
    ///
    /// Example to compute the location of a particular tile's pixel:
    /// ```
    /// pub fn read_pixel(bank: usize, index: usize, row: u8, col: u8) -> ColorID {
    ///     // ...
    ///     let offset = Self::compute_table_offset(bank, index, row, col);
    ///     cache[offset + col as usize]
    /// }
    /// ```
    pub fn compute_table_offset(bank: usize, index: usize, row: u8, col: u8) -> usize {
        let table_row = (index / Self::CACHE_TILE_COLS) as u32;
        let table_col = (index % Self::CACHE_TILE_COLS) as u32;
        // the number of dots a full row of tiles takes up in the cache
        const TABLE_TILE_ROW_DOTS: usize =
            TileDataCache::CACHE_TILE_DOTS * TileDataCache::CACHE_TILE_COLS;
        let offset = ((TABLE_TILE_ROW_DOTS as u32 * table_row)
            + (8 * (Self::CACHE_TILE_COLS as u32) * (row as u32))
            + (8 * table_col)) as usize;
        #[cfg(not(feature = "cgb"))]
        return offset + (col as usize);
        #[cfg(feature = "cgb")]
        return offset + (col as usize) + (bank * Self::CACHE_BANK_SIZE); // account for VRAM bank
    }

    #[rustfmt::skip]
    fn update_cache(&mut self, address: u16, data: [u8; 3], bank: usize) {
        let offset = (address - 0x8000) as u32;
        let tile_index = (offset / 16) as usize;
        let row = (offset % 16) / 2;
        let (hi, lo) = if (offset % 16) % 2 == 0 { (data[1], data[2]) } else { (data[0], data[1]) };
        let table_offset = Self::compute_table_offset(bank, tile_index, row as u8, 0);
        for col in 0..8 {
            let hi = ((hi as u32) >> (7 - col)) & 1;
            let lo = ((lo as u32) >> (7 - col)) & 1;
            let id = ((hi << 1) | lo) as usize;
            self.cache[table_offset + col] =
                [ColorID::C0, ColorID::C1, ColorID::C2, ColorID::C3][3 - id];
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VRAM {
    // serde doesn't support big arrays so use a boxed slice instead of a boxed big array :(
    data: Box<[u8]>,
    tile_data_cache: TileDataCache,
    bank: usize,
}

impl Default for VRAM {
    fn default() -> Self {
        Self {
            data: vec![0; 0x2000 * 2].into_boxed_slice(),
            tile_data_cache: TileDataCache::new(),
            bank: 0,
        }
    }
}

impl VRAM {
    pub fn tile_data_cache(&self) -> &TileDataCache {
        &self.tile_data_cache
    }

    pub(crate) fn data(&self, bank: usize, address: u16) -> u8 {
        let offset = 0x2000 * bank;
        self.data[offset + (address as usize) - 0x8000]
    }

    #[cfg(feature = "cgb")]
    pub(crate) fn attributes(&self, address: u16) -> Attributes {
        Attributes::from_bits(self.data(1, address)).unwrap()
    }

    fn bank_address(&self, address: u16) -> usize {
        self.bank * 0x2000 + (address as usize) - 0x8000
    }
}

impl Device for VRAM {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0x8000..=0x9fff => Ok(self.data[self.bank_address(address)]),
                0xff4f => {
                    let bank = self.bank as u8;
                    // Reading from this register will return the number of the currently loaded
                    // VRAM bank in bit 0, and all other bits will be set to 1.
                    Ok(0xfe | bank)
                }
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0x8000..=0x9fff => {
                    if let 0x8000..=0x97ff = address {
                        let prev = if address > 0x8000 { self.read(address-1).unwrap() } else { 0x00 };
                        let next = if address < 0x97ff { self.read(address+1).unwrap() } else { 0x00 };
                        self.tile_data_cache.update_cache(address, [prev, data, next], self.bank);
                    }
                    self.data[self.bank_address(address)] = data;
                },
                0xff4f => {
                    self.bank = (data & 1) as _;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {}
