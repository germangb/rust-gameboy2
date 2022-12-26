use crate::{
    cartridge::Cartridge, device::Device, error::Error, joypad::Button, ppu::LCD, LR35902,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Game Boy emulator.
///
/// ```
/// use core::{cartridge::MBC1, gb::GameBoy, joypad::Button};
///
/// let mut gb = GameBoy::new(MBC1::new(include_bytes!("mario.gb")), ());
///
/// // mario jump :)
/// gb.press(&Button::Right);
/// gb.press(&Button::A);
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GameBoy<C: Cartridge, O: LCD> {
    booted: bool,
    soc: LR35902<C, O>,
}

impl<C: Cartridge, O: LCD> GameBoy<C, O> {
    pub fn new(cartridge: C, output: O) -> Result<Self, Error> {
        let mut emulator = Self {
            booted: false,
            soc: LR35902::new(cartridge, output),
        };
        if cfg!(not(feature = "boot")) {
            emulator.skip_boot()?;
        }
        Ok(emulator)
    }

    /// Update emulator up to the next frame has been fully processed.
    /// In emulation terms, until the emulation detects a transition from VBLANK
    /// to OAM SEARCH state in the pixel processing unit (PPU).
    pub fn emulate_frame(&mut self) -> Result<(), Error> {
        self.soc.emulate_until_vblank_ends()
    }

    /// Get the SOC device.
    pub fn soc(&self) -> &LR35902<C, O> {
        &self.soc
    }

    /// Get the SOC device as mutable.
    pub fn soc_mut(&mut self) -> &mut LR35902<C, O> {
        &mut self.soc
    }

    /// Register a button press.
    /// This operation triggers the JOYPAD interrupt.
    pub fn press(&mut self, button: &Button) {
        self.soc.press(button);
    }

    /// Register a button release.
    pub fn release(&mut self, button: &Button) {
        self.soc.release(button)
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

    #[cfg(feature = "debug")]
    pub fn set_debug_overlays(&mut self, b: bool) {
        self.soc.set_debug_overlay(b)
    }

    fn boot_cpu(&mut self) {
        let cpu = self.soc.cpu.as_mut().unwrap();

        cpu.reg_mut().set_af(0x01b0);
        cpu.reg_mut().set_bc(0x0013);
        cpu.reg_mut().set_de(0x00d8);
        cpu.reg_mut().set_hl(0x014d);
        cpu.reg_mut().sp = 0xfffe;
        cpu.reg_mut().pc = 0x0100;

        if cfg!(feature = "cgb") {
            cpu.reg_mut().a = 0x11;
        }
    }

    fn boot_memory(&mut self) -> Result<(), Error> {
        let mmu = &mut self.soc;

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
