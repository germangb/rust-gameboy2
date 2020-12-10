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

use crate::{
    boot::Boot,
    cartridge::Cartridge,
    cpu::CPU,
    dev::{Address, Device, LogDevice},
    dma::OamDma,
    high_ram::HighRAM,
    joypad::{Button, Joypad},
    ppu::{lcd::LcdBuffer, PPU},
    timer::Timer,
    work_ram::WorkRAM,
};
pub use gb::GameBoy;
pub use gbc::GameBoyColor;
use log::{error, warn};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cell::Cell;

mod boot;
pub mod cartridge;
pub mod cpu;
pub mod dev;
mod dma;
mod gb;
mod gbc;
mod high_ram;
pub mod joypad;
pub mod ppu;
mod timer;
mod utils;
mod work_ram;

const CLOCK: u64 = 4_194_304;

struct EmulationStep {
    /// Number of elapsed ticks.
    /// Driven by main clock (4194304Hz)
    pub clock_ticks: u64,
}

trait Update {
    fn update(&mut self, step: &EmulationStep);
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Emulator<C> {
    running: Cell<bool>,
    cpu: CPU,
    cartridge: LogDevice<C>,
    boot: LogDevice<Boot>,
    oam_dma: LogDevice<OamDma>,
    joypad: LogDevice<Joypad>,
    ppu: LogDevice<PPU>,
    timer: LogDevice<Timer>,
    work_ram: LogDevice<WorkRAM>,
    high_ram: LogDevice<HighRAM>,
}

impl<C> Emulator<C> {
    fn new(cartridge: C) -> Self {
        Self {
            running: Cell::new(true),
            cpu: Default::default(),
            cartridge: LogDevice(cartridge),
            boot: Default::default(),
            oam_dma: Default::default(),
            joypad: Default::default(),
            ppu: Default::default(),
            timer: Default::default(),
            work_ram: Default::default(),
            high_ram: Default::default(),
        }
    }
}

impl<C: Cartridge> Emulator<C> {
    fn update(&mut self) {
        if self.running.get() {
            let step = todo!();

            self.oam_dma.update(&step);
            self.ppu.update(&step);
            self.timer.update(&step);
        }
    }

    fn stop_running(&self) {
        self.running.set(false);
    }

    #[rustfmt::skip]
    fn read_io(&self, address: Address) -> u8 {
        match address {
            0xff00..=0xff02 => todo!("Port/Mode"),
            0xff04..=0xff07 => todo!("Port/Mode"),
            0xff10..=0xff26 => todo!("Sound"),
            0xff30..=0xff3f => todo!("waveform RAM"),
            0xff40..=0xff45 => self.ppu.read(address),
            0xff46          => self.oam_dma.read(address),
            0xff47..=0xff4b => self.ppu.read(address),
            0xff4f          => todo!("Game Boy Color (VRAM Bank Select)"),
            0xff50          => self.boot.read(address),
            0xff51..=0xff55 => todo!("Game Boy color"),
            0xff68..=0xff6a => todo!("Game Boy color (DMA)"),
            _               => todo!("IO register I have missed ({:#04x})!", address),
        }
    }

    #[rustfmt::skip]
    fn write_io(&mut self, address: Address, data: u8) {
        match address {
            0xff00..=0xff02 => todo!("Port/Mode"),
            0xff04..=0xff07 => todo!("Port/Mode"),
            0xff10..=0xff26 => todo!("Sound"),
            0xff30..=0xff3f => todo!("waveform RAM"),
            0xff40..=0xff45 => self.ppu.write(address, data),
            0xff46          => {
                self.oam_dma.write(address, data);

                // do the OAM transfer all at once, then the emulator will block certain
                // locations in memory until the corresponding number of cycles has elapsed.
                if self.oam_dma.is_active() {
                    self.oam_dma_transfer();
                }
            }
            0xff47..=0xff4b => self.ppu.write(address, data),
            0xff4f          => todo!("Game Boy Color (VRAM Bank Select)"),
            0xff50          => self.boot.write(address, data),
            0xff51..=0xff55 => todo!("Game Boy color"),
            0xff68..=0xff6a => todo!("Game Boy color (DMA)"),
            _               => todo!("IO register I have missed ({:#04x})!", address),
        }
    }

    fn oam_dma_transfer(&mut self) {
        let src = self.oam_dma.start_address();

        let src = src..=src | 0x9f;
        let dst = 0xfe00..0xfe9f;

        for (src, dst) in src.zip(dst) {
            let data = self.read(src);
            self.write(dst, data);
        }
    }
}

impl<C: Cartridge> Device for Emulator<C> {
    fn read(&self, address: u16) -> u8 {
        let oam_dma = self.oam_dma.is_active();
        let boot = self.boot.is_enabled();

        match address {
            0x0000..=0x00ff if boot => self.boot.read(address),
            0x0000..=0x7fff if !oam_dma => self.cartridge.read(address),
            0x8000..=0x9fff if !oam_dma => self.ppu.read(address),
            0xa000..=0xbfff if !oam_dma => self.cartridge.read(address),
            0xc000..=0xdfff if !oam_dma => todo!("Work Ram"),
            // Nintendo says use of this area is prohibited.
            0xe000..=0xfdff => {
                warn!("Echo RAM");
                warn!("Nintendo says use of this area is prohibited");
                self.read(address - 0xe000 + 0xc000)
            }
            0xfe00..=0xfe9f if !oam_dma => self.ppu.read(address),
            0xfea0..=0xfeff if !oam_dma => {
                // emulate behavior depending on device & hardware revision
                // https://gbdev.io/pandocs/#fea0-feff-range
                todo!();

                error!("Memory region not in use");
                error!("Nintendo says use of this area is prohibited!");
                self.stop_running();
                0x00
            }
            0xfea0..=0xfeff => 0xff,
            0xff00..=0xff7f if !oam_dma => self.read_io(address),
            0xff80..=0xfffe => self.high_ram.read(address),
            0xffff..=0xffff if !oam_dma => todo!("Interrupts"),

            // illegal access during OAM DMA transfer
            // during this process, the GB is only allowed to access HRAM
            _ if oam_dma => {
                error!("Illegal address accessed: {:#04x}", address);
                error!("OAM Transfer in still in progress!");
                self.stop_running();
                0xff
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        let oam_dma = self.oam_dma.is_active();
        let boot = self.boot.is_enabled();

        // FIXME code repetition...
        match address {
            0x0000..=0x00ff if boot => self.boot.write(address, data),
            0x0000..=0x7fff if !oam_dma => self.cartridge.write(address, data),
            0x8000..=0x9fff if !oam_dma => self.ppu.write(address, data),
            0xa000..=0xbfff if !oam_dma => self.cartridge.write(address, data),
            0xc000..=0xdfff if !oam_dma => self.work_ram.write(address, data),
            // Nintendo says use of this area is prohibited.
            0xe000..=0xfdff => {
                error!("Echo RAM");
                error!("Nintendo says use of this area is prohibited");
                self.stop_running();
            }
            0xfe00..=0xfe9f if !oam_dma => self.ppu.write(address, data),
            0xfea0..=0xfeff => {
                // emulate behavior depending on device & hardware revision
                // https://gbdev.io/pandocs/#fea0-feff-range
                todo!();

                error!("Memory region not in use");
                error!("Nintendo says use of this area is prohibited!");
                self.stop_running();
            }
            0xff00..=0xff7f if !oam_dma => self.write_io(address, data),
            0xff80..=0xfffe => self.high_ram.write(address, data),
            0xffff..=0xffff if !oam_dma => todo!("Interrupts"),

            // illegal access during OAM DMA transfer
            // during this process, the GB is only allowed to access HRAM
            _ if oam_dma => {
                error!("Illegal address accessed: {:#04x}", address);
                error!("OAM Transfer in still in progress!");
                self.stop_running();
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::Cartridge, dev::Device, Emulator};

    #[test]
    fn oam_dma() {
        todo!();
    }
}
