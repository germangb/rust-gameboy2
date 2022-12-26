include!("integration_test.rs");

const DEFAULT_SECONDS: u32 = 10;

#[test]
fn read_timing() {
    integration_test(
        include_bytes!("gb-test-roms/mem_timing/individual/01-read_timing.gb"),
        todo!(),
        DEFAULT_SECONDS,
    );
}

#[test]
fn write_timing() {
    integration_test(
        include_bytes!("gb-test-roms/mem_timing/individual/02-write_timing.gb"),
        todo!(),
        DEFAULT_SECONDS,
    );
}

#[test]
fn modify_timing() {
    integration_test(
        include_bytes!("gb-test-roms/mem_timing/individual/03-modify_timing.gb"),
        todo!(),
        DEFAULT_SECONDS,
    );
}
