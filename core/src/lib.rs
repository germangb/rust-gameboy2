#![warn(
    clippy::all,
    clippy::doc_markdown,
    clippy::dbg_macro,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::pub_enum_variant_names,
    clippy::mem_forget,
    clippy::use_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    unused,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style
)]

#[macro_export]
macro_rules! device_match {
    ($address:ident { $($tt:tt)* }) => {
        match $address {
            $($tt)*
            _ => return Err($crate::error::Error::InvalidAddr($address))
        }
    }
}

use crate::{
    apu::APU,
    boot::Boot,
    cartridge::Cartridge,
    cpu::CPU,
    device::{Device, Result},
    dma::DMA,
    irq::IRQ,
    joypad::Joypad,
    ppu::PPU,
    ram::{HighRAM, WorkRAM},
    timer::Timer,
};
use log::{info, warn};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// re-exports
use crate::dma::HDMA;
pub use gb::GameBoy;
pub use joypad::Button;
pub use ppu::lcd;

mod apu;
mod boot;
pub mod cartridge;
pub mod cpu;
pub mod device;
mod dma;
pub mod error;
mod gb;
mod irq;
mod joypad;
mod ppu;
mod ram;
mod timer;
mod utils;

const CLOCK: u64 = 4_194_304;

trait Update {
    fn update(&mut self, ticks: u64, flags: &mut irq::Flags);
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Emulator<C> {
    // borrow checker workaround
    // cpu will be leaving the Option temporarily
    cpu: Option<CPU>,
    cartridge: C,
    boot: Boot,
    dma: DMA,
    hdma: HDMA,
    joypad: Joypad,
    ppu: PPU,
    timer: Timer,
    work_ram: WorkRAM,
    high_ram: HighRAM,
    irq: IRQ,
    apu: APU,
}

impl<C> Emulator<C> {
    fn new(cartridge: C) -> Self {
        Self {
            cpu: Some(Default::default()),
            cartridge,
            boot: Default::default(),
            dma: Default::default(),
            hdma: Default::default(),
            joypad: Default::default(),
            ppu: Default::default(),
            timer: Default::default(),
            work_ram: Default::default(),
            high_ram: Default::default(),
            irq: Default::default(),
            apu: Default::default(),
        }
    }
}

impl<C: Cartridge> Emulator<C> {
    fn set_debug_overlay(&mut self, b: bool) {
        self.ppu.set_debug_overlays(b);
    }

    fn update(&mut self) -> Result<()> {
        let mut cpu = self.cpu.take().unwrap();
        let ticks = cpu.update(self)?;
        self.cpu = Some(cpu);
        let mut flags = irq::Flags::default();

        self.dma.update(ticks, &mut flags);
        self.ppu.update(ticks, &mut flags);
        self.timer.update(ticks, &mut flags);

        // request IF register with requested interrupts.
        self.irq.fi |= flags;
        Ok(())
    }

    fn update_frame(&mut self) -> Result<()> {
        // run until VBLANK
        while self.read(0xff41)? & 0b11 != 0b01 {
            self.update()?;
        }

        self.ppu.finish();

        // run until next OAM search
        while self.read(0xff41)? & 0b11 != 0b10 {
            self.update()?;
        }

        Ok(())
    }

    fn read_io(&self, address: u16) -> Result<u8> {
        match address {
            0xff00 => self.joypad.read(address),
            0xff01..=0xff02 => {
                warn!("Port/Mode not implemented {:04x}", address);
                Ok(0)
            }
            0xff04..=0xff07 => self.timer.read(address),
            0xff0f => self.irq.read(address),
            0xff10..=0xff26 => self.apu.read(address),
            0xff30..=0xff3f => self.apu.read(address),
            0xff40..=0xff45 => self.ppu.read(address),
            0xff46 => self.dma.read(address),
            0xff47..=0xff4b => self.ppu.read(address),
            0xff4f => self.ppu.read(address),
            0xff50 => self.boot.read(address),
            0xff51..=0xff54 => self.hdma.read(address),
            0xff55 => Ok(0xff),
            0xff68..=0xff6b => self.ppu.read(address),
            0xff70 => self.work_ram.read(address),
            _ => {
                warn!("Unknown IO address: {:04x}", address);
                Ok(0xff)
            }
        }
    }

    fn write_io(&mut self, address: u16, data: u8) -> Result<()> {
        match address {
            0xff00 => self.joypad.write(address, data),
            0xff01..=0xff02 => {
                warn!("Port/Mode not implemented: {:04x}, {:02x}", address, data);
                Ok(())
            }
            0xff04..=0xff07 => self.timer.write(address, data),
            0xff0f => self.irq.write(address, data),
            0xff10..=0xff26 => self.apu.write(address, data),
            0xff30..=0xff3f => self.apu.write(address, data),
            0xff40..=0xff45 => self.ppu.write(address, data),
            0xff46 => {
                self.dma.write(address, data)?;

                // do the OAM transfer all at once, then the emulator will block certain
                // locations in memory until the corresponding number of cycles has elapsed.
                if self.dma.is_active() {
                    self.dma()?;
                }

                Ok(())
            }
            0xff47..=0xff4b => self.ppu.write(address, data),
            0xff4f => self.ppu.write(address, data),
            0xff50 => self.boot.write(address, data),
            0xff51..=0xff54 => self.hdma.write(address, data),
            0xff55 => self.hdma(data),
            0xff68..=0xff6b => self.ppu.write(address, data),
            0xff70 => self.work_ram.write(address, data),
            _ => {
                warn!("Unknown IO address: {:04x}, data: {:02x}", address, data);
                Ok(())
            }
        }
    }

    fn dma(&mut self) -> Result<()> {
        let src = self.dma.start_address();

        let src = src..=src | 0x9f;
        let dst = 0xfe00..0xfe9f;

        for (src, dst) in src.zip(dst) {
            let data = self.read(src)?;
            self.write(dst, data)?;
        }
        Ok(())
    }

    fn hdma(&mut self, hdma5: u8) -> Result<()> {
        let hdma1 = self.hdma.hdma1 as u16;
        let hdma2 = self.hdma.hdma2 as u16;
        let hdma3 = self.hdma.hdma3 as u16;
        let hdma4 = self.hdma.hdma4 as u16;
        let hdma5 = hdma5 as u16;

        let source = (hdma1 << 8) | (hdma2 & 0xf0);
        let destination = 0x8000 | ((hdma3 & 0x1f) << 8) | (hdma4 & 0xf0);
        let len = ((hdma5 & 0x7f) + 1) * 16;

        let src = source..source + len;
        let dst = destination..destination + len;
        for (src, dst) in src.zip(dst) {
            let data = self.read(src)?;
            self.write(dst, data)?;
        }

        Ok(())
    }
}

impl<C: Cartridge> Device for Emulator<C> {
    fn read(&self, address: u16) -> Result<u8> {
        let boot = self.boot.is_enabled();

        device_match! {
            address {
                #[cfg(not(feature = "cgb"))]
                0x0000..=0x00ff if boot => self.boot.read(address),
                #[cfg(feature = "cgb")]
                0x0000..=0x00ff | 0x0150..=0x0900 if boot => self.boot.read(address),
                0x0000..=0x7fff => self.cartridge.read(address),
                0x8000..=0x9fff => self.ppu.read(address),
                0xa000..=0xbfff => self.cartridge.read(address),
                0xc000..=0xdfff => self.work_ram.read(address),
                // Nintendo says use of this area is prohibited.
                0xe000..=0xfdff => {
                    warn!("Nintendo says use of this area is prohibited (Echo RAM).");

                    self.read(address - 0xe000 + 0xc000)
                }
                0xfe00..=0xfe9f => self.ppu.read(address),

                // TODO emulate behavior depending on device & hardware revision
                //  https://gbdev.io/pandocs/#fea0-feff-range
                0xfea0..=0xfeff => {
                    warn!("Nintendo says use of this area is prohibited (Unused).");

                    Ok(0xff)
                }

                0xff00..=0xff7f => self.read_io(address),
                0xff80..=0xfffe => self.high_ram.read(address),
                0xffff => self.irq.read(address),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        let boot = self.boot.is_enabled();

        device_match! {
            address {
                #[cfg(not(feature = "cgb"))]
                0x0000..=0x00ff if boot => self.boot.write(address, data),
                #[cfg(feature = "cgb")]
                0x0000..=0x00ff | 0x0150..=0x0900 if boot => self.boot.write(address, data),
                0x0000..=0x7fff => self.cartridge.write(address, data),
                0x8000..=0x9fff => self.ppu.write(address, data),
                0xa000..=0xbfff => self.cartridge.write(address, data),
                0xc000..=0xdfff => self.work_ram.write(address, data),
                // Nintendo says use of this area is prohibited.
                0xe000..=0xfdff => {
                    warn!("Nintendo says use of this area is prohibited (Echo RAM).");
                    Ok(())
                }
                0xfe00..=0xfe9f => self.ppu.write(address, data),

                // TODO emulate behavior depending on device & hardware revision
                //  https://gbdev.io/pandocs/#fea0-feff-range
                0xfea0..=0xfeff => {
                    warn!("Nintendo says use of this area is prohibited (Unused).");

                    Ok(())
                }

                0xff00..=0xff7f => self.write_io(address, data),
                0xff80..=0xfffe => self.high_ram.write(address, data),
                0xffff => self.irq.write(address, data),
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn oam_dma() {
        todo!();
    }
}
