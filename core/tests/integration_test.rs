// run test rom for the given amount of seconds, before checking the state of
// the display that determines the test result.
fn integration_test(name: &str, rom: &[u8], ground_truth: &[u8], seconds: u32) {
    use core::{
        cartridge::ROM,
        lcd::{HEIGHT, WIDTH},
        GameBoy,
    };
    use image::{ImageBuffer, Rgba};
    use std::path::PathBuf;

    let rom = ROM::new(rom.to_vec());
    let mut gb = GameBoy::new(rom);

    gb.skip_boot();

    for _ in 0..60 * seconds {
        gb.update_frame();
    }

    let display: &[u8; WIDTH * HEIGHT * 4] = unsafe { std::mem::transmute(gb.display()) };

    // save image to disk
    #[cfg(todo)]
    if ground_truth != &display[..] {
        let image: ImageBuffer<Rgba<u8>, Vec<_>> =
            ImageBuffer::from_vec(WIDTH as _, HEIGHT as _, display.to_vec())
                .expect("Error creating buffer image");

        let mut path: PathBuf = ["/tmp/", name].iter().collect();
        path.set_extension("jpg");

        image
            .save_with_format(path, image::ImageFormat::Jpeg)
            .expect("Error saving screenshot");
    }

    assert_eq!(ground_truth, &display[..]);
}
