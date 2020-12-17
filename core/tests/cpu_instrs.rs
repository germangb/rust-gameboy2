include!("integration_test.rs");

const DEFAULT_SECONDS: u32 = 10;

#[test]
fn special() {
    integration_test(
        include_bytes!("gb-test-roms/cpu_instrs/individual/01-special.gb"),
        include_bytes!("cpu_instrs/01-special.bin"),
        DEFAULT_SECONDS,
    );
}

#[test]
fn interrupts() {
    integration_test(
        include_bytes!("gb-test-roms/cpu_instrs/individual/02-interrupts.gb"),
        include_bytes!("cpu_instrs/02-interrupts.bin"),
        DEFAULT_SECONDS,
    );
}

#[test]
fn op_sp_hl() {
    integration_test(
        include_bytes!("gb-test-roms/cpu_instrs/individual/03-op sp,hl.gb"),
        include_bytes!("cpu_instrs/03-op sp,hl.bin"),
        DEFAULT_SECONDS,
    );
}

#[test]
fn op_r_imm() {
    integration_test(
        include_bytes!("gb-test-roms/cpu_instrs/individual/04-op r,imm.gb"),
        include_bytes!("cpu_instrs/04-op r,imm.bin"),
        DEFAULT_SECONDS,
    );
}

#[test]
fn op_rp() {
    integration_test(
        include_bytes!("gb-test-roms/cpu_instrs/individual/05-op rp.gb"),
        include_bytes!("cpu_instrs/05-op rp.bin"),
        DEFAULT_SECONDS,
    );
}

#[test]
fn ld_r_r() {
    integration_test(
        include_bytes!("gb-test-roms/cpu_instrs/individual/06-ld r,r.gb"),
        include_bytes!("cpu_instrs/06-ld r,r.bin"),
        DEFAULT_SECONDS,
    );
}

#[test]
fn jr_jp_call_ret_rst() {
    integration_test(
        include_bytes!("gb-test-roms/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb"),
        include_bytes!("cpu_instrs/07-jr,jp,call,ret,rst.bin"),
        DEFAULT_SECONDS,
    );
}

#[test]
fn misc_instrs() {
    integration_test(
        include_bytes!("gb-test-roms/cpu_instrs/individual/08-misc instrs.gb"),
        include_bytes!("cpu_instrs/08-misc instrs.bin"),
        DEFAULT_SECONDS,
    );
}

#[test]
fn op_r_r() {
    integration_test(
        include_bytes!("gb-test-roms/cpu_instrs/individual/09-op r,r.gb"),
        include_bytes!("cpu_instrs/09-op r,r.bin"),
        DEFAULT_SECONDS * 2,
    );
}

#[test]
fn bit_ops() {
    integration_test(
        include_bytes!("gb-test-roms/cpu_instrs/individual/10-bit ops.gb"),
        include_bytes!("cpu_instrs/10-bit ops.bin"),
        DEFAULT_SECONDS * 2,
    );
}

#[test]
fn op_a_hl() {
    integration_test(
        include_bytes!("gb-test-roms/cpu_instrs/individual/11-op a,(hl).gb"),
        include_bytes!("cpu_instrs/11-op a,(hl).bin"),
        DEFAULT_SECONDS * 3,
    );
}
