#![cfg_attr(
    debug_assertions,
    allow(
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
            _ => return Err($crate::error::ReadError::UnknownAddr($address))
        }
    }
}

#[macro_export]
macro_rules! dev_write {
    ($address:ident, $data:ident { $($tt:tt)* }) => {
        match $address {
            $($tt)*
            _ => return Err($crate::error::WriteError::UnknownAddr($address, $data))
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
    debug::Breakpoint,
    device::{Device, MemoryBus},
    dma::OAMDMA,
    error::{Error, ReadError, WriteError},
    irq::IRQ,
    joypad::{Button, Joypad},
    ppu::{LCD, PPU},
    ram::{hram::HRAM, vram::VRAM, wram::WRAM},
    serial::Serial,
    timer::Timer,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod apu;
mod boot;
pub mod cartridge;
pub mod cpu;
pub mod debug;
pub mod device;
mod dma;
pub mod error;
pub mod gb;
mod irq;
pub mod joypad;
pub mod ppu;
pub mod ram;
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
    work_ram: WRAM,
    high_ram: HRAM,
    irq: IRQ,
    apu: APU,
    serial: Serial,
    #[cfg(feature = "cgb")]
    double_speed: bool,
}

impl<C: Cartridge, O: LCD> LR35902<C, O> {
    pub fn new(cartridge: C, output: O) -> Self {
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
            #[cfg(feature = "cgb")]
            double_speed: false,
        }
    }

    /// Return the current frequency of the CPU.
    pub fn clock_freq(&self) -> u64 {
        #[cfg(feature = "cgb")]
        return if self.double_speed { CLOCK * 2 } else { CLOCK };
        #[cfg(not(feature = "cgb"))]
        return CLOCK;
    }

    /// Return the PPU.
    pub fn ppu(&self) -> &PPU<O> {
        return &self.ppu;
    }

    /// Return the PPU as mutable.
    pub fn ppu_mut(&mut self) -> &mut PPU<O> {
        return &mut self.ppu;
    }

    /// Returns the CPU.
    pub fn cpu(&self) -> &CPU {
        self.cpu.as_ref().unwrap()
    }

    /// Returns the CPU as mutable.
    pub fn cpu_mut(&mut self) -> &mut CPU {
        self.cpu.as_mut().unwrap()
    }

    /// Return the VRAM device
    pub fn vram(&self) -> &VRAM {
        self.ppu.vram()
    }

    fn update_cpu(&mut self) -> Result<u64, Error> {
        // borrow-checker workaround / hack
        // temporarily take ownership of CPU away
        let mut cpu = self.cpu.take().unwrap();
        match cpu.update(self) {
            Ok(ticks) => {
                self.cpu = Some(cpu);
                Ok(ticks)
            }
            Err(err) => {
                self.cpu = Some(cpu);
                Err(err)
            }
        }
    }

    pub fn step(&mut self) -> Result<(), Error> {
        let mut ticks = self.update_cpu()?;

        // run 2x clocks in the CPU if double speed is enabled
        #[cfg(feature = "cgb")]
        if self.double_speed {
            ticks /= 2;
        }

        let mut flags = irq::Flags::empty();

        // sync the rest of the components
        self.timer.update(ticks, &mut flags);
        self.ppu.update(ticks, &mut flags);

        self.irq.fi |= flags;
        Ok(())
    }

    /// Step emulation until a breakpoint is hit.
    pub fn step_breakpoint<B: Breakpoint>(&mut self, mut breakpoit: B) -> Result<B, Error> {
        loop {
            breakpoit.init(self);
            self.step()?;
            if breakpoit.breakpoint(self) {
                break;
            }
        }
        Ok(breakpoit)
    }

    fn press(&mut self, button: &Button) {
        self.joypad.press(button);
        self.irq.fi |= irq::Flags::JOYPAD;
    }

    fn release(&mut self, button: &Button) {
        self.joypad.release(button)
    }

    // TODO(german) emulate OAM timings
    fn do_oam_dma(&mut self, data: u8) {
        let mut oam_buf = self.oam_buf.take().unwrap();
        let src = (data as u16) << 8;
        self.read_exact(src, &mut oam_buf[..]).unwrap();
        self.ppu.write_exact(0xfe00, &oam_buf[..]).unwrap();
        self.oam_buf = Some(oam_buf);
    }

    // TODO(german) optimise using memcpy
    // TODO(german) emulate VRAM DMA timings
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
            let data = <Self as MemoryBus>::read(self, src).unwrap();
            <Self as MemoryBus>::write(self, dst, data).unwrap();
        }
    }

    #[cfg(feature = "cgb")]
    fn double_speed_read(&self) -> Result<u8, ReadError> {
        if self.double_speed {
            Ok(0x80)
        } else {
            Ok(0)
        }
    }

    #[cfg(feature = "cgb")]
    fn double_speed_write(&mut self, data: u8) -> Result<(), WriteError> {
        if (data & 1) != 0 {
            self.double_speed = true;
        }
        Ok(())
    }
}

impl<C: Cartridge, O: LCD> Device for LR35902<C, O> {
    #[allow(unreachable_patterns)]
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        let boot = self.boot.is_enabled();
        dev_read! {
            address {
                0x0000..=0x00ff if boot => self.boot.read(address),
                #[cfg(feature = "cgb")]
                0x0150..=0x0900 if boot => self.boot.read(address),
                0x0000..=0x7fff => self.cartridge.read(address),
                0x8000..=0x9fff => self.ppu.read(address),
                0xa000..=0xbfff => self.cartridge.read(address),
                0xc000..=0xdfff => self.work_ram.read(address),
                // ECHO RAM
                0xe000..=0xfdff => self.work_ram.read(address),
                0xfe00..=0xfe9f => self.ppu.read(address),
                // TODO emulate behavior depending on device & hardware revision (https://gbdev.io/pandocs/#fea0-feff-range)
                0xfea0..=0xfeff => Ok(0x00), // return this for now...
                // IO registers
                0xff00 => self.joypad.read(address),
                0xff01 | 0xff02 => self.serial.read(address),
                0xff04..=0xff07 => self.timer.read(address),
                0xff0f => self.irq.read(address),
                0xff10..=0xff14 |
                0xff15..=0xff19 |
                0xff1a..=0xff1e |
                0xff1f..=0xff26 |
                0xff27..=0xff2f |
                0xff30..=0xff3f => self.apu.read(address),
                0xff40..=0xff45 |
                0xff47..=0xff4b |
                0xff4f          |
                0xff68..=0xff6b => self.ppu.read(address),
                0xff46 => self.oam_dma.read(address),
                #[cfg(feature = "cgb")]
                0xff4d => self.double_speed_read(),
                0xff50 => self.boot.read(address),
                #[cfg(feature = "cgb")]
                0xff51..=0xff54 => self.vram_dma.read(address),
                // TODO Emulate OAM timings
                0xff55 => Ok(0xff),
                0xff70 => self.work_ram.read(address),
                0xff71..=0xff7f => Err(ReadError::UnknownAddr(address)), // undocumented registers
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
                0x0000..=0x7fff => self.cartridge.write(address, data),
                0x8000..=0x9fff => self.ppu.write(address, data),
                0xa000..=0xbfff => self.cartridge.write(address, data),
                0xc000..=0xdfff => self.work_ram.write(address, data),
                // ECHO RAM
                0xe000..=0xfdff => self.work_ram.write(address, data),
                0xfe00..=0xfe9f => self.ppu.write(address, data),
                // TODO emulate behavior depending on device & hardware revision (https://gbdev.io/pandocs/#fea0-feff-range)
                0xfea0..=0xfeff => Ok(()),
                // IO registers
                0xff00 => self.joypad.write(address, data),
                0xff01 | 0xff02 => self.serial.write(address, data),
                0xff04..=0xff07 => self.timer.write(address, data),
                0xff0f => self.irq.write(address, data),
                0xff10..=0xff14 |
                0xff15..=0xff19 |
                0xff1a..=0xff1e |
                0xff1f..=0xff26 |
                0xff27..=0xff2f |
                0xff30..=0xff3f => self.apu.write(address, data),
                0xff40..=0xff45 |
                0xff47..=0xff4b |
                0xff4f          |
                0xff68..=0xff6b => self.ppu.write(address, data),
                0xff46 => {
                    self.do_oam_dma(data);
                    Ok(())
                }
                #[cfg(feature = "cgb")]
                0xff4d => self.double_speed_write(data),
                0xff50 => self.boot.write(address, data),
                #[cfg(feature = "cgb")]
                0xff51..=0xff54 => self.vram_dma.write(address, data),
                #[cfg(feature = "cgb")]
                0xff55 => {
                    self.do_vram_dma(data);
                    Ok(())
                }
                0xff70 => self.work_ram.write(address, data),
                0xff71..=0xff7f => Err(WriteError::UnknownAddr(address, data)), // undocumented registers
                //
                0xff80..=0xfffe => self.high_ram.write(address, data),
                0xffff => self.irq.write(address, data),
            }
        }
    }

    fn read_exact(&self, address: u16, buf: &mut [u8]) -> Result<(), ReadError> {
        match address {
            // 0x0000..=0x7fff if (address as usize) + buf.len() <= 0x7fff => {
            //     // cartridge rom
            //     Ok(())
            // }
            // 0xa000..=0xbfff if (address as usize) + buf.len() <= 0xbfff => {
            //     // cartridge ram
            //     Ok(())
            // }
            // 0xc000..=0xdfff if (address as usize) + buf.len() <= 0xdfff => {
            //     self.work_ram.read_exact(address, buf)
            // }
            _ => self.read_exact_fallback(address, buf),
        }
    }
}

impl<C: Cartridge, O: LCD> MemoryBus for LR35902<C, O> {}

#[cfg(test)]
mod test {
    #[test]
    fn oam_dma() {
        todo!();
    }
}
