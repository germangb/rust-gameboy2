use crate::{
    cartridge::Cartridge,
    debug::NextFrame,
    device::{Device, MemoryBus},
    error::Error,
    joypad::Button,
    ppu::LCD,
    LR35902,
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
    soc: LR35902<C, O>,
}

impl<C: Cartridge, O: LCD> GameBoy<C, O> {
    pub fn new(cartridge: C, output: O) -> Self {
        let soc = LR35902::new(cartridge, output);
        Self { soc }
    }

    pub fn new_noboot(cartridge: C, output: O) -> Result<Self, Error> {
        let mut gb = Self::new(cartridge, output);
        gb.boot()?;
        Ok(gb)
    }

    /// Update until emulator reaches the next frame.
    pub fn next_frame(&mut self) -> Result<(), Error> {
        let _ = self.soc.step_breakpoint(NextFrame::new())?;
        Ok(())
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

    fn boot(&mut self) -> Result<(), Error> {
        self.boot_memory()?;
        self.boot_cpu();
        Ok(())
    }

    fn boot_cpu(&mut self) {
        let cpu = self.soc.cpu.as_mut().unwrap();

        cpu.registers_mut().set_af(0x01b0);
        cpu.registers_mut().set_bc(0x0013);
        cpu.registers_mut().set_de(0x00d8);
        cpu.registers_mut().set_hl(0x014d);
        cpu.registers_mut().sp = 0xfffe;
        cpu.registers_mut().pc = 0x0100;

        if cfg!(feature = "cgb") {
            cpu.registers_mut().a = 0x11;
        }
    }

    fn boot_memory(&mut self) -> Result<(), Error> {
        let soc = &mut self.soc;

        <LR35902<C, O> as MemoryBus>::write(soc, 0xff05, 0x00)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff06, 0x00)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff07, 0x00)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff10, 0x80)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff11, 0xbf)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff12, 0xf3)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff14, 0xbf)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff16, 0x3f)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff17, 0x00)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff19, 0xbf)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff1a, 0x7f)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff1b, 0xff)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff1c, 0x9f)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff1e, 0xbf)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff20, 0xff)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff21, 0x00)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff22, 0x00)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff23, 0xbf)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff24, 0x77)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff25, 0xf3)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff26, 0xf1)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff40, 0x91)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff42, 0x00)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff43, 0x00)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff45, 0x00)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff47, 0xfc)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff48, 0xff)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff49, 0xff)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff4a, 0x00)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff4b, 0x00)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xffff, 0x00)?;
        <LR35902<C, O> as MemoryBus>::write(soc, 0xff50, 0x01)?;
        Ok(())
    }
}
