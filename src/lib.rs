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

pub use crate::joypad::Button;
use crate::{
    cartridge::Cartridge,
    dev::{Address, Device, LogDevice},
    dma::OamDma,
    joypad::Joypad,
    ppu::{lcd::LcdBuffer, Ppu},
    timer::Timer,
};
use log::{error, warn};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cell::Cell;

pub mod cartridge;
pub mod cpu;
pub mod dev;
mod dma;
mod high_ram;
mod joypad;
mod ppu;
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

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Emulator<C> {
    running: Cell<bool>,
    cartridge: LogDevice<C>,
    oam_dma: LogDevice<OamDma>,
    joypad: LogDevice<Joypad>,
    ppu: LogDevice<Ppu>,
    timer: LogDevice<Timer>,
}

impl<C> Emulator<C> {
    fn new(cartridge: C) -> Self {
        todo!()
    }

    fn display(&self) -> &LcdBuffer {
        self.ppu.buffer()
    }

    fn stop_running(&self) {
        self.running.set(false);
    }

    fn read_io(&self, address: Address) -> u8 {
        match address {
            0xff47 => self.oam_dma.read(address),
            _ => todo!("io"),
        }
    }

    fn write_io(&mut self, address: Address, data: u8) {
        match address {
            0xff47 => {
                self.oam_dma.write(address, data);
                if self.oam_dma.is_active() {
                    let start = self.oam_dma.start_address();
                    self.ppu.oam_dma_transfer(start);
                }
            }
            _ => todo!("io"),
        }
    }
}

impl<C: Cartridge> Device for Emulator<C> {
    fn read(&self, address: u16) -> u8 {
        let oam_dma = self.oam_dma.is_active();

        match address {
            0x0000..=0x7fff if !oam_dma => self.cartridge.read(address),
            0x8000..=0x9fff if !oam_dma => self.ppu.read(address),
            0xa000..=0xbfff if !oam_dma => self.cartridge.read(address),
            0xc000..=0xdfff if !oam_dma => todo!("work ram"),
            // Nintendo says use of this area is prohibited.
            0xe000..=0xfdff => {
                warn!("Echo RAM");
                warn!("Nintendo says use of this area is prohibited");
                self.read(address - 0xe000 + 0xc000)
            }
            0xfe00..=0xfe9f if !oam_dma => self.ppu.read(address),
            0xfea0..=0xfeff => {
                error!("Memory region not in use");
                error!("Nintendo says use of this area is prohibited!");
                self.stop_running();
                0
            }
            0xff00..=0xff7f if !oam_dma => self.read_io(address),
            0xff80..=0xfffe => todo!("high ram"),
            0xffff..=0xffff if !oam_dma => todo!("interrupts"),

            // illegal access during OAM DMA transfer
            // during this process, the GB is only allowed to access HRAM
            _ if oam_dma => {
                error!("Illegal address accessed: {:#04x}", address);
                error!("OAM Transfer in still in progress!");
                self.stop_running();
                0
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        let oam_dma = self.oam_dma.is_active();

        // FIXME code repetition...
        match address {
            0x0000..=0x7fff if !oam_dma => self.cartridge.write(address, data),
            0x8000..=0x9fff if !oam_dma => self.ppu.write(address, data),
            0xa000..=0xbfff if !oam_dma => self.cartridge.write(address, data),
            0xc000..=0xdfff if !oam_dma => todo!("work ram"),
            // Nintendo says use of this area is prohibited.
            0xe000..=0xfdff => {
                error!("Echo RAM");
                error!("Nintendo says use of this area is prohibited");
                self.stop_running();
            }
            0xfe00..=0xfe9f if !oam_dma => self.ppu.write(address, data),
            0xfea0..=0xfeff => {
                error!("Memory region not in use");
                error!("Nintendo says use of this area is prohibited!");
                self.stop_running();
            }
            0xff00..=0xff7f if !oam_dma => self.write_io(address, data),
            0xff80..=0xfffe => todo!("high ram"),
            0xffff..=0xffff if !oam_dma => todo!("interrupts"),

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

pub struct GameBoy;
pub struct GameBoyColor;
