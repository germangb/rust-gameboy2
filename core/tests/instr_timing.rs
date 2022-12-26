include!("integration_test.rs");

const DEFAULT_SECONDS: u32 = 4;

#[test]
fn instr_timing() {
    integration_test(
        include_bytes!("gb-test-roms/instr_timing/instr_timing.gb"),
        include_bytes!("instr_timing/instr_timing.bin"),
        DEFAULT_SECONDS,
    );
}
