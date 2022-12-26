use crate::{
    device::Device,
    error::{ReadError, WriteError},
};
use bitflags::bitflags;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Attributes: u8 {
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
pub enum TilePixel {
    C0 = 0xff_000000,
    C1 = 0xff_555555,
    C2 = 0xff_aaaaaa,
    C3 = 0xff_ffffff,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VRAM {
    // serde doesn't support big arrays so use a boxed slice instead of a boxed big array :(
    data: Box<[u8]>,
    bank: usize,
    // Tile data corresponding to the region:
    // 0x8000..=0x97ff
    // represented as a 16x24 image
    #[cfg(not(feature = "cgb"))]
    cache_tiles: Box<[TilePixel; 24 * 16 * 8 * 8]>,
    #[cfg(feature = "cgb")]
    cache_tiles: Box<[TilePixel; 24 * 16 * 8 * 8 * 2]>,
}

impl Default for VRAM {
    fn default() -> Self {
        Self {
            data: vec![0; 0x2000 * 2].into_boxed_slice(),
            bank: 0,
            #[cfg(not(feature = "cgb"))]
            cache_tiles: Box::new([TilePixel::C1; 24 * 16 * 8 * 8]),
            #[cfg(feature = "cgb")]
            cache_tiles: Box::new([TilePixel::C1; 24 * 16 * 8 * 8 * 2]),
        }
    }
}

impl VRAM {
    pub fn tile_data(&self) -> &[u8] {
        unsafe {
            let len = std::mem::size_of::<TilePixel>() * self.cache_tiles.len();
            std::slice::from_raw_parts(self.cache_tiles.as_ptr() as *const u8, len)
        }
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
                    // update tile cache
                    if let 0x8000..=0x97ff = address {
                        let tile_address_offset = (address - 0x8000) as u64;
                        let tile_index = tile_address_offset / 16;
                        let tile_row = (tile_address_offset % 16) / 2;
                        let (hi, lo) = if (tile_address_offset % 16) % 2 == 0 {
                            (data, self.read(address + 1).unwrap())
                        } else {
                            (self.read(address - 1).unwrap(), data)
                        };

                        let table_row = tile_index / 24;
                        let table_col = tile_index % 24;
                        let mut cache_tiles_offset = ((8 * 24 * 8 * table_row) + (8 * 24 * tile_row) + (8 * table_col)) as usize;
                        #[cfg(feature = "cgb")]
                        {
                            cache_tiles_offset += self.cache_tiles.len()/2 * self.bank;
                        }

                        for tile_col in 0..8 {
                            let hi = (hi >> ((7-tile_col) as u8)) & 1;
                            let lo = (lo >> ((7-tile_col) as u8)) & 1;
                            match (hi << 1) | lo {
                                0 => self.cache_tiles[cache_tiles_offset + tile_col] = TilePixel::C0,
                                1 => self.cache_tiles[cache_tiles_offset + tile_col] = TilePixel::C1,
                                2 => self.cache_tiles[cache_tiles_offset + tile_col] = TilePixel::C2,
                                3 => self.cache_tiles[cache_tiles_offset + tile_col] = TilePixel::C3,
                                _ => unreachable!(),
                            }
                        }
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
mod test {
    use crate::{cartridge::NoCartridge, device::Device, LR35902};

    #[test]
    fn video_ram() {
        let mut emu = LR35902::new(NoCartridge);

        emu.write(0x8000, 1).unwrap();
        emu.write(0x9fff, 2).unwrap();

        assert_eq!(
            [1, 2],
            [emu.read(0x8000).unwrap(), emu.read(0x9fff).unwrap()]
        );
    }

    #[test]
    fn video_ram_bank() {
        let mut emu = LR35902::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0x8000, 0xa).unwrap();
        emu.write(0x9fff, 0xb).unwrap();
        emu.write(0xff4f, 1).unwrap();
        emu.write(0x8000, 0xc).unwrap();
        emu.write(0x9fff, 0xd).unwrap();
        emu.write(0xff4f, 0).unwrap();

        states.push(emu.read(0x8000).unwrap());
        states.push(emu.read(0x9fff).unwrap());
        emu.write(0xff4f, 1).unwrap();
        states.push(emu.read(0x8000).unwrap());
        states.push(emu.read(0x9fff).unwrap());

        assert_eq!(vec![0xa, 0xb, 0xc, 0xd], states)
    }

    #[test]
    fn video_ram_attributes() {
        todo!()
    }
}
