use crate::dasm::PrefixRegister::HLPtr;
use parse_display::Display;
use std::io::{Bytes, Read};

#[rustfmt::skip]
const OPCODE_LEN: &[usize; 256] = &[
    1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1,
    1, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 1, 3, 3, 2, 1,
    1, 1, 3, 0, 3, 1, 2, 1, 1, 1, 3, 0, 3, 0, 2, 1,
    2, 1, 1, 0, 0, 1, 2, 1, 2, 1, 3, 0, 0, 0, 2, 1,
    2, 1, 1, 1, 0, 1, 2, 1, 2, 1, 3, 1, 0, 0, 2, 1,
];

/// Immediate address for LD Opcodes.
#[derive(Display, Debug, Eq, PartialEq)]
pub enum Address {
    #[display("({0:04X})")]
    U16(u16),
    /// 0xff00 + u8
    #[display("(FF00+{0:02X})")]
    U8(u8),
    #[display("(HL)")]
    HLPtr,
}

#[derive(Display, Debug, Eq, PartialEq)]
pub enum Data {
    #[display("{0:04X}")]
    U16(u16),
    #[display("{0:02X}")]
    U8(u8),
    #[display("{0:02X}")]
    I8(u8),
}

/// Set of registers used in arithmetic and logic opcodes.
#[derive(Display, Debug, Eq, PartialEq)]
pub enum ALUSrc {
    A,
    B,
    C,
    BC,
    D,
    E,
    DE,
    H,
    L,
    HL,
    #[display("(HL)")]
    HLPtr,
    #[display("{0}")]
    Data(Data),
    SP,
}

/// Used to store arithmetic and logic opcode results.
/// Normally it's always A, but some opcodes store the result in SP.
#[derive(Display, Debug, Eq, PartialEq)]
pub enum ALUDst {
    A,
    B,
    C,
    BC,
    D,
    E,
    DE,
    H,
    L,
    HL,
    #[display("(HL)")]
    HLPtr,
    SP,
}

/// Registers involved in stack Opcodes.
#[derive(Display, Debug, Eq, PartialEq)]
pub enum StackRegister {
    BC,
    DE,
    HL,
    AF,
}

#[derive(Display, Debug, Eq, PartialEq)]
pub enum Bit {
    #[display("0")]
    B0,
    #[display("1")]
    B1,
    #[display("2")]
    B2,
    #[display("3")]
    B3,
    #[display("4")]
    B4,
    #[display("5")]
    B5,
    #[display("6")]
    B6,
    #[display("7")]
    B7,
}

/// RST Opcode addresses.
#[derive(Display, Debug, Eq, PartialEq)]
pub enum RSTAddress {
    #[display("00H")]
    H00,
    #[display("10H")]
    H10,
    #[display("20H")]
    H20,
    #[display("30H")]
    H30,
    #[display("08H")]
    H08,
    #[display("18H")]
    H18,
    #[display("28H")]
    H28,
    #[display("38H")]
    H38,
}

#[derive(Display, Debug, Eq, PartialEq)]
pub enum Flags {
    Z,
    C,
    NZ,
    NC,
}

/// Source of LD Opcode.
#[derive(Display, Debug, Eq, PartialEq)]
pub enum LDSrc {
    A,
    B,
    C,
    #[display("(C)")]
    CPtr,
    #[display("(BC)")]
    BCPtr,
    D,
    E,
    #[display("(DE)")]
    DEPtr,
    H,
    L,
    #[display("(HL)")]
    HLPtr,
    #[display("(HL+)")]
    HLPtrInc,
    #[display("(HL-)")]
    HLPtrDec,
    HL,
    SP,
    #[display("SP+{0:02X}")]
    SPOffset(u8),
    #[display("{0}")]
    Data(Data),
    #[display("{0}")]
    Address(Address),
}

/// Destination of LD Opcode.
#[derive(Display, Debug, Eq, PartialEq)]
pub enum LDDst {
    A,
    B,
    C,
    BC,
    #[display("(C)")]
    CPtr,
    #[display("(BC)")]
    BCPtr,
    D,
    E,
    DE,
    #[display("(DE)")]
    DEPtr,
    H,
    L,
    HL,
    #[display("(HL)")]
    HLPtr,
    #[display("(HL+)")]
    HLPtrInc,
    #[display("(HL-)")]
    HLPtrDec,
    SP,
    #[display("{0}")]
    Address(Address),
}

/// GameBoy CPU Opcodes.
///
/// ```
/// use utils::dasm::{Data, LDDst, LDSrc, Opcode};
///
/// fn display(opcode: Opcode) -> String {
///     format!("{}", opcode)
/// }
///
/// assert_eq!("NOP", display(Opcode::NOP));
/// assert_eq!("LD HL,FF40h", display(Opcode::LD(LDDst::HL, LDSrc::Data(Data::U16(0xff40)))));
/// assert_eq!("LD A,(HL)", display(Opcode::LD(LDDst::A, LDSrc::HLPtr)));
/// ```
#[derive(Display, Debug, Eq, PartialEq)]
pub enum Opcode {
    #[display("<unknown>")]
    Unknown,

    NOP,
    #[display("STOP 0")]
    STOP,
    #[display("HALT")]
    HALT,
    DI,
    EI,
    #[display("CB {0}")]
    PrefixOpcode(PrefixOpcode),

    #[display("LD {0},{1}")]
    LD(LDDst, LDSrc),

    DAA,
    SCF,
    CPL,
    CCF,
    #[display("INC {0}")]
    INC(ALUDst),
    #[display("DEC {0}")]
    DEC(ALUDst),
    #[display("ADD {0},{1}")]
    ADD(ALUDst, ALUSrc),
    #[display("ADC A,{0}")]
    ADC(ALUSrc),
    #[display("DUB {0}")]
    SUB(ALUSrc),
    #[display("DBC A,{0}")]
    SBC(ALUSrc),
    #[display("AND {0}")]
    AND(ALUSrc),
    #[display("XOR {0}")]
    XOR(ALUSrc),
    #[display("OR {0}")]
    OR(ALUSrc),
    #[display("CP {0}")]
    CP(ALUSrc),

    RLCA,
    RRCA,
    RLA,
    RRA,

    #[display("PUSH {0}")]
    PUSH(StackRegister),
    #[display("POP {0}")]
    POP(StackRegister),

    #[display("JP {0}")]
    JP(Address),
    #[display("JP {0},{1}")]
    JPFlags(Flags, Address),
    #[display("JR {0:02X}")]
    JR(u8),
    #[display("JR {0},{1:02X}")]
    JRFlags(Flags, u8),
    #[display("CALL {0}")]
    CALL(Address),
    #[display("CALL {0},{1}")]
    CALLFlags(Flags, Address),
    RET,
    #[display("RET {0}")]
    RETFlags(Flags),
    RETI,
    #[display("RST {0}")]
    RST(RSTAddress),
}

/// Registers used in 0xCB prefixed Opcodes.
#[derive(Display, Debug, Eq, PartialEq, Copy, Clone)]
pub enum PrefixRegister {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    #[display("(HL)")]
    HLPtr,
}

/// 0xCB prefixed Opcodes.
#[derive(Display, Debug, Eq, PartialEq)]
pub enum PrefixOpcode {
    #[display("RLC {0}")]
    RLC(PrefixRegister),
    #[display("RRC {0}")]
    RRC(PrefixRegister),
    #[display("RL {0}")]
    RL(PrefixRegister),
    #[display("RR {0}")]
    RR(PrefixRegister),
    #[display("SLA {0}")]
    SLA(PrefixRegister),
    #[display("SRA {0}")]
    SRA(PrefixRegister),
    #[display("SRL {0}")]
    SRL(PrefixRegister),
    #[display("SWAP {0}")]
    SWAP(PrefixRegister),
    #[display("BIT {0},{1}")]
    BIT(Bit, PrefixRegister),
    #[display("RES {0},{1}")]
    RES(Bit, PrefixRegister),
    #[display("SET {0},{1}")]
    SET(Bit, PrefixRegister),
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum Error {
    #[error("Decoding error")]
    /// Error decoding current opcode.
    DecodeError,
}

#[derive(Debug)]
pub struct Disassembler<'input> {
    ended: bool,
    input: Bytes<&'input [u8]>,
}

impl<'input> Disassembler<'input> {
    pub fn new(input: &'input [u8]) -> Self {
        let input = input.bytes();
        Self {
            ended: false,
            input,
        }
    }

    fn fetch(&mut self) -> Result<u8, Error> {
        match self.input.next() {
            Some(Ok(opcode)) => Ok(opcode),
            None | Some(Err(_)) => Err(Error::DecodeError),
        }
    }

    fn fetch_u16(&mut self) -> Result<u16, Error> {
        let lo = self.fetch()? as u16;
        let hi = self.fetch()? as u16;
        Ok(lo | (hi << 8))
    }

    fn next_opcode(&mut self, opcode: u8) -> Result<Opcode, Error> {
        use Opcode::*;
        match opcode {
            0x00..=0x7f => match opcode {
                0x00..=0x3f => match opcode {
                    0x00..=0x1f => match opcode {
                        0x00..=0x0f => match opcode {
                            0x00 => Ok(NOP),
                            0x01 => Ok(LD(LDDst::BC, LDSrc::Data(Data::U16(self.fetch_u16()?)))),
                            0x02 => Ok(LD(LDDst::BCPtr, LDSrc::A)),
                            0x03 => Ok(INC(ALUDst::BC)),
                            0x04 => Ok(INC(ALUDst::B)),
                            0x05 => Ok(DEC(ALUDst::B)),
                            0x06 => Ok(LD(LDDst::B, LDSrc::Data(Data::U8(self.fetch()?)))),
                            0x07 => Ok(RLCA),
                            0x08 => Ok(LD(
                                LDDst::Address(Address::U16(self.fetch_u16()?)),
                                LDSrc::SP,
                            )),
                            0x09 => Ok(ADD(ALUDst::HL, ALUSrc::BC)),
                            0x0a => Ok(LD(LDDst::A, LDSrc::BCPtr)),
                            0x0b => Ok(DEC(ALUDst::BC)),
                            0x0c => Ok(INC(ALUDst::C)),
                            0x0d => Ok(DEC(ALUDst::C)),
                            0x0e => Ok(LD(LDDst::C, LDSrc::Data(Data::U8(self.fetch()?)))),
                            0x0f => Ok(RRCA),
                            _ => unreachable!(),
                        },
                        0x10..=0x1f => match opcode {
                            0x10 => Ok(STOP),
                            0x11 => Ok(LD(LDDst::DE, LDSrc::Data(Data::U16(self.fetch_u16()?)))),
                            0x12 => Ok(LD(LDDst::DEPtr, LDSrc::A)),
                            0x13 => Ok(INC(ALUDst::DE)),
                            0x14 => Ok(INC(ALUDst::D)),
                            0x15 => Ok(DEC(ALUDst::D)),
                            0x16 => Ok(LD(LDDst::D, LDSrc::Data(Data::U8(self.fetch()?)))),
                            0x17 => Ok(RLA),
                            0x18 => Ok(JR(self.fetch()?)),
                            0x19 => Ok(ADD(ALUDst::HL, ALUSrc::DE)),
                            0x1a => Ok(LD(LDDst::A, LDSrc::DEPtr)),
                            0x1b => Ok(DEC(ALUDst::DE)),
                            0x1c => Ok(INC(ALUDst::E)),
                            0x1d => Ok(DEC(ALUDst::E)),
                            0x1e => Ok(LD(LDDst::E, LDSrc::Data(Data::U8(self.fetch()?)))),
                            0x1f => Ok(RRA),
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    },
                    0x20..=0x3f => match opcode {
                        0x20..=0x2f => match opcode {
                            0x20 => Ok(JRFlags(Flags::NZ, self.fetch()?)),
                            0x21 => Ok(LD(LDDst::HL, LDSrc::Data(Data::U16(self.fetch_u16()?)))),
                            0x22 => Ok(LD(LDDst::HLPtrInc, LDSrc::A)),
                            0x23 => Ok(INC(ALUDst::HL)),
                            0x24 => Ok(INC(ALUDst::H)),
                            0x25 => Ok(DEC(ALUDst::H)),
                            0x26 => Ok(LD(LDDst::H, LDSrc::Data(Data::U8(self.fetch()?)))),
                            0x27 => Ok(DAA),
                            0x28 => Ok(JRFlags(Flags::Z, self.fetch()?)),
                            0x29 => Ok(ADD(ALUDst::HL, ALUSrc::HL)),
                            0x2a => Ok(LD(LDDst::A, LDSrc::HLPtrInc)),
                            0x2b => Ok(DEC(ALUDst::HL)),
                            0x2c => Ok(INC(ALUDst::L)),
                            0x2d => Ok(DEC(ALUDst::L)),
                            0x2e => Ok(LD(LDDst::L, LDSrc::Data(Data::U8(self.fetch()?)))),
                            0x2f => Ok(CPL),
                            _ => unreachable!(),
                        },
                        0x30..=0x3f => match opcode {
                            0x30 => Ok(JRFlags(Flags::NC, self.fetch()?)),
                            0x31 => Ok(LD(LDDst::SP, LDSrc::Data(Data::U16(self.fetch_u16()?)))),
                            0x32 => Ok(LD(LDDst::HLPtrDec, LDSrc::A)),
                            0x33 => Ok(INC(ALUDst::SP)),
                            0x34 => Ok(INC(ALUDst::HLPtr)),
                            0x35 => Ok(DEC(ALUDst::HLPtr)),
                            0x36 => Ok(LD(LDDst::HLPtr, LDSrc::Data(Data::U8(self.fetch()?)))),
                            0x37 => Ok(SCF),
                            0x38 => Ok(JRFlags(Flags::C, self.fetch()?)),
                            0x39 => Ok(ADD(ALUDst::HL, ALUSrc::SP)),
                            0x3a => Ok(LD(LDDst::A, LDSrc::HLPtrDec)),
                            0x3b => Ok(DEC(ALUDst::SP)),
                            0x3c => Ok(INC(ALUDst::A)),
                            0x3d => Ok(DEC(ALUDst::A)),
                            0x3e => Ok(LD(LDDst::A, LDSrc::Data(Data::U8(self.fetch()?)))),
                            0x3f => Ok(CCF),
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                },
                0x40..=0x7f => match opcode {
                    0x40..=0x5f => match opcode {
                        0x40..=0x4f => match opcode {
                            0x40 => Ok(LD(LDDst::B, LDSrc::B)),
                            0x41 => Ok(LD(LDDst::B, LDSrc::C)),
                            0x42 => Ok(LD(LDDst::B, LDSrc::D)),
                            0x43 => Ok(LD(LDDst::B, LDSrc::E)),
                            0x44 => Ok(LD(LDDst::B, LDSrc::H)),
                            0x45 => Ok(LD(LDDst::B, LDSrc::L)),
                            0x46 => Ok(LD(LDDst::B, LDSrc::HLPtr)),
                            0x47 => Ok(LD(LDDst::B, LDSrc::A)),
                            0x48 => Ok(LD(LDDst::C, LDSrc::B)),
                            0x49 => Ok(LD(LDDst::C, LDSrc::C)),
                            0x4a => Ok(LD(LDDst::C, LDSrc::D)),
                            0x4b => Ok(LD(LDDst::C, LDSrc::E)),
                            0x4c => Ok(LD(LDDst::C, LDSrc::H)),
                            0x4d => Ok(LD(LDDst::C, LDSrc::L)),
                            0x4e => Ok(LD(LDDst::C, LDSrc::HLPtr)),
                            0x4f => Ok(LD(LDDst::C, LDSrc::A)),
                            _ => unreachable!(),
                        },
                        0x50..=0x5f => match opcode {
                            0x50 => Ok(LD(LDDst::D, LDSrc::B)),
                            0x51 => Ok(LD(LDDst::D, LDSrc::C)),
                            0x52 => Ok(LD(LDDst::D, LDSrc::D)),
                            0x53 => Ok(LD(LDDst::D, LDSrc::E)),
                            0x54 => Ok(LD(LDDst::D, LDSrc::H)),
                            0x55 => Ok(LD(LDDst::D, LDSrc::L)),
                            0x56 => Ok(LD(LDDst::D, LDSrc::HLPtr)),
                            0x57 => Ok(LD(LDDst::D, LDSrc::A)),
                            0x58 => Ok(LD(LDDst::E, LDSrc::B)),
                            0x59 => Ok(LD(LDDst::E, LDSrc::C)),
                            0x5a => Ok(LD(LDDst::E, LDSrc::D)),
                            0x5b => Ok(LD(LDDst::E, LDSrc::E)),
                            0x5c => Ok(LD(LDDst::E, LDSrc::H)),
                            0x5d => Ok(LD(LDDst::E, LDSrc::L)),
                            0x5e => Ok(LD(LDDst::E, LDSrc::HLPtr)),
                            0x5f => Ok(LD(LDDst::E, LDSrc::A)),
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    },
                    0x60..=0x7f => match opcode {
                        0x60..=0x6f => match opcode {
                            0x60 => Ok(LD(LDDst::H, LDSrc::B)),
                            0x61 => Ok(LD(LDDst::H, LDSrc::C)),
                            0x62 => Ok(LD(LDDst::H, LDSrc::D)),
                            0x63 => Ok(LD(LDDst::H, LDSrc::E)),
                            0x64 => Ok(LD(LDDst::H, LDSrc::H)),
                            0x65 => Ok(LD(LDDst::H, LDSrc::L)),
                            0x66 => Ok(LD(LDDst::H, LDSrc::HLPtr)),
                            0x67 => Ok(LD(LDDst::H, LDSrc::A)),
                            0x68 => Ok(LD(LDDst::L, LDSrc::B)),
                            0x69 => Ok(LD(LDDst::L, LDSrc::C)),
                            0x6a => Ok(LD(LDDst::L, LDSrc::D)),
                            0x6b => Ok(LD(LDDst::L, LDSrc::E)),
                            0x6c => Ok(LD(LDDst::L, LDSrc::H)),
                            0x6d => Ok(LD(LDDst::L, LDSrc::L)),
                            0x6e => Ok(LD(LDDst::L, LDSrc::HLPtr)),
                            0x6f => Ok(LD(LDDst::L, LDSrc::A)),
                            _ => unreachable!(),
                        },
                        0x70..=0x7f => match opcode {
                            0x70 => Ok(LD(LDDst::HLPtr, LDSrc::B)),
                            0x71 => Ok(LD(LDDst::HLPtr, LDSrc::C)),
                            0x72 => Ok(LD(LDDst::HLPtr, LDSrc::D)),
                            0x73 => Ok(LD(LDDst::HLPtr, LDSrc::E)),
                            0x74 => Ok(LD(LDDst::HLPtr, LDSrc::H)),
                            0x75 => Ok(LD(LDDst::HLPtr, LDSrc::L)),
                            0x76 => Ok(HALT),
                            0x77 => Ok(LD(LDDst::HLPtr, LDSrc::A)),
                            0x78 => Ok(LD(LDDst::A, LDSrc::B)),
                            0x79 => Ok(LD(LDDst::A, LDSrc::C)),
                            0x7a => Ok(LD(LDDst::A, LDSrc::D)),
                            0x7b => Ok(LD(LDDst::A, LDSrc::E)),
                            0x7c => Ok(LD(LDDst::A, LDSrc::H)),
                            0x7d => Ok(LD(LDDst::A, LDSrc::L)),
                            0x7e => Ok(LD(LDDst::A, LDSrc::HLPtr)),
                            0x7f => Ok(LD(LDDst::A, LDSrc::A)),
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            },
            0x80..=0xff => match opcode {
                0x80..=0xbf => match opcode {
                    0x80..=0x9f => match opcode {
                        0x80..=0x8f => match opcode {
                            0x80 => Ok(ADD(ALUDst::A, ALUSrc::B)),
                            0x81 => Ok(ADD(ALUDst::A, ALUSrc::C)),
                            0x82 => Ok(ADD(ALUDst::A, ALUSrc::D)),
                            0x83 => Ok(ADD(ALUDst::A, ALUSrc::E)),
                            0x84 => Ok(ADD(ALUDst::A, ALUSrc::H)),
                            0x85 => Ok(ADD(ALUDst::A, ALUSrc::L)),
                            0x86 => Ok(ADD(ALUDst::A, ALUSrc::HLPtr)),
                            0x87 => Ok(ADD(ALUDst::A, ALUSrc::A)),
                            0x88 => Ok(ADC(ALUSrc::B)),
                            0x89 => Ok(ADC(ALUSrc::C)),
                            0x8a => Ok(ADC(ALUSrc::D)),
                            0x8b => Ok(ADC(ALUSrc::E)),
                            0x8c => Ok(ADC(ALUSrc::H)),
                            0x8d => Ok(ADC(ALUSrc::L)),
                            0x8e => Ok(ADC(ALUSrc::HLPtr)),
                            0x8f => Ok(ADC(ALUSrc::A)),
                            _ => unreachable!(),
                        },
                        0x90..=0x9f => match opcode {
                            0x90 => Ok(SUB(ALUSrc::B)),
                            0x91 => Ok(SUB(ALUSrc::C)),
                            0x92 => Ok(SUB(ALUSrc::D)),
                            0x93 => Ok(SUB(ALUSrc::E)),
                            0x94 => Ok(SUB(ALUSrc::H)),
                            0x95 => Ok(SUB(ALUSrc::L)),
                            0x96 => Ok(SUB(ALUSrc::HLPtr)),
                            0x97 => Ok(SUB(ALUSrc::A)),
                            0x98 => Ok(SBC(ALUSrc::B)),
                            0x99 => Ok(SBC(ALUSrc::C)),
                            0x9a => Ok(SBC(ALUSrc::D)),
                            0x9b => Ok(SBC(ALUSrc::E)),
                            0x9c => Ok(SBC(ALUSrc::H)),
                            0x9d => Ok(SBC(ALUSrc::L)),
                            0x9e => Ok(SBC(ALUSrc::HLPtr)),
                            0x9f => Ok(SBC(ALUSrc::A)),
                            _ => unreachable!(),
                        },
                        _ => unimplemented!(),
                    },
                    0xa0..=0xbf => match opcode {
                        0xa0..=0xaf => match opcode {
                            0xa0 => Ok(AND(ALUSrc::B)),
                            0xa1 => Ok(AND(ALUSrc::C)),
                            0xa2 => Ok(AND(ALUSrc::D)),
                            0xa3 => Ok(AND(ALUSrc::E)),
                            0xa4 => Ok(AND(ALUSrc::H)),
                            0xa5 => Ok(AND(ALUSrc::L)),
                            0xa6 => Ok(AND(ALUSrc::HLPtr)),
                            0xa7 => Ok(AND(ALUSrc::A)),
                            0xa8 => Ok(XOR(ALUSrc::B)),
                            0xa9 => Ok(XOR(ALUSrc::C)),
                            0xaa => Ok(XOR(ALUSrc::D)),
                            0xab => Ok(XOR(ALUSrc::E)),
                            0xac => Ok(XOR(ALUSrc::H)),
                            0xad => Ok(XOR(ALUSrc::L)),
                            0xae => Ok(XOR(ALUSrc::HLPtr)),
                            0xaf => Ok(XOR(ALUSrc::A)),
                            _ => unimplemented!(),
                        },
                        0xb0..=0xbf => match opcode {
                            0xb0 => Ok(OR(ALUSrc::B)),
                            0xb1 => Ok(OR(ALUSrc::C)),
                            0xb2 => Ok(OR(ALUSrc::D)),
                            0xb3 => Ok(OR(ALUSrc::E)),
                            0xb4 => Ok(OR(ALUSrc::H)),
                            0xb5 => Ok(OR(ALUSrc::L)),
                            0xb6 => Ok(OR(ALUSrc::HLPtr)),
                            0xb7 => Ok(OR(ALUSrc::A)),
                            0xb8 => Ok(CP(ALUSrc::B)),
                            0xb9 => Ok(CP(ALUSrc::C)),
                            0xba => Ok(CP(ALUSrc::D)),
                            0xbb => Ok(CP(ALUSrc::E)),
                            0xbc => Ok(CP(ALUSrc::H)),
                            0xbd => Ok(CP(ALUSrc::L)),
                            0xbe => Ok(CP(ALUSrc::HLPtr)),
                            0xbf => Ok(CP(ALUSrc::A)),
                            _ => unimplemented!(),
                        },
                        _ => unimplemented!(),
                    },
                    _ => unreachable!(),
                },
                0xc0..=0xff => match opcode {
                    0xc0..=0xdf => match opcode {
                        0xc0..=0xcf => match opcode {
                            0xc0 => Ok(RETFlags(Flags::NZ)),
                            0xc1 => Ok(POP(StackRegister::BC)),
                            0xc2 => Ok(JPFlags(Flags::NZ, Address::U16(self.fetch_u16()?))),
                            0xc3 => Ok(JP(Address::U16(self.fetch_u16()?))),
                            0xc4 => Ok(CALLFlags(Flags::NZ, Address::U16(self.fetch_u16()?))),
                            0xc5 => Ok(PUSH(StackRegister::BC)),
                            0xc6 => Ok(ADD(ALUDst::A, ALUSrc::Data(Data::U8(self.fetch()?)))),
                            0xc7 => Ok(RST(RSTAddress::H00)),
                            0xc8 => Ok(RETFlags(Flags::Z)),
                            0xc9 => Ok(RET),
                            0xca => Ok(JPFlags(Flags::Z, Address::U16(self.fetch_u16()?))),
                            0xcb => Ok(PrefixOpcode(self.next_opcode_prefix()?)),
                            0xcc => Ok(CALLFlags(Flags::Z, Address::U16(self.fetch_u16()?))),
                            0xcd => Ok(CALL(Address::U16(self.fetch_u16()?))),
                            0xce => Ok(ADC(ALUSrc::Data(Data::U8(self.fetch()?)))),
                            0xcf => Ok(RST(RSTAddress::H08)),
                            _ => unreachable!(),
                        },
                        0xd0..=0xdf => match opcode {
                            0xd0 => Ok(RETFlags(Flags::NC)),
                            0xd1 => Ok(POP(StackRegister::DE)),
                            0xd2 => Ok(JPFlags(Flags::NC, Address::U16(self.fetch_u16()?))),
                            0xd3 => Ok(Unknown),
                            0xd4 => Ok(CALLFlags(Flags::NC, Address::U16(self.fetch_u16()?))),
                            0xd5 => Ok(PUSH(StackRegister::DE)),
                            0xd6 => Ok(SUB(ALUSrc::Data(Data::U8(self.fetch()?)))),
                            0xd7 => Ok(RST(RSTAddress::H10)),
                            0xd8 => Ok(RETFlags(Flags::C)),
                            0xd9 => Ok(RETI),
                            0xda => Ok(JPFlags(Flags::C, Address::U16(self.fetch_u16()?))),
                            0xdb => Ok(Unknown),
                            0xdc => Ok(CALLFlags(Flags::C, Address::U16(self.fetch_u16()?))),
                            0xdd => Ok(Unknown),
                            0xde => Ok(SBC(ALUSrc::Data(Data::U8(self.fetch()?)))),
                            0xdf => Ok(RST(RSTAddress::H18)),
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    },
                    0xe0..=0xff => match opcode {
                        0xe0..=0xef => match opcode {
                            0xe0 => Ok(LD(LDDst::Address(Address::U8(self.fetch()?)), LDSrc::A)),
                            0xe1 => Ok(POP(StackRegister::HL)),
                            0xe2 => Ok(LD(LDDst::CPtr, LDSrc::A)),
                            0xe3 => Ok(Unknown),
                            0xe4 => Ok(Unknown),
                            0xe5 => Ok(PUSH(StackRegister::HL)),
                            0xe6 => Ok(AND(ALUSrc::Data(Data::U8(self.fetch()?)))),
                            0xe7 => Ok(RST(RSTAddress::H20)),
                            0xe8 => Ok(ADD(ALUDst::SP, ALUSrc::Data(Data::I8(self.fetch()?)))),
                            0xe9 => Ok(JP(Address::HLPtr)),
                            0xea => Ok(LD(
                                LDDst::Address(Address::U16(self.fetch_u16()?)),
                                LDSrc::A,
                            )),
                            0xeb => Ok(Unknown),
                            0xec => Ok(Unknown),
                            0xed => Ok(Unknown),
                            0xee => Ok(XOR(ALUSrc::Data(Data::U8(self.fetch()?)))),
                            0xef => Ok(RST(RSTAddress::H28)),
                            _ => unimplemented!(),
                        },
                        0xf0..=0xff => match opcode {
                            0xf0 => Ok(LD(LDDst::A, LDSrc::Address(Address::U8(self.fetch()?)))),
                            0xf1 => Ok(POP(StackRegister::AF)),
                            0xf2 => Ok(LD(LDDst::A, LDSrc::CPtr)),
                            0xf3 => Ok(DI),
                            0xf4 => Ok(Unknown),
                            0xf5 => Ok(PUSH(StackRegister::AF)),
                            0xf6 => Ok(OR(ALUSrc::Data(Data::U8(self.fetch()?)))),
                            0xf7 => Ok(RST(RSTAddress::H30)),
                            0xf8 => Ok(LD(LDDst::HL, LDSrc::SPOffset(self.fetch()?))),
                            0xf9 => Ok(LD(LDDst::SP, LDSrc::HL)),
                            0xfa => Ok(LD(
                                LDDst::A,
                                LDSrc::Address(Address::U16(self.fetch_u16()?)),
                            )),
                            0xfb => Ok(EI),
                            0xfc => Ok(Unknown),
                            0xfd => Ok(Unknown),
                            0xfe => Ok(CP(ALUSrc::Data(Data::U8(self.fetch()?)))),
                            0xff => Ok(RST(RSTAddress::H38)),
                            _ => unimplemented!(),
                        },
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn next_opcode_prefix(&mut self) -> Result<PrefixOpcode, Error> {
        use PrefixOpcode::*;
        use PrefixRegister::*;
        fn bit(op: u8) -> Bit {
            match op {
                0x40..=0x47 | 0x80..=0x87 | 0xc0..=0xc7 => Bit::B0,
                0x48..=0x4f | 0x88..=0x8f | 0xc8..=0xcf => Bit::B1,
                0x50..=0x57 | 0x90..=0x97 | 0xd0..=0xd7 => Bit::B2,
                0x58..=0x5f | 0x98..=0x9f | 0xd8..=0xdf => Bit::B3,
                0x60..=0x67 | 0xa0..=0xa7 | 0xe0..=0xe7 => Bit::B4,
                0x68..=0x6f | 0xa8..=0xaf | 0xe8..=0xef => Bit::B5,
                0x70..=0x77 | 0xb0..=0xb7 | 0xf0..=0xf7 => Bit::B6,
                0x78..=0x7f | 0xb8..=0xbf | 0xf8..=0xff => Bit::B7,
                _ => unreachable!(),
            }
        }
        #[rustfmt::skip]
        fn reg(mut op: u8) -> PrefixRegister {
            op &= 0xf;
            if op > 0x7 { op -= 0x8; }
            [B, C, D, E, H, L, HLPtr, A][op as usize]
        }
        // TODO(german) test
        let opcode = self.fetch()?;
        match opcode {
            0x00..=0x07 => Ok(RLC(reg(opcode))),
            0x08..=0x0f => Ok(RRC(reg(opcode))),
            0x10..=0x17 => Ok(RL(reg(opcode))),
            0x18..=0x1f => Ok(RR(reg(opcode))),
            0x20..=0x27 => Ok(SLA(reg(opcode))),
            0x28..=0x2f => Ok(SRA(reg(opcode))),
            0x30..=0x37 => Ok(SWAP(reg(opcode))),
            0x38..=0x3f => Ok(SRL(reg(opcode))),
            0x40..=0x7f => Ok(BIT(bit(opcode), reg(opcode))),
            0x80..=0xbf => Ok(RES(bit(opcode), reg(opcode))),
            0xc0..=0xff => Ok(SET(bit(opcode), reg(opcode))),
        }
    }
}

impl<'input> Iterator for Disassembler<'input> {
    type Item = Result<(Opcode, usize), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }
        match self.fetch() {
            Ok(0xcb) => Some(
                self.next_opcode_prefix()
                    .map(|opcode| (Opcode::PrefixOpcode(opcode), 2)),
            ),
            Ok(opcode) => Some(
                self.next_opcode(opcode)
                    .map(|op| (op, OPCODE_LEN[opcode as usize].max(1))),
            ),
            Err(err) => {
                self.ended = true;
                Some(Err(err))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dasm::{Address, Disassembler, Flags, Opcode, RSTAddress, StackRegister};

    #[test]
    fn misc_control() {
        let mut dis = Disassembler::new(&[0x00, 0x10, 0x76, 0xf3, 0xfb]);
        assert_eq!(Some(Ok((Opcode::NOP, 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::STOP, 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::HALT, 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::DI, 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::EI, 1))), dis.next());
    }

    #[test]
    fn rot_shift_bit() {
        let mut dis = Disassembler::new(&[0x07, 0x17, 0x0f, 0x1f]);
        assert_eq!(Some(Ok((Opcode::RLCA, 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RLA, 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RRCA, 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RRA, 1))), dis.next());
    }

    #[test]
    fn stack_push_pop() {
        let mut dis = Disassembler::new(&[0xc1, 0xd1, 0xe1, 0xf1, 0xc5, 0xd5, 0xe5, 0xf5]);
        assert_eq!(Some(Ok((Opcode::POP(StackRegister::BC), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::POP(StackRegister::DE), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::POP(StackRegister::HL), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::POP(StackRegister::AF), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::PUSH(StackRegister::BC), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::PUSH(StackRegister::DE), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::PUSH(StackRegister::HL), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::PUSH(StackRegister::AF), 1))), dis.next());
    }

    #[test]
    fn ret() {
        let mut dis = Disassembler::new(&[0xc0, 0xd0, 0xc8, 0xd8, 0xc9, 0xd9]);
        assert_eq!(Some(Ok((Opcode::RETFlags(Flags::NZ), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RETFlags(Flags::NC), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RETFlags(Flags::Z), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RETFlags(Flags::C), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RET, 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RETI, 1))), dis.next());
    }

    #[test]
    #[rustfmt::skip]
    fn call() {
        let mut dis = Disassembler::new(&[
            0xc4, 0xcd, 0xab,
            0xd4, 0xcd, 0xab,
            0xcc, 0xcd, 0xab,
            0xdc, 0xcd, 0xab,
            0xcd, 0xcd, 0xab,
        ]);
        assert_eq!(Some(Ok((Opcode::CALLFlags(Flags::NZ, Address::U16(0xabcd)), 3))), dis.next());
        assert_eq!(Some(Ok((Opcode::CALLFlags(Flags::NC, Address::U16(0xabcd)), 3))), dis.next());
        assert_eq!(Some(Ok((Opcode::CALLFlags(Flags::Z, Address::U16(0xabcd)), 3))), dis.next());
        assert_eq!(Some(Ok((Opcode::CALLFlags(Flags::C, Address::U16(0xabcd)), 3))), dis.next());
        assert_eq!(Some(Ok((Opcode::CALL(Address::U16(0xabcd)), 3))), dis.next());
    }

    #[test]
    #[rustfmt::skip]
    fn jp() {
        let mut dis = Disassembler::new(&[
            0xc2, 0xcd, 0xab,
            0xd2, 0xcd, 0xab,
            0xca, 0xcd, 0xab,
            0xda, 0xcd, 0xab,
            0xc3, 0xcd, 0xab,
            0xe9,
        ]);
        assert_eq!(Some(Ok((Opcode::JPFlags(Flags::NZ, Address::U16(0xabcd)), 3))), dis.next());
        assert_eq!(Some(Ok((Opcode::JPFlags(Flags::NC, Address::U16(0xabcd)), 3))), dis.next());
        assert_eq!(Some(Ok((Opcode::JPFlags(Flags::Z, Address::U16(0xabcd)), 3))), dis.next());
        assert_eq!(Some(Ok((Opcode::JPFlags(Flags::C, Address::U16(0xabcd)), 3))), dis.next());
        assert_eq!(Some(Ok((Opcode::JP(Address::U16(0xabcd)), 3))), dis.next());
        assert_eq!(Some(Ok((Opcode::JP(Address::HLPtr), 1))), dis.next());
    }

    #[test]
    fn rst() {
        let mut dis = Disassembler::new(&[0xc7, 0xd7, 0xe7, 0xf7, 0xcf, 0xdf, 0xef, 0xff]);
        assert_eq!(Some(Ok((Opcode::RST(RSTAddress::H00), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RST(RSTAddress::H10), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RST(RSTAddress::H20), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RST(RSTAddress::H30), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RST(RSTAddress::H08), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RST(RSTAddress::H18), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RST(RSTAddress::H28), 1))), dis.next());
        assert_eq!(Some(Ok((Opcode::RST(RSTAddress::H38), 1))), dis.next());
    }

    #[test]
    #[rustfmt::skip]
    fn jr() {
        let mut dis = Disassembler::new(&[
            0x20, 0xab,
            0x30, 0xab,
            0x28, 0xab,
            0x38, 0xab,
            0x18, 0xab,
        ]);
        assert_eq!(Some(Ok((Opcode::JRFlags(Flags::NZ, 0xab), 2))), dis.next());
        assert_eq!(Some(Ok((Opcode::JRFlags(Flags::NC, 0xab), 2))), dis.next());
        assert_eq!(Some(Ok((Opcode::JRFlags(Flags::Z, 0xab), 2))), dis.next());
        assert_eq!(Some(Ok((Opcode::JRFlags(Flags::C, 0xab), 2))), dis.next());
        assert_eq!(Some(Ok((Opcode::JR(0xab), 2))), dis.next());
    }
}
