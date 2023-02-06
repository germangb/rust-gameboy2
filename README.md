# `rust-gameboy2`

[![Build Status](https://travis-ci.org/germangb/rust-gameboy2.svg?branch=main)](https://travis-ci.org/germangb/rust-gameboy2)

Attempt to rewrite the old [`germangb/rust-gameboy`] emulator project.

[`germangb/rust-gameboy`]: https://github.com/germangb/rust-gameboy

![](assets/ferris.png)
![](assets/batman.png)
![](assets/camera.png)
![](assets/doraemon.png)

![](assets/gold_cgb.png)
![](assets/simpsons.png)
![](assets/zelda_cgb.png)
![](assets/mario_deluxe.png)

## Automated tests

```bash
cargo test --test cpu_instrs
cargo test --test instr_timing
cargo test --test mem_timing
```

![](assets/cpu_instrs.png)
![](assets/instr_timing.png)
![](assets/mem_timing.png)

## Build

### Boot ROMs

You must provide your own boot ROMs as they are not included in the repo.

- `/core/boot/boot.gb`
- `/core/boot/boot.gbc` (if building with `--features cgb`)

You may or may not find them here https://gbdev.gg8.se/files/roms/bootroms/

### Native build

```bash
cargo run -p native --release [--features cgb] -- [ROM FILE]
```

Focus on the LCD window for game controls:

- `Left`, `Right`, `Up`, `Down` maps to DPAD buttons.
- `Z` maps to A button
- `X` maps to B button
- `Enter` maps to Start button
- `RightShift` maps to Select button

Other keyboard controls (for primitive debugging):

- `C` Change the ROM (Will open filesystem file selector).
- `P` Pause/Resume emulation
- `R` Reset emulation
- `S` Step instruction (CPU Window)
- `B` Set Instruction breakpoint (CPU Window)
- `L` Set LCD line breakpoint (CPU Window)
- `RightShift + P` Override PC register (CPU Window)

(The Memory window `--features mem` is not yet finished)

- `R` Read byte from memory (MEM Window)
- `RightShift + P` Write byte to memory (MEM Window)

### WASM

```bash
cd wasm/
wasm-pack build [--features cgb] # build NPM package
cd www/
npm run start # start HTTP server
```

![](assets/wasm.png)

## References

- https://fosdem.org/2023/schedule/event/gb_arm/
- http://problemkaputt.de/pandocs.htm
- https://gbdev.gg8.se/wiki/
- https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf
- https://gekkio.fi/files/gb-docs/gbctr.pdf
- https://github.com/gbdev/awesome-gbdev
- https://github.com/AntonioND/gbcam-rev-engineer/tree/master/doc
