// run test rom for the given amount of seconds, before checking the state of
// the display, which determines the test result.
fn integration_test(rom: &[u8], ground_truth: &[u8], seconds: u32) {
    use core::{
        cartridge::ROM,
        lcd::{HEIGHT, WIDTH},
        GameBoy,
    };

    let rom = ROM::new(rom.to_vec());
    let mut gb = GameBoy::new(rom).unwrap();

    gb.skip_boot().unwrap();

    for _ in 0..60 * seconds {
        gb.update_frame().unwrap();
    }

    let display: &[u8; WIDTH * HEIGHT * 4] = unsafe { std::mem::transmute(gb.display()) };

    assert_eq!(ground_truth, &display[..]);
}
