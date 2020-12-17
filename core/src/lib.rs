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
    apu::APU,
    boot::Boot,
    cartridge::Cartridge,
    cpu::CPU,
    device::{Address, Device},
    dma::DMA,
    irq::IRQ,
    joypad::Joypad,
    ppu::PPU,
    ram::{HighRAM, WorkRAM},
    timer::Timer,
};
use log::{error, info, warn};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cell::Cell;

// re-exports
use crate::error::Error;
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

#[derive(Default)]
pub struct Request {
    pub vblank: bool,
    pub stat: bool,
    pub joypad: bool,
    pub serial: bool,
    pub timer: bool,
}

trait Update {
    fn update(&mut self, ticks: u64, request: &mut Request);
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Emulator<C> {
    running: Cell<bool>,
    // borrow checker workaround
    // cpu will be leaving the Option temporarily
    cpu: Option<CPU>,
    cartridge: C,
    boot: Boot,
    oam_dma: DMA,
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
            running: Cell::new(true),
            cpu: Some(Default::default()),
            cartridge,
            boot: Default::default(),
            oam_dma: Default::default(),
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

    fn update(&mut self) -> Result<(), Error> {
        if self.running.get() {
            let mut cpu = self.cpu.take().unwrap();
            let ticks = cpu.update(self)?;
            self.cpu = Some(cpu);
            let mut request = Request::default();

            self.oam_dma.update(ticks, &mut request);
            self.ppu.update(ticks, &mut request);
            self.timer.update(ticks, &mut request);

            // request IF register with requested interrupts.
            self.update_irq(&request)?;
            Ok(())
        } else {
            panic!("Emulator is no longer running")
        }
    }

    fn update_frame(&mut self) -> Result<(), Error> {
        if self.running.get() {
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
        } else {
            panic!("Emulator is no longer running")
        }
    }

    fn stop_running(&self) {
        self.running.set(false);
    }

    #[rustfmt::skip]
    fn read_io(&self, address: Address) -> Result<u8, Error> {
        match address {
            0xff00          => self.joypad.read(address),
            0xff01..=0xff02 => {
                warn!("Port/Mode not implemented {:04x}", address);
                Ok(0)
            },
            0xff04..=0xff07 => self.timer.read(address),
            0xff0f          => self.irq.read(address),
            0xff10..=0xff26 => self.apu.read(address),
            0xff30..=0xff3f => self.apu.read(address),
            0xff40..=0xff45 => self.ppu.read(address),
            0xff46          => self.oam_dma.read(address),
            0xff47..=0xff4b => self.ppu.read(address),
            0xff4f          => {
                warn!("Game Boy Color (VRAM Bank Select)");
                Ok(0)
            },
            0xff50          => self.boot.read(address),
            0xff51..=0xff55 => todo!("Game Boy color"),
            0xff68..=0xff6a => todo!("Game Boy color (DMA)"),
            _               => {
                warn!("Unknown IO address: {:04x}", address);
                Ok(0xff)
            },
        }
    }

    #[rustfmt::skip]
    fn write_io(&mut self, address: Address, data: u8) -> Result<(), Error> {
        match address {
            0xff00          => self.joypad.write(address, data),
            0xff01..=0xff02 => {
                warn!("Port/Mode not implemented: {:04x}, {:02x}", address, data);
                Ok(())
            },
            0xff04..=0xff07 => self.timer.write(address, data),
            0xff0f          => self.irq.write(address, data),
            0xff10..=0xff26 => self.apu.write(address, data),
            0xff30..=0xff3f => self.apu.write(address, data),
            0xff40..=0xff45 => self.ppu.write(address, data),
            0xff46          => {
                self.oam_dma.write(address, data)?;

                // do the OAM transfer all at once, then the emulator will block certain
                // locations in memory until the corresponding number of cycles has elapsed.
                if self.oam_dma.is_active() {
                    self.oam_dma_transfer()?;
                }

                Ok(())
            }
            0xff47..=0xff4b => self.ppu.write(address, data),
            0xff4f          => {
                warn!("Game Boy Color (VRAM Bank Select): {:02x}", data);
                Ok(())
            },
            0xff50          => self.boot.write(address, data),
            0xff51..=0xff55 => todo!("Game Boy color"),
            0xff68..=0xff6a => todo!("Game Boy color (DMA)"),
            _               => {
                warn!("Unknown IO address: {:04x}, data: {:02x}", address, data);
                Ok(())
            },
        }
    }

    fn update_irq(&mut self, request: &Request) -> Result<(), Error> {
        let mut fi = self.read(0xff0f)?;

        if request.vblank {
            //info!("Request VBLANK interrupt");

            fi |= 0b0000_0001
        }
        if request.stat {
            //info!("Request LCDC interrupt");

            fi |= 0b0000_0010
        }
        if request.timer {
            //info!("Request TIMER interrupt");

            fi |= 0b0000_0100
        }
        if request.serial {
            //info!("Request SERIAL interrupt");

            fi |= 0b0000_1000
        }
        if request.joypad {
            //info!("Request JOYPAD interrupt");

            fi |= 0b0001_0000
        }

        self.write(0xff0f, fi)?;
        Ok(())
    }

    fn oam_dma_transfer(&mut self) -> Result<(), Error> {
        let src = self.oam_dma.start_address();

        let src = src..=src | 0x9f;
        let dst = 0xfe00..0xfe9f;

        for (src, dst) in src.zip(dst) {
            let data = self.read(src)?;
            self.write(dst, data)?;
        }
        Ok(())
    }
}

impl<C: Cartridge> Device for Emulator<C> {
    const DEBUG_NAME: &'static str = "Emulator";

    fn read(&self, address: u16) -> Result<u8, Error> {
        let boot = self.boot.is_enabled();

        match address {
            0x0000..=0x00ff if boot => self.boot.read(address),
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
            0xfea0..=0xfeff => {
                error!("Nintendo says use of this area is prohibited (Unused).");

                return Ok(0x42);

                // emulate behavior depending on device & hardware revision
                // https://gbdev.io/pandocs/#fea0-feff-range
                todo!();

                self.stop_running();
                Ok(0x00)
            }
            0xff00..=0xff7f => self.read_io(address),
            0xff80..=0xfffe => self.high_ram.read(address),
            0xffff => self.irq.read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), Error> {
        let boot = self.boot.is_enabled();

        match address {
            0x0000..=0x00ff if boot => self.boot.write(address, data),
            0x0000..=0x7fff => self.cartridge.write(address, data),
            0x8000..=0x9fff => self.ppu.write(address, data),
            0xa000..=0xbfff => self.cartridge.write(address, data),
            0xc000..=0xdfff => self.work_ram.write(address, data),
            // Nintendo says use of this area is prohibited.
            0xe000..=0xfdff => {
                error!("Nintendo says use of this area is prohibited (Echo RAM).");

                self.stop_running();
                Ok(())
            }
            0xfe00..=0xfe9f => self.ppu.write(address, data),
            0xfea0..=0xfeff => {
                error!("Nintendo says use of this area is prohibited (Unused).");

                return Ok(());

                // emulate behavior depending on device & hardware revision
                // https://gbdev.io/pandocs/#fea0-feff-range
                todo!();

                self.stop_running();
                Ok(())
            }
            0xff00..=0xff7f => self.write_io(address, data),
            0xff80..=0xfffe => self.high_ram.write(address, data),
            0xffff => self.irq.write(address, data),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Emulator, Request};

    #[test]
    fn update_irq() {
        let mut emu = Emulator::new(NoCartridge);
        let mut fi = Vec::new();

        fi.push(emu.read(0xff0f).unwrap());

        emu.update_irq(&Request {
            vblank: true,
            ..Default::default()
        })
        .unwrap();
        fi.push(emu.read(0xff0f).unwrap());

        emu.update_irq(&Request {
            stat: true,
            ..Default::default()
        })
        .unwrap();
        fi.push(emu.read(0xff0f).unwrap());

        emu.update_irq(&Request {
            timer: true,
            ..Default::default()
        })
        .unwrap();
        fi.push(emu.read(0xff0f).unwrap());

        emu.update_irq(&Request {
            serial: true,
            ..Default::default()
        })
        .unwrap();
        fi.push(emu.read(0xff0f).unwrap());

        emu.update_irq(&Request {
            joypad: true,
            ..Default::default()
        })
        .unwrap();
        fi.push(emu.read(0xff0f).unwrap());

        assert_eq!(
            vec![0b00000, 0b00001, 0b00011, 0b00111, 0b01111, 0b11111],
            fi
        );
    }

    #[test]
    fn oam_dma() {
        todo!();
    }
}
