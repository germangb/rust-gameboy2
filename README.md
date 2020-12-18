# `rust-gameboy2`

[![Build Status](https://travis-ci.org/germangb/rust-gameboy2.svg?branch=main)](https://travis-ci.org/germangb/rust-gameboy2)

Attempt to rewrite the former [`germangb/rust-gameboy`] emulator project.

[`germangb/rust-gameboy`]: https://github.com/germangb/rust-gameboy

![](assets/zelda.png)
![](assets/batman.png)
![](assets/camera.png)
![](assets/doraemon.png)

## Integration tests

```bash
cargo test --test cpu_instrs
cargo test --test instr_timing
cargo test --test mem_timing
```

![](assets/cpu_instrs.png)
![](assets/instr_timing.png)
![](assets/mem_timing.png)

## References

- http://problemkaputt.de/pandocs.htm
- https://gbdev.gg8.se/wiki/
- https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf
- https://gekkio.fi/files/gb-docs/gbctr.pdf
- https://github.com/gbdev/awesome-gbdev
- https://github.com/AntonioND/gbcam-rev-engineer/tree/master/doc