use crate::{
    cartridge::Cartridge, cpu::CPU, device::Device, error::Error, lcd::Display, Button, Emulator,
    Request,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Game Boy (non-color) emulator.
///
/// ```
/// use core::{GameBoy, Button, cartridge::NoCartridge};
///
/// let mut gb = GameBoy::new(NoCartridge);
///
/// // mario jump :)
/// gb.press(&Button::Right);
/// gb.press(&Button::A);
///
/// let lcd = gb.ppu().display();
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GameBoy<C: Cartridge> {
    booted: bool,
    emulator: Emulator<C>,
}

impl<C: Cartridge> GameBoy<C> {
    pub fn new(cartridge: C) -> Result<Self, Error> {
        let mut emulator = Self {
            booted: false,
            emulator: Emulator::new(cartridge),
        };
        if cfg!(not(feature = "boot")) {
            emulator.skip_boot()?;
        }
        Ok(emulator)
    }

    pub fn set_debug_overlays(&mut self, b: bool) {
        self.emulator.set_debug_overlay(b)
    }

    pub fn display(&self) -> &Display {
        self.emulator.ppu.display()
    }

    pub fn cpu(&self) -> &CPU {
        self.emulator.cpu.as_ref().unwrap()
    }

    pub fn cpu_mut(&mut self) -> &mut CPU {
        self.emulator.cpu.as_mut().unwrap()
    }

    pub fn press(&mut self, button: &Button) -> Result<(), Error> {
        self.emulator.joypad.press(button);
        self.emulator.update_irq(&Request {
            joypad: true,
            ..Default::default()
        })?;
        Ok(())
    }

    pub fn release(&mut self, button: &Button) {
        self.emulator.joypad.release(button)
    }

    /// Skip boot sequence.
    pub fn skip_boot(&mut self) -> Result<(), Error> {
        if self.booted {
            if cfg!(debug_assertions) {
                panic!("Emulator is already booted!");
            }
        } else {
            self.booted = true;
            self.boot_memory()?;
            self.boot_cpu();
        }
        Ok(())
    }

    /// Update emulator.
    pub fn update_frame(&mut self) -> Result<(), Error> {
        self.emulator.update_frame()
    }

    fn boot_cpu(&mut self) {
        let cpu = self.emulator.cpu.as_mut().unwrap();

        cpu.registers_mut().set_af(0x01b0);
        cpu.registers_mut().set_bc(0x0013);
        cpu.registers_mut().set_de(0x00d8);
        cpu.registers_mut().set_hl(0x014d);
        cpu.registers_mut().sp = 0xfffe;
        cpu.registers_mut().pc = 0x0100;
    }

    fn boot_memory(&mut self) -> Result<(), Error> {
        let mmu = &mut self.emulator;

        mmu.write(0xff05, 0x00)?;
        mmu.write(0xff06, 0x00)?;
        mmu.write(0xff07, 0x00)?;
        mmu.write(0xff10, 0x80)?;
        mmu.write(0xff11, 0xbf)?;
        mmu.write(0xff12, 0xf3)?;
        mmu.write(0xff14, 0xbf)?;
        mmu.write(0xff16, 0x3f)?;
        mmu.write(0xff17, 0x00)?;
        mmu.write(0xff19, 0xbf)?;
        mmu.write(0xff1a, 0x7f)?;
        mmu.write(0xff1b, 0xff)?;
        mmu.write(0xff1c, 0x9f)?;
        mmu.write(0xff1e, 0xbf)?;
        mmu.write(0xff20, 0xff)?;
        mmu.write(0xff21, 0x00)?;
        mmu.write(0xff22, 0x00)?;
        mmu.write(0xff23, 0xbf)?;
        mmu.write(0xff24, 0x77)?;
        mmu.write(0xff25, 0xf3)?;
        mmu.write(0xff26, 0xf1)?;
        mmu.write(0xff40, 0x91)?;
        mmu.write(0xff42, 0x00)?;
        mmu.write(0xff43, 0x00)?;
        mmu.write(0xff45, 0x00)?;
        mmu.write(0xff47, 0xfc)?;
        mmu.write(0xff48, 0xff)?;
        mmu.write(0xff49, 0xff)?;
        mmu.write(0xff4a, 0x00)?;
        mmu.write(0xff4b, 0x00)?;
        mmu.write(0xffff, 0x00)?;
        mmu.write(0xff50, 0x01)?;

        Ok(())
    }
}

impl<C: Cartridge> Device for GameBoy<C> {
    const DEBUG_NAME: &'static str = "GameBoy (DMG01)";

    fn read(&self, address: u16) -> Result<u8, Error> {
        self.emulator.read(address)
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), Error> {
        self.emulator.write(address, data)
    }
}
