#![cfg_attr(
    debug_assertions,
    deny(
        unused,
        future_incompatible,
        nonstandard_style,
        rust_2021_incompatible_or_patterns,
        rust_2021_incompatible_closure_captures,
        rust_2021_compatibility,
        rust_2021_prelude_collisions,
        rust_2021_prefixes_incompatible_syntax,
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
        clippy::needless_borrow
    )
)]

#[macro_export]
macro_rules! dev_read {
    ($address:ident { $($tt:tt)* }) => {
        match $address {
            $($tt)*
            _ => return Err($crate::error::ReadError::InvalidAddress($address))
        }
    }
}

#[macro_export]
macro_rules! dev_write {
    ($address:ident, $data:ident { $($tt:tt)* }) => {
        match $address {
            $($tt)*
            _ => return Err($crate::error::WriteError::InvalidAddress($address, $data))
        }
    }
}

#[cfg(feature = "cgb")]
use crate::dma::VRAMDMA;
use crate::{
    apu::APU,
    boot::Boot,
    cartridge::Cartridge,
    cpu::CPU,
    device::{Device, MainDevice},
    dma::OAMDMA,
    error::{Error, ReadError, WriteError},
    irq::IRQ,
    joypad::{Button, Joypad},
    ppu::{LCD, PPU},
    ram::{HighRAM, WorkRAM, VRAM},
    serial::Serial,
    timer::Timer,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod apu;
mod boot;
pub mod cartridge;
pub mod cpu;
pub mod device;
mod dma;
pub mod error;
pub mod gb;
mod irq;
pub mod joypad;
pub mod ppu;
mod ram;
mod serial;
mod timer;
mod utils;

const CLOCK: u64 = 4_194_304;

trait Update {
    fn update(&mut self, ticks: u64, flags: &mut irq::Flags);
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LR35902<C: Cartridge, O: LCD> {
    // borrow checker workaround
    // cpu will be leaving the Option temporarily
    cpu: Option<CPU>,
    cartridge: C,
    boot: Boot,
    oam_dma: OAMDMA,
    oam_buf: Option<Box<[u8; 0xa0]>>,
    #[cfg(feature = "cgb")]
    vram_dma: VRAMDMA,
    joypad: Joypad,
    ppu: PPU<O>,
    timer: Timer,
    work_ram: WorkRAM,
    high_ram: HighRAM,
    irq: IRQ,
    apu: APU,
    serial: Serial,
    double_speed: bool,
}

impl<C: Cartridge, O: LCD> LR35902<C, O> {
    fn new(cartridge: C, output: O) -> Self {
        Self {
            cpu: Some(Default::default()),
            cartridge,
            boot: Default::default(),
            oam_dma: Default::default(),
            oam_buf: Some(Box::new([0; 0xa0])),
            #[cfg(feature = "cgb")]
            vram_dma: Default::default(),
            joypad: Default::default(),
            ppu: PPU::new(output),
            timer: Default::default(),
            work_ram: Default::default(),
            high_ram: Default::default(),
            irq: Default::default(),
            apu: Default::default(),
            serial: Default::default(),
            double_speed: false,
        }
    }
}

impl<C: Cartridge, O: LCD> LR35902<C, O> {
    /// Return the PPU.
    pub fn ppu(&self) -> &PPU<O> {
        return &self.ppu;
    }

    /// Returns the CPU.
    pub fn cpu(&self) -> &CPU {
        if let Some(cpu) = self.cpu.as_ref() {
            cpu
        } else {
            unreachable!()
        }
    }

    /// Return the VRAM device
    pub fn vram(&self) -> &VRAM {
        self.ppu.vram()
    }

    #[cfg(feature = "debug")]
    fn set_debug_overlay(&mut self, b: bool) {
        self.ppu.set_debug_overlays(b);
    }

    /// Run next instruction and update state o the components.
    fn step(&mut self) -> Result<(), Error> {
        // borrow-checker workaround / hack
        // temporarily take ownership of CPU away
        let mut cpu = self.cpu.take().unwrap();
        let mut ticks = cpu.update(self)?;
        self.cpu = Some(cpu);

        // all cpu instructions take a multiple of 4 cycles so this is fine
        if self.double_speed {
            ticks /= 2;
        }

        // update the rest of the components in lockstep
        // collect interrupts in the process
        let mut flags = irq::Flags::empty();

        self.timer.update(ticks, &mut flags);
        self.ppu.update(ticks, &mut flags);

        // update interrupt registers
        self.irq.fi |= flags;
        Ok(())
    }

    fn press(&mut self, button: &Button) {
        self.joypad.press(button);
        self.irq.fi |= irq::Flags::JOYPAD;
    }

    fn release(&mut self, button: &Button) {
        self.joypad.release(button)
    }

    fn emulate_until_vblank_ends(&mut self) -> Result<(), Error> {
        // run until VBLANK is hit
        while <Self as Device>::read(self, 0xff41)? & 0b11 != 0b01 {
            self.step()?;
        }

        // run VBLANK (until OAM search is hit)
        while <Self as Device>::read(self, 0xff41)? & 0b11 != 0b10 {
            self.step()?;
        }

        Ok(())
    }

    fn do_oam_dma(&mut self) {
        let mut oam_buf = self.oam_buf.take().unwrap();
        let src = (self.oam_dma.dma as u16) << 8;
        self.read_exact(src, &mut oam_buf[..]).unwrap();
        self.ppu.write_exact(0xfe00, &oam_buf[..]).unwrap();
        self.oam_buf = Some(oam_buf);
    }

    // TODO optimise using memcpy
    #[cfg(feature = "cgb")]
    fn do_vram_dma(&mut self, hdma5: u8) {
        let hdma1 = self.vram_dma.hdma1 as u16;
        let hdma2 = self.vram_dma.hdma2 as u16;
        let hdma3 = self.vram_dma.hdma3 as u16;
        let hdma4 = self.vram_dma.hdma4 as u16;
        let hdma5 = hdma5 as u16;

        let source = (hdma1 << 8) | (hdma2 & 0xf0);
        let destination = 0x8000 | ((hdma3 & 0x1f) << 8) | (hdma4 & 0xf0);
        let len = ((hdma5 & 0x7f) + 1) * 16;

        let src = source..source + len;
        let dst = destination..destination + len;
        for (src, dst) in src.zip(dst) {
            let data = <Self as MainDevice>::read(self, src).unwrap();
            <Self as MainDevice>::write(self, dst, data).unwrap();
        }
    }

    fn read_double_speed(&self) -> Result<u8, ReadError> {
        if self.double_speed {
            Ok(0b10000000)
        } else {
            Ok(0)
        }
    }

    fn write_double_speed(&mut self, data: u8) {
        if (data & 1) != 0 {
            self.double_speed = true;
        }
    }
}

impl<C: Cartridge, O: LCD> Device for LR35902<C, O> {
    #[allow(unreachable_patterns)]
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        let boot = self.boot.is_enabled();

        dev_read! {
            address {
                #[cfg(not(feature = "cgb"))]
                0x0000..=0x00ff if boot => self.boot.read(address),
                #[cfg(feature = "cgb")]
                0x0000..=0x00ff | 0x0150..=0x0900 if boot => self.boot.read(address),
                0x0000..=0x7fff => self.cartridge.read(address),
                0x8000..=0x9fff => self.ppu.read(address),
                0xa000..=0xbfff => self.cartridge.read(address),
                0xc000..=0xdfff => self.work_ram.read(address),
                0xe000..=0xfdff => {
                    log::warn!("ECHO RAM read at address 0x{address:04x}");
                    self.work_ram.read(address - 0x2000)
                }
                0xfe00..=0xfe9f => self.ppu.read(address),
                // TODO emulate behavior depending on device & hardware revision (https://gbdev.io/pandocs/#fea0-feff-range)
                0xfea0..=0xfeff => Ok(0xff), // return this for now...
                // IO registers
                0xff00 => self.joypad.read(address),
                0xff01..=0xff02 => self.serial.read(address),
                0xff04..=0xff07 => self.timer.read(address),
                0xff0f => self.irq.read(address),
                0xff10..=0xff26 => self.apu.read(address),
                0xff30..=0xff3f => self.apu.read(address),
                0xff40..=0xff45 => self.ppu.read(address),
                0xff46 => self.oam_dma.read(address),
                0xff47..=0xff4b => self.ppu.read(address),
                0xff4d => self.read_double_speed(),
                0xff4f => self.ppu.read(address),
                0xff50 => self.boot.read(address),
                #[cfg(feature = "cgb")]
                0xff51..=0xff54 => self.vram_dma.read(address),
                // TODO Emulate OAM timings
                0xff55 => Ok(0xff),
                0xff68..=0xff6b => self.ppu.read(address),
                0xff70 => self.work_ram.read(address),
                0xff71..=0xff7f => Err(ReadError::InvalidAddress(address)), // undocumented registers
                //
                0xff80..=0xfffe => self.high_ram.read(address),
                0xffff => self.irq.read(address),
            }
        }
    }

    #[allow(unreachable_patterns)]
    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        let boot = self.boot.is_enabled();
        dev_write! {
            address, data {
                #[cfg(not(feature = "cgb"))]
                0x0000..=0x00ff if boot => self.boot.write(address, data),
                #[cfg(feature = "cgb")]
                0x0000..=0x00ff | 0x0150..=0x0900 if boot => self.boot.write(address, data),
                0x0000..=0x7fff => self.cartridge.write(address, data),
                0x8000..=0x9fff => self.ppu.write(address, data),
                0xa000..=0xbfff => self.cartridge.write(address, data),
                0xc000..=0xdfff => self.work_ram.write(address, data),
                0xe000..=0xfdff => {
                    log::warn!("ECHO RAM write at address 0x{address:04x} data 0x{data:02x}");
                    self.work_ram.write(address - 0x2000, data)
                }
                0xfe00..=0xfe9f => self.ppu.write(address, data),
                // TODO emulate behavior depending on device & hardware revision (https://gbdev.io/pandocs/#fea0-feff-range)
                0xfea0..=0xfeff => Ok(()),
                // IO registers
                0xff00 => self.joypad.write(address, data),
                0xff01..=0xff02 => self.serial.write(address, data),
                0xff04..=0xff07 => self.timer.write(address, data),
                0xff0f => self.irq.write(address, data),
                0xff10..=0xff26 => self.apu.write(address, data),
                0xff30..=0xff3f => self.apu.write(address, data),
                0xff40..=0xff45 => self.ppu.write(address, data),
                0xff46 => {
                    self.oam_dma.write(address, data)?;
                    // TODO emulate OAM timings
                    self.do_oam_dma();
                    Ok(())
                }
                0xff47..=0xff4b => self.ppu.write(address, data),
                0xff4d => {
                    self.write_double_speed(data);
                    Ok(())
                }
                0xff4f => self.ppu.write(address, data),
                0xff50 => self.boot.write(address, data),
                #[cfg(feature = "cgb")]
                0xff51..=0xff54 => self.vram_dma.write(address, data),
                #[cfg(feature = "cgb")]
                0xff55 => {
                    // TODO emulate HDMA timings
                    self.do_vram_dma(data);
                    Ok(())
                }
                0xff68..=0xff6b => self.ppu.write(address, data),
                0xff70 => self.work_ram.write(address, data),
                0xff71..=0xff7f => Err(WriteError::InvalidAddress(address, data)), // undocumented registers
                //
                0xff80..=0xfffe => self.high_ram.write(address, data),
                0xffff => self.irq.write(address, data),
            }
        }
    }

    fn read_exact(&self, address: u16, buf: &mut [u8]) -> Result<(), ReadError> {
        match address {
            0x0000..=0x7fff if (address as usize) + buf.len() <= 0x7fff => {
                // cartridge rom
                Ok(())
            }
            0xa000..=0xbfff if (address as usize) + buf.len() <= 0xbfff => {
                // cartridge ram
                Ok(())
            }
            0xc000..=0xdfff if (address as usize) + buf.len() <= 0xdfff => {
                self.work_ram.read_exact(address, buf)
            }
            _ => self.read_exact_fallback(address, buf),
        }
    }
}

impl<C: Cartridge, O: LCD> MainDevice for LR35902<C, O> {}

#[cfg(test)]
mod test {
    #[test]
    fn oam_dma() {
        todo!();
    }
}
