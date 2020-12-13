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
    dma::OamDma,
    high_ram::HighRAM,
    irq::IRQ,
    joypad::Joypad,
    ppu::PPU,
    timer::Timer,
    work_ram::WorkRAM,
};
use log::{error, info, warn};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cell::Cell;

// re-exports
use crate::irq::Request;
pub use gb::GameBoy;
pub use gbc::GameBoyColor;
pub use joypad::Button;
pub use ppu::lcd;

mod apu;
mod boot;
pub mod cartridge;
pub mod cpu;
pub mod device;
mod dma;
mod gb;
mod gbc;
mod high_ram;
mod irq;
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
    fn update(&mut self, step: &EmulationStep, request: &mut Request);
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
    oam_dma: OamDma,
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
    fn update(&mut self) {
        if self.running.get() {
            let mut cpu = self.cpu.take().unwrap();
            let cpu_step = cpu.update(self);
            self.cpu = Some(cpu);
            let step = EmulationStep {
                clock_ticks: cpu_step,
            };
            let mut request = Request::default();

            self.oam_dma.update(&step, &mut request);
            self.ppu.update(&step, &mut request);
            self.timer.update(&step, &mut request);

            // request IF register with requested interrupts.
            self.update_irq(&request);
        } else {
            warn!("Emulator is no longer running")
        }
    }

    fn update_frame(&mut self) {
        if self.running.get() {
            // run until VBLANK
            while self.read(0xff41) & 0b11 != 0b01 {
                self.update();
            }
            // run until next OAM search
            while self.read(0xff41) & 0b11 != 0b10 {
                self.update();
            }
        } else {
            warn!("Emulator is no longer running")
        }
    }

    fn stop_running(&self) {
        self.running.set(false);
    }

    #[rustfmt::skip]
    fn read_io(&self, address: Address) -> u8 {
        match address {
            0xff00          => self.joypad.read(address),
            0xff01..=0xff02 => {
                //warn!("Port/Mode not implemented");
                0
            },
            0xff04..=0xff07 => self.timer.read(address),
            0xff0f          => self.irq.read(address),
            0xff10..=0xff26 => self.apu.read(address),
            0xff30..=0xff3f => self.apu.read(address),
            0xff40..=0xff45 => self.ppu.read(address),
            0xff46          => self.oam_dma.read(address),
            0xff47..=0xff4b => self.ppu.read(address),
            0xff4f          => todo!("Game Boy Color (VRAM Bank Select)"),
            0xff50          => self.boot.read(address),
            0xff51..=0xff55 => todo!("Game Boy color"),
            0xff68..=0xff6a => todo!("Game Boy color (DMA)"),
            _               => {
                warn!("Unknown IO address: {:#04x}", address);
                0xff
            },
        }
    }

    #[rustfmt::skip]
    fn write_io(&mut self, address: Address, data: u8) {
        match address {
            0xff00          => self.joypad.write(address, data),
            0xff01..=0xff02 => {
                //warn!("Port/Mode not implemented");
            },
            0xff04..=0xff07 => self.timer.write(address, data),
            0xff0f          => self.irq.write(address, data),
            0xff10..=0xff26 => self.apu.write(address, data),
            0xff30..=0xff3f => self.apu.write(address, data),
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
            0xff4f          => {
                warn!("Game Boy Color (VRAM Bank Select): {:#x}", data);
            },
            0xff50          => self.boot.write(address, data),
            0xff51..=0xff55 => todo!("Game Boy color"),
            0xff68..=0xff6a => todo!("Game Boy color (DMA)"),
            _               => {
                warn!("Unknown IO address: {:#04x}, data: {:#x}", address, data);
            },
        }
    }

    fn update_irq(&mut self, request: &Request) {
        let mut fi = self.read(0xff0f);

        if request.vblank {
            info!("Request VBLANK interrupt");

            fi |= 0b0000_0001
        }
        if request.lcd_stat {
            info!("Request LCDC interrupt");

            fi |= 0b0000_0010
        }
        if request.timer {
            info!("Request TIMER interrupt");

            fi |= 0b0000_0100
        }
        if request.serial {
            info!("Request SERIAL interrupt");

            fi |= 0b0000_1000
        }
        if request.joypad {
            info!("Request JOYPAD interrupt");

            fi |= 0b0001_0000
        }

        self.write(0xff0f, fi);
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
    const DEBUG_NAME: &'static str = "Emulator";

    fn read(&self, address: u16) -> u8 {
        let boot = self.boot.is_enabled();

        match address {
            0x0000..=0x00ff if boot => self.boot.read(address),
            0x0000..=0x7fff => self.cartridge.read(address),
            0x8000..=0x9fff => self.ppu.read(address),
            0xa000..=0xbfff => self.cartridge.read(address),
            0xc000..=0xdfff => self.work_ram.read(address),
            // Nintendo says use of this area is prohibited.
            0xe000..=0xfdff => {
                warn!("Echo RAM");
                warn!("Nintendo says use of this area is prohibited");
                self.read(address - 0xe000 + 0xc000)
            }
            0xfe00..=0xfe9f => self.ppu.read(address),
            0xfea0..=0xfeff => {
                return 0x42;

                // emulate behavior depending on device & hardware revision
                // https://gbdev.io/pandocs/#fea0-feff-range
                todo!();

                error!("Memory region not in use");
                error!("Nintendo says use of this area is prohibited!");
                self.stop_running();
                0x00
            }
            0xff00..=0xff7f => self.read_io(address),
            0xff80..=0xfffe => self.high_ram.read(address),
            0xffff => self.irq.read(address),
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        let boot = self.boot.is_enabled();

        match address {
            0x0000..=0x00ff if boot => self.boot.write(address, data),
            0x0000..=0x7fff => self.cartridge.write(address, data),
            0x8000..=0x9fff => self.ppu.write(address, data),
            0xa000..=0xbfff => self.cartridge.write(address, data),
            0xc000..=0xdfff => self.work_ram.write(address, data),
            // Nintendo says use of this area is prohibited.
            0xe000..=0xfdff => {
                error!("Echo RAM");
                error!("Nintendo says use of this area is prohibited");
                self.stop_running();
            }
            0xfe00..=0xfe9f => self.ppu.write(address, data),
            0xfea0..=0xfeff => {
                return;

                // emulate behavior depending on device & hardware revision
                // https://gbdev.io/pandocs/#fea0-feff-range
                todo!();

                error!("Memory region not in use");
                error!("Nintendo says use of this area is prohibited!");
                self.stop_running();
            }
            0xff00..=0xff7f => self.write_io(address, data),
            0xff80..=0xfffe => self.high_ram.write(address, data),
            0xffff => self.irq.write(address, data),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        cartridge::{Cartridge, NoCartridge},
        device::Device,
        irq::Request,
        Emulator,
    };

    #[test]
    fn update_irq() {
        let mut emu = Emulator::new(NoCartridge);
        let mut fi = Vec::new();

        fi.push(emu.read(0xff0f));

        emu.update_irq(&Request {
            vblank: true,
            ..Default::default()
        });
        fi.push(emu.read(0xff0f));

        emu.update_irq(&Request {
            lcd_stat: true,
            ..Default::default()
        });
        fi.push(emu.read(0xff0f));

        emu.update_irq(&Request {
            timer: true,
            ..Default::default()
        });
        fi.push(emu.read(0xff0f));

        emu.update_irq(&Request {
            serial: true,
            ..Default::default()
        });
        fi.push(emu.read(0xff0f));

        emu.update_irq(&Request {
            joypad: true,
            ..Default::default()
        });
        fi.push(emu.read(0xff0f));

        assert_eq!(
            vec![0b00000, 0b00001, 0b00011, 0b00111, 0b01111, 0b11111,],
            fi
        );
    }

    #[test]
    fn oam_dma() {
        todo!();
    }
}
