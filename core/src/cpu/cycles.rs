#![allow(clippy::all)]

const UNDEF: u16 = 0x00;

macro_rules! branch {
    ($l:expr, $r:expr) => {
        ($l << 8) | $r
    };
}

#[rustfmt::skip]
const UNPREFIXED: &[u16; 256] = &[
    //       x0,             x1, x2,              x3,    x4,              x5, x6, x7, x8,             x9, xA,              xB,    xC,              xD,    xE, xF
    /* 0x */ 4_,             12, 8_,              8_,    4_,              4_, 8_, 4_, 20,             8_, 8_,              8_,    4_,              4_,    8_, 4_,
    /* 1x */ 4_,             12, 8_,              8_,    4_,              4_, 8_, 4_, 12,             8_, 8_,              8_,    4_,              4_,    8_, 4_,
    /* 2x */ branch!(12, 8), 12, 8_,              8_,    4_,              4_, 8_, 4_, branch!(12, 8), 8_, 8_,              8_,    4_,              4_,    8_, 4_,
    /* 3x */ branch!(12, 8), 12, 8_,              8_,    12,              12, 12, 4_, branch!(12, 8), 8_, 8_,              8_,    4_,              4_,    8_, 4_,
    /* 4x */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* 5x */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* 6x */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* 7x */ 8_,             8_, 8_,              8_,    8_,              8_, 4_, 8_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* 8x */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* 9x */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* Ax */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* Bx */ 4_,             4_, 4_,              4_,    4_,              4_, 8_, 4_, 4_,             4_, 4_,              4_,    4_,              4_,    8_, 4_,
    /* Cx */ branch!(20, 8), 12, branch!(16, 12), 16,    branch!(24, 12), 16, 8_, 16, branch!(20, 8), 16, branch!(16, 12), 4_,    branch!(24, 12), 24,    8_, 16,
    /* Dx */ branch!(20, 8), 12, branch!(16, 12), UNDEF, branch!(24, 12), 16, 8_, 16, branch!(20, 8), 16, branch!(16, 12), UNDEF, branch!(24, 12), UNDEF, 8_, 16,
    /* Ex */ 12,             12, 8_,              UNDEF, UNDEF,           16, 8_, 16, 16,             4_, 16,              UNDEF, UNDEF,           UNDEF, 8_, 16,
    /* Fx */ 12,             12, 8_,              4_,    4_,              16, 8_, 16, 12,             8_, 16,              4_,    UNDEF,           UNDEF, 8_, 16,
];

/// Return the number of CPU clock cycles that correspond to executing the given
/// opcode. If `branch == true` and the instruction is is a branch instruction,
/// return the number of cycles if the branch was taken.
pub fn unprefixed(opcode: u8, branch: bool) -> u64 {
    let mut cycles = UNPREFIXED[opcode as usize];
    if branch {
        cycles >>= 8;
    }
    let cycles = cycles & 0xff;
    assert_ne!(cycles, UNDEF);
    cycles as _
}

#[rustfmt::skip]
const PREFIXED: &[u16; 256] = &[
    //       x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, xA, xB, xC, xD, xE, xF
    /* 0x */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* 1x */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* 2x */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* 3x */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* 4x */ 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_,
    /* 5x */ 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_,
    /* 6x */ 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_,
    /* 7x */ 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 12, 8_,
    /* 8x */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* 9x */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* Ax */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* Bx */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* Cx */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* Dx */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* Ex */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
    /* Fx */ 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_, 8_, 8_, 8_, 8_, 8_, 8_, 16, 8_,
];

/// Same as `unprefixed()` function but for the `CB` prefixed instructions.
pub fn prefixed(opcode: u8) -> u64 {
    let cycles = PREFIXED[opcode as usize];
    assert_ne!(cycles, UNDEF);
    cycles as _
}
