use crate::{
    cartridge::Cartridge,
    cpu::CPU,
    dev::Device,
    joypad::{Button, Joypad},
    ppu::PPU,
    Emulator,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Game Boy (non-color) emulator.
///
/// ```
/// use core::GameBoy;
/// use core::cartridge::NoCartridge;
/// use core::joypad::Button;
///
/// let mut gb = GameBoy::new(NoCartridge);
///
/// // mario jump :)
/// gb.joypad_mut().press(&Button::Right);
/// gb.joypad_mut().press(&Button::A);
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GameBoy<C> {
    booted: bool,
    emulator: Emulator<C>,
}

impl<C> GameBoy<C> {
    pub fn new(cartridge: C) -> Self {
        let mut emulator = Self {
            booted: false,
            emulator: Emulator::new(cartridge),
        };
        #[cfg(not(feature = "boot"))]
        emulator.skip_boot();
        emulator
    }

    pub fn cpu(&self) -> &CPU {
        &self.emulator.cpu
    }

    pub fn cpu_mut(&mut self) -> &mut CPU {
        &mut self.emulator.cpu
    }

    pub fn ppu(&self) -> &PPU {
        &self.emulator.ppu
    }

    pub fn joypad_mut(&mut self) -> &mut Joypad {
        &mut self.emulator.joypad
    }
}

impl<C: Cartridge> GameBoy<C> {
    /// Skip boot sequence.
    pub fn skip_boot(&mut self) {
        if self.booted {
            if cfg!(debug_assertions) {
                panic!("Emulator is already booted!");
            }
        } else {
            self.booted = true;
            self.boot_memory();
            self.boot_cpu();
        }
    }

    fn boot_cpu(&mut self) {
        let cpu = &mut self.emulator.cpu;

        cpu.registers_mut().set_af(0x01b0);
        cpu.registers_mut().set_bc(0x0013);
        cpu.registers_mut().set_de(0x00d8);
        cpu.registers_mut().set_hl(0x014d);
        cpu.registers_mut().sp = 0xfffe;
        cpu.registers_mut().pc = 0x0100;
    }

    fn boot_memory(&mut self) {
        let mmu = &mut self.emulator;

        mmu.write(0xff05, 0x00);
        mmu.write(0xff06, 0x00);
        mmu.write(0xff07, 0x00);
        mmu.write(0xff10, 0x80);
        mmu.write(0xff11, 0xbf);
        mmu.write(0xff12, 0xf3);
        mmu.write(0xff14, 0xbf);
        mmu.write(0xff16, 0x3f);
        mmu.write(0xff17, 0x00);
        mmu.write(0xff19, 0xbf);
        mmu.write(0xff1a, 0x7f);
        mmu.write(0xff1b, 0xff);
        mmu.write(0xff1c, 0x9f);
        mmu.write(0xff1e, 0xbf);
        mmu.write(0xff20, 0xff);
        mmu.write(0xff21, 0x00);
        mmu.write(0xff22, 0x00);
        mmu.write(0xff23, 0xbf);
        mmu.write(0xff24, 0x77);
        mmu.write(0xff25, 0xf3);
        mmu.write(0xff26, 0xf1);
        mmu.write(0xff40, 0x91);
        mmu.write(0xff42, 0x00);
        mmu.write(0xff43, 0x00);
        mmu.write(0xff45, 0x00);
        mmu.write(0xff47, 0xfc);
        mmu.write(0xff48, 0xff);
        mmu.write(0xff49, 0xff);
        mmu.write(0xff4a, 0x00);
        mmu.write(0xff4b, 0x00);
        mmu.write(0xffff, 0x00);
        mmu.write(0xff50, 0x01);
    }
}

impl<C: Cartridge> Device for GameBoy<C> {
    fn read(&self, address: u16) -> u8 {
        self.emulator.read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        self.emulator.write(address, data)
    }
}
