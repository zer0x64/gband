use crate::bus::CpuBus;
use crate::cpu::decoder::{OpMemAddress16, OpMemAddress8, Opcode, OpcodeCB};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub fn disassemble(bus: &mut CpuBus) -> Vec<(u8, u16, String)> {
    let mut pc = 0u16;
    let mut disassembly = Vec::new();

    let rom_bank = bus.get_cartridge_rom_bank();
    let ram_bank = bus.get_cartridge_ram_bank();
    let wram_bank = bus.read_without_dma_check(0xFF70, false);

    // Read the entire memory space
    while pc < 0xFFFF {
        let pc_temp = pc;
        let bank = match pc_temp {
            0x0000..=0x3fff => {
                // Cartridge ROM0
                0
            }
            0x4000..=0x7fff => {
                // Cartridge ROMX
                rom_bank
            }
            0x8000..=0x9FFF => {
                // VRAM
                0 // TODO
            }
            0xA000..=0xBFFF => {
                // Cartridge RAM
                ram_bank
            }
            0xC000..=0xCFFF | 0xE000..=0xEFFF => {
                // WRAM0
                0
            }
            0xD000..=0xDFFF | 0xF000..=0xFDFF => {
                // WRAMX
                wram_bank
            }
            _ => 0,
        };

        let op = Opcode::from(read_immediate(bus, &mut pc));
        disassembly.push((bank, pc_temp, op.to_string(bus, &mut pc)));
    }

    disassembly
}

fn read_immediate(bus: &mut CpuBus, pc: &mut u16) -> u8 {
    let immediate = bus.read(*pc);
    *pc = pc.wrapping_add(1);
    immediate
}

fn read_immediate16(bus: &mut CpuBus, pc: &mut u16) -> u16 {
    let lsb = read_immediate(bus, pc) as u16;
    let msb = read_immediate(bus, pc) as u16;
    (msb << 8) | lsb
}

impl Opcode {
    fn to_string(&self, bus: &mut CpuBus, pc: &mut u16) -> String {
        match self {
            Opcode::Unknown => "???".to_string(),
            Opcode::CBPrefix => {
                let op = OpcodeCB::from(read_immediate(bus, pc));
                op.to_string()
            }
            Opcode::LdRR(target, source) => format!("ld {target:?}, {source:?}").to_lowercase(),
            Opcode::LdRImm(target) => {
                let immediate = read_immediate(bus, pc);
                format!("ld {target:?}, {immediate:02x}").to_lowercase()
            }
            Opcode::LdRMem(target, source) => {
                let source = match source {
                    OpMemAddress16::Register(source) => format!("[{source:?}]"),
                    OpMemAddress16::RegisterIncrease(source) => format!("[{source:?}+]"),
                    OpMemAddress16::RegisterDecrease(source) => format!("[{source:?}-]"),
                    OpMemAddress16::Immediate => {
                        let addr = read_immediate16(bus, pc);
                        format!("[{addr:04x}]")
                    }
                };
                format!("ld {target:?}, {source}").to_lowercase()
            }
            Opcode::LdMemR(target, source) => {
                let target = match target {
                    OpMemAddress16::Register(target) => format!("[{target:?}]"),
                    OpMemAddress16::RegisterIncrease(target) => format!("[{target:?}+]"),
                    OpMemAddress16::RegisterDecrease(target) => format!("[{target:?}-]"),
                    OpMemAddress16::Immediate => {
                        let addr = read_immediate16(bus, pc);
                        format!("[{addr:04x}]")
                    }
                };
                format!("ld {target}, {source:?}").to_lowercase()
            }
            Opcode::LdMemImm(target) => {
                let immediate = read_immediate(bus, pc);
                format!("ld {target:?}, {immediate:02x}").to_lowercase()
            }
            Opcode::LdhRead(target, source) => {
                let source = match source {
                    OpMemAddress8::Register(source) => format!("[{source:?}]"),
                    OpMemAddress8::Immediate => {
                        let addr = 0xFF00 | read_immediate(bus, pc) as u16;
                        format!("[{addr:04x}]")
                    }
                };
                format!("ldh {target:?}, {source}").to_lowercase()
            }
            Opcode::LdhWrite(target, source) => {
                let target = match target {
                    OpMemAddress8::Register(target) => format!("[{target:?}]"),
                    OpMemAddress8::Immediate => {
                        let addr = 0xFF00 | read_immediate(bus, pc) as u16;
                        format!("[{addr:04x}]")
                    }
                };
                format!("ldh {target}, {source:?}").to_lowercase()
            }
            Opcode::Ld16RImm(target) => {
                let immediate = read_immediate16(bus, pc);
                format!("ld {target:?}, {immediate:04x}").to_lowercase()
            }
            Opcode::Ld16MemSp => {
                let addr = read_immediate16(bus, pc);
                format!("ld [{addr:04x}], sp")
            }
            Opcode::Ld16SpHL => "ld sp, hl".to_string(),
            Opcode::Push(reg) => format!("push {reg:?}").to_lowercase(),
            Opcode::Pop(reg) => format!("pop {reg:?}").to_lowercase(),
            Opcode::AluR(op, reg) => format!("{op:?} {reg:?}").to_lowercase(),
            Opcode::AluImm(op) => {
                let val = read_immediate(bus, pc);
                format!("{op:?} {val:02x}").to_lowercase()
            }
            Opcode::AluMem(op) => format!("{op:?}, [hl]").to_lowercase(),
            Opcode::IncR(reg) => format!("inc {reg:?}").to_lowercase(),
            Opcode::IncMem => "inc [hl]".to_string(),
            Opcode::DecR(reg) => format!("dec {reg:?}").to_lowercase(),
            Opcode::DecMem => "dec [hl]".to_string(),
            Opcode::Daa => "daa".to_string(),
            Opcode::Cpl => "cpl".to_string(),
            Opcode::Add16HL(reg) => format!("add hl, {reg:?}").to_lowercase(),
            Opcode::Add16SPSigned => {
                let immediate = read_immediate(bus, pc);
                format!("add sp, {immediate:02x}").to_lowercase()
            }
            Opcode::Inc16R(reg) => format!("inc {reg:?}").to_lowercase(),
            Opcode::Dec16R(reg) => format!("dec {reg:?}").to_lowercase(),
            Opcode::Ld16HLSPSigned => {
                let immediate = read_immediate(bus, pc);
                format!("ld hl, sp+{immediate:02x}")
            }
            Opcode::RlcA => "rlca".to_string(),
            Opcode::RlA => "rla".to_string(),
            Opcode::RrcA => "rrca".to_string(),
            Opcode::RrA => "rra".to_string(),
            Opcode::JpImm => {
                let addr = read_immediate16(bus, pc);
                format!("jp {addr:04x}")
            }
            Opcode::JpHL => "jp hl".to_string(),
            Opcode::JpCond(condition) => {
                let addr = read_immediate16(bus, pc);
                format!("jp {condition:?}, {addr:04x}")
            }
            Opcode::JpRel => {
                let offset = read_immediate(bus, pc) as i8;
                let addr = pc.wrapping_add(offset as u16);
                format!("jr {addr:04x}")
            }
            Opcode::JpRelCond(condition) => {
                let offset = read_immediate(bus, pc) as i8;
                let addr = pc.wrapping_add(offset as u16);
                format!("jr {condition:?}, {addr:04x}")
            }
            Opcode::Call => {
                let addr = read_immediate16(bus, pc);
                format!("call {addr:04x}")
            }
            Opcode::CallCond(condition) => {
                let addr = read_immediate16(bus, pc);
                format!("call {condition:?}, {addr:04x}")
            }
            Opcode::Ret => "ret".to_string(),
            Opcode::RetCond(condition) => format!("ret {condition:?}").to_lowercase(),
            Opcode::Reti => "reti".to_string(),
            Opcode::Rst(index) => format!("rst {index:02x}").to_lowercase(),
            Opcode::Nop => "nop".to_string(),
            Opcode::Ccf => "ccf".to_string(),
            Opcode::Scf => "scf".to_string(),
            Opcode::Halt => "halt".to_string(),
            Opcode::Stop => "stop".to_string(),
            Opcode::Di => "di".to_string(),
            Opcode::Ei => "ei".to_string(),
        }
    }
}

impl OpcodeCB {
    fn to_string(&self) -> String {
        match self {
            OpcodeCB::RotateR(op, reg) => format!("{op:?} {reg:?}").to_lowercase(),
            OpcodeCB::RotateMem(op) => format!("{op:?} [hl]").to_lowercase(),
            OpcodeCB::BitR(index, reg) => format!("bit {index}, {reg:?}").to_lowercase(),
            OpcodeCB::BitMem(index) => format!("bit {index}, [hl]").to_lowercase(),
            OpcodeCB::ResR(index, reg) => format!("res {index}, {reg:?}").to_lowercase(),
            OpcodeCB::ResMem(index) => format!("res {index}, [hl]").to_lowercase(),
            OpcodeCB::SetR(index, reg) => format!("set {index}, {reg:?}").to_lowercase(),
            OpcodeCB::SetMem(index) => format!("bit {index}, [hl]").to_lowercase(),
        }
    }
}
