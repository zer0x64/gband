use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive, Clone, Copy, Debug)]
#[repr(u8)]
pub enum Register {
    B = 0,
    C = 1,
    D = 2,
    E = 3,
    H = 4,
    L = 5,
    // HL = 6, // Used in instruction encoding for (HL) and immediates, not truly used as a register
    A = 7,
}

#[derive(TryFromPrimitive, Clone, Copy, Debug)]
#[repr(u8)]
pub enum RegisterPair {
    BC = 0,
    DE = 1,
    HL = 2,
    SP = 3,
    AF = 4, // Only used in Push and Pop, otherwise SP is used. Can't use the same int in rust
}

#[derive(TryFromPrimitive, Clone, Copy, Debug)]
#[repr(u8)]
pub enum Alu {
    Add = 0,
    Adc = 1,
    Sub = 2,
    Sbc = 3,
    And = 4,
    Xor = 5,
    Or = 6,
    Cp = 7,
}

#[derive(TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum Condition {
    NonZero = 0,
    Zero = 1,
    NoCarry = 2,
    Carry = 3,
}

impl core::fmt::Debug for Condition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Condition::NonZero => "nz",
                Condition::Zero => "z",
                Condition::NoCarry => "nc",
                Condition::Carry => "c",
            }
        )
    }
}

#[derive(TryFromPrimitive, Clone, Copy, Debug)]
#[repr(u8)]
pub enum Rot {
    Rlc = 0,
    Rrc = 1,
    Rl = 2,
    Rr = 3,
    Sla = 4,
    Sra = 5,
    Swap = 6,
    Srl = 7,
}

#[derive(Clone, Copy, Debug)]
pub enum OpMemAddress16 {
    Register(RegisterPair),
    RegisterIncrease(RegisterPair),
    RegisterDecrease(RegisterPair),
    Immediate,
}

#[derive(Clone, Copy, Debug)]
pub enum OpMemAddress8 {
    Register(Register),
    Immediate,
}

#[derive(Clone, Copy, Debug)]
pub enum Opcode {
    Unknown,
    CBPrefix,

    // 8 bits load
    LdRR(Register, Register),
    LdRImm(Register),
    LdRMem(Register, OpMemAddress16),
    LdMemR(OpMemAddress16, Register),
    LdMemImm(RegisterPair),
    LdhRead(Register, OpMemAddress8),
    LdhWrite(OpMemAddress8, Register),

    // 16 bits load
    Ld16RImm(RegisterPair),
    Ld16MemSp,
    Ld16SpHL,
    Push(RegisterPair),
    Pop(RegisterPair),

    // 8 bits ALU
    AluR(Alu, Register),
    AluImm(Alu),
    AluMem(Alu),
    IncR(Register),
    IncMem,
    DecR(Register),
    DecMem,
    Daa,
    Cpl,

    // 16 bits ALU
    Add16HL(RegisterPair),
    Add16SPSigned,
    Inc16R(RegisterPair),
    Dec16R(RegisterPair),
    Ld16HLSPSigned,

    // Rotate
    RlcA,
    RlA,
    RrcA,
    RrA,

    // Jump
    JpImm,
    JpHL,
    JpCond(Condition),
    JpRel,
    JpRelCond(Condition),
    Call,
    CallCond(Condition),
    Ret,
    RetCond(Condition),
    Reti,
    Rst(u8),

    // Cpu control
    Nop,
    Ccf,
    Scf,
    Halt,
    Stop,
    Di,
    Ei,
}

impl From<u8> for Opcode {
    fn from(op: u8) -> Self {
        // Typical binary encodings are xx,yyy,zzz and xx,ppq,zzz
        match &op {
            0x40..=0x45
            | 0x47..=0x4D
            | 0x4F..=0x55
            | 0x57..=0x5D
            | 0x5F..=0x65
            | 0x67..=0x6D
            | 0x6F..=0x6F
            | 0x78..=0x7D
            | 0x7F => {
                // Encoding: 01,yyy,zzz y: target reg8 z: source reg8
                let target = Register::try_from((op & 0o070) >> 3)
                    .expect("LD r,r: Unexpected target register");
                let source =
                    Register::try_from(op & 0o007).expect("LD r,r: Unexpected source register");
                Self::LdRR(target, source)
            }
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x3E => {
                // Encoding: 00,yyy,110 y: target reg8
                let target = Register::try_from((op & 0o070) >> 3)
                    .expect("LD r,n: Unexpected target register");
                Self::LdRImm(target)
            }
            0x46 | 0x4E | 0x56 | 0x5E | 0x66 | 0x6E | 0x7E => {
                // Encoding: 01,yyy,110 y: target reg8
                let target = Register::try_from((op & 0o070) >> 3)
                    .expect("LD r,(HL): Unexpected target register");
                Self::LdRMem(target, OpMemAddress16::Register(RegisterPair::HL))
            }
            0x0A | 0x1A => {
                // Encoding: 00,pp1,010 p: source reg16 (BC and DE only)
                let source = RegisterPair::try_from((op & 0b00110000) >> 4)
                    .expect("LD A,(rr): Unexpected source register");
                Self::LdRMem(Register::A, OpMemAddress16::Register(source))
            }
            0x2A => {
                // Encoding: 00,101,010
                Self::LdRMem(
                    Register::A,
                    OpMemAddress16::RegisterIncrease(RegisterPair::HL),
                )
            }
            0x3A => {
                // Encoding: 00,111,010
                Self::LdRMem(
                    Register::A,
                    OpMemAddress16::RegisterDecrease(RegisterPair::HL),
                )
            }
            0xFA => {
                // Encoding: 11,111,010
                Self::LdRMem(Register::A, OpMemAddress16::Immediate)
            }
            0x70..=0x75 | 0x77 => {
                // Encoding: 01,110,zzz z: source reg8
                let source =
                    Register::try_from(op & 0o007).expect("LD (HL),r: Unexpected source register");
                Self::LdMemR(OpMemAddress16::Register(RegisterPair::HL), source)
            }
            0x02 | 0x12 => {
                // Encoding: 00,pp0,010 p: target reg16 (BC and DE only)
                let target = RegisterPair::try_from((op & 0b00110000) >> 4)
                    .expect("LD (rr),A: Unexpected target register");
                Self::LdMemR(OpMemAddress16::Register(target), Register::A)
            }
            0x22 => {
                // Encoding: 00,100,010
                Self::LdMemR(
                    OpMemAddress16::RegisterIncrease(RegisterPair::HL),
                    Register::A,
                )
            }
            0x32 => {
                // Encoding: 00,110,010
                Self::LdMemR(
                    OpMemAddress16::RegisterDecrease(RegisterPair::HL),
                    Register::A,
                )
            }
            0xEA => {
                // Encoding: 11_101_010
                Self::LdMemR(OpMemAddress16::Immediate, Register::A)
            }
            0x36 => {
                // Encoding: 00,110,110
                Self::LdMemImm(RegisterPair::HL)
            }
            0xF2 => {
                // Encoding: 11,110,010
                Self::LdhRead(Register::A, OpMemAddress8::Register(Register::C))
            }
            0xF0 => {
                // Encoding: 11,110,000
                Self::LdhRead(Register::A, OpMemAddress8::Immediate)
            }
            0xE2 => {
                // Encoding: 11,100,010
                Self::LdhWrite(OpMemAddress8::Register(Register::C), Register::A)
            }
            0xE0 => {
                // Encoding: 11,100,000
                Self::LdhWrite(OpMemAddress8::Immediate, Register::A)
            }
            0x01 | 0x11 | 0x21 | 0x31 => {
                // Encoding: 00,pp0,001 p: target reg16
                let target = RegisterPair::try_from((op & 0b00110000) >> 4)
                    .expect("LD rr,nn: Unexpected target register");
                Self::Ld16RImm(target)
            }
            0x08 => {
                // Encoding: 00,001,000
                Self::Ld16MemSp
            }
            0xF9 => {
                // Encoding: 11,111,001
                Self::Ld16SpHL
            }
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                // Encoding: 11,pp0,101 p: source reg16
                // This uses AF for 3, not SP
                let source = RegisterPair::try_from((op & 0b00110000) >> 4)
                    .expect("PUSH rr: Unexpected source register");
                Self::Push(if let RegisterPair::SP = source {
                    RegisterPair::AF
                } else {
                    source
                })
            }
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                // Encoding: 11,pp0,001 p: target reg16
                // This uses AF for 3, not SP
                let target = RegisterPair::try_from((op & 0b00110000) >> 4)
                    .expect("POP rr: Unexpected target register");
                Self::Pop(if let RegisterPair::SP = target {
                    RegisterPair::AF
                } else {
                    target
                })
            }
            0x80..=0x85
            | 0x87..=0x8D
            | 0x8F..=0x95
            | 0x97..=0x9D
            | 0x9F..=0xA5
            | 0xA7..=0xAD
            | 0xAF..=0xB5
            | 0xB7..=0xBD
            | 0xBF => {
                // Encoding: 10,yyy,zzz y: alu op z: source reg8
                let alu_op =
                    Alu::try_from((op & 0o070) >> 3).expect("Alu r: Unexpected alu operation");
                let source =
                    Register::try_from(op & 0o007).expect("Alu r: Unexpected source register");
                Self::AluR(alu_op, source)
            }
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
                // Encoding: 11,yyy,110 y: alu op
                let alu_op =
                    Alu::try_from((op & 0o070) >> 3).expect("Alu n: Unexpected alu operation");
                Self::AluImm(alu_op)
            }
            0x86 | 0x8E | 0x96 | 0x9E | 0xA6 | 0xAE | 0xB6 | 0xBE => {
                // Encoding: 10,yyy,110 y: alu op
                let alu_op =
                    Alu::try_from((op & 0o070) >> 3).expect("Alu (HL): Unexpected alu operation");
                Self::AluMem(alu_op)
            }
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x3C => {
                // Encoding: 00,yyy,100 y: source reg8
                let source = Register::try_from((op & 0o070) >> 3)
                    .expect("INC r: Unexpected source register");
                Self::IncR(source)
            }
            0x34 => {
                // Encoding: 00,110,100
                Self::IncMem
            }
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x3D => {
                // Encoding: 00,yyy,101 y: source reg8
                let source = Register::try_from((op & 0o070) >> 3)
                    .expect("DEC r: Unexpected source register");
                Self::DecR(source)
            }
            0x35 => {
                // Encoding: 00,110,101
                Self::DecMem
            }
            0x27 => {
                // Encoding: 00,100,111
                Self::Daa
            }
            0x2F => {
                // Encoding: 00,101,111
                Self::Cpl
            }
            0x09 | 0x19 | 0x29 | 0x39 => {
                // Encoding: 00,pp1,001 p: source reg16
                let source = RegisterPair::try_from((op & 0b00110000) >> 4)
                    .expect("ADD HL,rr: Unexpected source register");
                Self::Add16HL(source)
            }
            0xE8 => {
                // Encoding: 11,101,000
                Self::Add16SPSigned
            }
            0x03 | 0x13 | 0x23 | 0x33 => {
                // Encoding: 00,pp0,011 p: source reg16
                let source = RegisterPair::try_from((op & 0b00110000) >> 4)
                    .expect("INC rr: Unexpected source register");
                Self::Inc16R(source)
            }
            0x0B | 0x1B | 0x2B | 0x3B => {
                // Encoding: 00,pp1,011 p: source reg16
                let source = RegisterPair::try_from((op & 0b00110000) >> 4)
                    .expect("DEC rr: Unexpected source register");
                Self::Dec16R(source)
            }
            0xF8 => {
                // Encoding: 11,111,000
                Self::Ld16HLSPSigned
            }
            0x07 => {
                // Encoding: 00,000,111
                Self::RlcA
            }
            0x17 => {
                // Encoding: 00,010,111
                Self::RlA
            }
            0x0F => {
                // Encoding: 00,001,111
                Self::RrcA
            }
            0x1F => {
                // Encoding: 00,011,111
                Self::RrA
            }
            0xC3 => {
                // Encoding: 11,000,011
                Self::JpImm
            }
            0xE9 => {
                // Encoding: 11,101,001
                Self::JpHL
            }
            0xC2 | 0xCA | 0xD2 | 0xDA => {
                // Encoding: 11,0yy,010 y: flag condition
                let cond = Condition::try_from((op & 0b00011000) >> 3)
                    .expect("JP f,nn: Unexpected condition");
                Self::JpCond(cond)
            }
            0x18 => {
                // Encoding: 00,011,000
                Self::JpRel
            }
            0x20 | 0x28 | 0x30 | 0x38 => {
                // Encoding: 00,yyy,000 y: flag condition (must substract 4)
                let cond = Condition::try_from(((op & 0o070) >> 3) - 4)
                    .expect("JR f,dd: Unexpected condition");
                Self::JpRelCond(cond)
            }
            0xCD => {
                // Encoding: 11,001,101
                Self::Call
            }
            0xC4 | 0xCC | 0xD4 | 0xDC => {
                // Encoding: 11,0yy,100 y: flag condition
                let cond = Condition::try_from((op & 0b00011000) >> 3)
                    .expect("CALL f,nn: Unexpected condition");
                Self::CallCond(cond)
            }
            0xC9 => {
                // Encoding: 11,001,001
                Self::Ret
            }
            0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                // Encoding: 11,0yy,000 y: flag condition
                let cond = Condition::try_from((op & 0b00011000) >> 3)
                    .expect("RET f: Unexpected condition");
                Self::RetCond(cond)
            }
            0xD9 => {
                // Encoding: 11,011,001
                Self::Reti
            }
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                // Encoding: 11,yyy,111 y: call address (then *8)
                let addr = op & 0o070;
                Self::Rst(addr)
            }
            0x00 => {
                // Encoding: 00,000,000
                Self::Nop
            }
            0x3F => {
                // Encoding: 00,111,111
                Self::Ccf
            }
            0x37 => {
                // Encoding: 00,110,111
                Self::Scf
            }
            0x76 => {
                // Encoding: 01,110,110
                Self::Halt
            }
            0x10 => {
                // Encoding: 00,010,000
                Self::Stop
            }
            0xF3 => {
                // Encoding: 11,110,011
                Self::Di
            }
            0xFB => {
                // Encoding: 11,111,011
                Self::Ei
            }
            0xCB => {
                // Encoding: 11,001,011
                Self::CBPrefix
            }
            _ => Self::Unknown,
        }
    }
}

impl Opcode {
    pub fn cycles(&self) -> u8 {
        match self {
            Self::Unknown => 1,
            Self::CBPrefix => 1,
            Self::LdRR(_, _) => 1,
            Self::LdRImm(_) => 2,
            Self::LdRMem(_, mem) => match mem {
                OpMemAddress16::Immediate => 4,
                _ => 2,
            },
            Self::LdMemR(mem, _) => match mem {
                OpMemAddress16::Immediate => 4,
                _ => 2,
            },
            Self::LdMemImm(_) => 3,
            Self::LdhRead(_, mem) => match mem {
                OpMemAddress8::Register(_) => 2,
                OpMemAddress8::Immediate => 3,
            },
            Self::LdhWrite(mem, _) => match mem {
                OpMemAddress8::Register(_) => 2,
                OpMemAddress8::Immediate => 3,
            },
            Self::Ld16RImm(_) => 3,
            Self::Ld16MemSp => 5,
            Self::Ld16SpHL => 2,
            Self::Push(_) => 4,
            Self::Pop(_) => 3,
            Self::AluR(_, _) => 1,
            Self::AluImm(_) => 2,
            Self::AluMem(_) => 2,
            Self::IncR(_) => 1,
            Self::IncMem => 3,
            Self::DecR(_) => 1,
            Self::DecMem => 3,
            Self::Daa => 1,
            Self::Cpl => 1,
            Self::Add16HL(_) => 2,
            Self::Add16SPSigned => 4,
            Self::Inc16R(_) => 2,
            Self::Dec16R(_) => 2,
            Self::Ld16HLSPSigned => 3,
            Self::RlcA => 1,
            Self::RlA => 1,
            Self::RrcA => 1,
            Self::RrA => 1,
            Self::JpImm => 4,
            Self::JpHL => 1,
            Self::JpCond(_) => 3, // +1 if condition true
            Self::JpRel => 3,
            Self::JpRelCond(_) => 2, // +1 if condition true
            Self::Call => 6,
            Self::CallCond(_) => 3, // +3 if condition true
            Self::Ret => 4,
            Self::RetCond(_) => 2, // +3 if condition true
            Self::Reti => 4,
            Self::Rst(_) => 4,
            Self::Nop => 1,
            Self::Ccf => 1,
            Self::Scf => 1,
            Self::Halt => 1, // Actually... unknown
            Self::Stop => 1, // Actually... unknown
            Self::Di => 1,
            Self::Ei => 1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum OpcodeCB {
    RotateR(Rot, Register),
    RotateMem(Rot),
    BitR(u8, Register),
    BitMem(u8),
    ResR(u8, Register),
    ResMem(u8),
    SetR(u8, Register),
    SetMem(u8),
}

impl From<u8> for OpcodeCB {
    fn from(op: u8) -> Self {
        match op {
            0x00..=0x05
            | 0x07..=0x0D
            | 0x0F..=0x15
            | 0x17..=0x1D
            | 0x1F..=0x25
            | 0x27..=0x2D
            | 0x2F..=0x35
            | 0x37..=0x3D
            | 0x3F => {
                // Encoding: 00,yyy,zzz y: rot op z: source reg8
                let rot_op =
                    Rot::try_from((op & 0o070) >> 3).expect("Rot r: Unexpected rot operation");
                let source =
                    Register::try_from(op & 0o007).expect("Rot r: Unexpected source register");
                Self::RotateR(rot_op, source)
            }
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
                // Encoding: 00,yyy,110 y: rot op
                let rot_op =
                    Rot::try_from((op & 0o070) >> 3).expect("Rot (HL): Unexpected rot operation");
                Self::RotateMem(rot_op)
            }
            0x40..=0x45
            | 0x47..=0x4D
            | 0x4F..=0x55
            | 0x57..=0x5D
            | 0x5F..=0x65
            | 0x67..=0x6D
            | 0x6F..=0x75
            | 0x77..=0x7D
            | 0x7F => {
                // Encoding: 01,yyy,zzz y: bit index z: source reg8
                let index = (op & 0o070) >> 3;
                let source =
                    Register::try_from(op & 0o007).expect("BIT n, r: Unexpected source register");
                Self::BitR(index, source)
            }
            0x46 | 0x4E | 0x56 | 0x5E | 0x66 | 0x6E | 0x76 | 0x7E => {
                // Encoding: 01,yyy,110 y: bit index
                let index = (op & 0o070) >> 3;
                Self::BitMem(index)
            }
            0x80..=0x85
            | 0x87..=0x8D
            | 0x8F..=0x95
            | 0x97..=0x9D
            | 0x9F..=0xA5
            | 0xA7..=0xAD
            | 0xAF..=0xB5
            | 0xB7..=0xBD
            | 0xBF => {
                // Encoding: 10,yyy,zzz y: bit index z: source reg8
                let index = (op & 0o070) >> 3;
                let source =
                    Register::try_from(op & 0o007).expect("RES n, r: Unexpected source register");
                Self::ResR(index, source)
            }
            0x86 | 0x8E | 0x96 | 0x9E | 0xA6 | 0xAE | 0xB6 | 0xBE => {
                // Encoding: 10,yyy,110 y: bit index
                let index = (op & 0o070) >> 3;
                Self::ResMem(index)
            }
            0xC0..=0xC5
            | 0xC7..=0xCD
            | 0xCF..=0xD5
            | 0xD7..=0xDD
            | 0xDF..=0xE5
            | 0xE7..=0xED
            | 0xEF..=0xF5
            | 0xF7..=0xFD
            | 0xFF => {
                // Encoding: 11,yyy,zzz y: bit index z: source reg8
                let index = (op & 0o070) >> 3;
                let source =
                    Register::try_from(op & 0o007).expect("SET n, r: Unexpected source register");
                Self::SetR(index, source)
            }
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
                // Encoding: 11,yyy,110 y: bit index
                let index = (op & 0o070) >> 3;
                Self::SetMem(index)
            }
        }
    }
}

impl OpcodeCB {
    pub fn cycles(&self) -> u8 {
        match self {
            Self::RotateR(_, _) => 2,
            Self::RotateMem(_) => 4,
            Self::BitR(_, _) => 2,
            Self::BitMem(_) => 3,
            Self::ResR(_, _) => 2,
            Self::ResMem(_) => 4,
            Self::SetR(_, _) => 2,
            Self::SetMem(_) => 4,
        }
    }
}

#[cfg(test)]
#[test]
fn test_all_instructions_implemented() {
    for i in 0u8..=255u8 {
        let opcode = Opcode::from(i);
        match i {
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                assert!(
                    matches!(opcode, Opcode::Unknown),
                    "{:#04X} should be unknown",
                    i
                );
            }
            _ => {
                assert!(
                    !matches!(opcode, Opcode::Unknown),
                    "{:#04X} shouldn't be unknown",
                    i
                );
            }
        }
    }
}
