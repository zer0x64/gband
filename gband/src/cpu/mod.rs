mod decoder;

use bitflags::bitflags;

use crate::{bus::CpuBus, InterruptReg, OamDma};
use decoder::{
    Alu, Condition, OpMemAddress16, OpMemAddress8, Opcode, OpcodeCB, Register, RegisterPair, Rot,
};

bitflags! {
    pub struct FlagRegister: u8 {
        const UNUSED = 0x0F;
        const C = 0x10;
        const H = 0x20;
        const N = 0x40;
        const Z = 0x80;
    }
}

pub struct Cpu {
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub a: u8,
    pub f: FlagRegister,
    pub sp: u16,
    pub pc: u16,

    pub cycles: u8,
    pub opcode_latch: Opcode,
    pub interrupt_master_enable: bool,
    pub ime_pending: Option<bool>,
    pub halted: bool,
    pub halt_bug_active: bool,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            // Values after CGB Boot ROM
            b: 0,
            c: 0,
            d: 0xFF,
            e: 0x56,
            h: 0,
            l: 0x0D,
            a: 0x11,
            f: FlagRegister::Z,

            sp: 0xFFFE,
            pc: 0x0100,

            cycles: 0,
            opcode_latch: Opcode::Unknown,
            interrupt_master_enable: false,
            ime_pending: None,
            halted: false,
            halt_bug_active: false,
        }
    }
}

impl Cpu {
    pub fn clock(&mut self, bus: &mut CpuBus) {
        self.handle_oam_dma(bus);

        // TODO: Don't clock if the CPU is STOPped
        if bus.get_timer_registers().clock() {
            bus.request_interrupt(InterruptReg::TIMER);
        }

        if bus.get_serial_port().clock() {
            bus.request_interrupt(InterruptReg::SERIAL);
        }

        // Fetch/Execute overlap, last cycle of execute runs at the same time as the next fetch
        if !self.halted && self.cycles != 0 {
            self.execute(bus);

            // We are not emulating cycle-accurate yet, so just reset the latch to unknown to noop the remaining cycles
            self.opcode_latch = Opcode::Unknown;

            self.cycles -= 1;
        }

        if self.cycles == 0 {
            match self.ime_pending {
                Some(true) => self.ime_pending = Some(false),
                Some(false) => {
                    self.interrupt_master_enable = true;
                    self.ime_pending = None;
                }
                None => {}
            }

            self.handle_interrupt(bus);

            if !self.halted {
                self.fetch(bus);
            }
        }
    }

    fn handle_interrupt(&mut self, bus: &mut CpuBus) {
        let interrupts_status = bus.read(0xFF0F);
        let interrupts_enable = bus.read(0xFFFF);

        // Get the highest priority interrupt requested, bit 0 is higher priority
        let pending = interrupts_enable & interrupts_status & 0x1F;
        let pending_index = pending.trailing_zeros() as u16;

        if pending != 0 {
            // Wake up from halt, even if ime is not set
            self.halted = false;

            if self.interrupt_master_enable {
                // Unset ime and request flag
                self.interrupt_master_enable = false;
                bus.write(0xFF0F, interrupts_status & !(1 << pending_index));

                // Save pc and run ISR
                self.push_stack(bus, self.pc);
                self.pc = 0x0040 + 0x0008 * pending_index;

                // The ISR takes 5 cycles
                self.cycles = 5;
            }
        }
    }

    // TODO: Remove pub added for criterion
    pub fn fetch(&mut self, bus: &mut CpuBus) {
        self.opcode_latch = Opcode::from(self.read_immediate(bus));
        self.cycles = self.opcode_latch.cycles();

        if self.halt_bug_active {
            // Revert pc increment here, instead of running this on every read_immediate
            self.pc = self.pc.wrapping_sub(1);
            self.halt_bug_active = false;
        }
    }

    // TODO: Remove pub added for criterion
    pub fn execute(&mut self, bus: &mut CpuBus) {
        // In Z80 / GB, unknown instructions are just noop
        match self.opcode_latch {
            Opcode::Unknown | Opcode::Nop => {
                // noop
            }
            Opcode::CBPrefix => {
                self.run_cb(bus);
            }
            Opcode::LdRR(target, source) => {
                self.set_register(target, self.get_register(source));
            }
            Opcode::LdRImm(target) => {
                let immediate = self.read_immediate(bus);
                self.set_register(target, immediate);
            }
            Opcode::LdRMem(target, source) => {
                let val = match source {
                    OpMemAddress16::Register(source) => bus.read(self.get_register_pair(source)),
                    OpMemAddress16::RegisterIncrease(source) => {
                        let reg = self.get_register_pair(source);
                        self.set_register_pair(source, reg.wrapping_add(1));
                        bus.read(reg)
                    }
                    OpMemAddress16::RegisterDecrease(source) => {
                        let reg = self.get_register_pair(source);
                        self.set_register_pair(source, reg.wrapping_sub(1));
                        bus.read(reg)
                    }
                    OpMemAddress16::Immediate => {
                        let addr = self.read_immediate16(bus);
                        bus.read(addr)
                    }
                };

                self.set_register(target, val);
            }
            Opcode::LdMemR(target, source) => {
                let addr = match target {
                    OpMemAddress16::Register(target) => self.get_register_pair(target),
                    OpMemAddress16::RegisterIncrease(target) => {
                        let reg = self.get_register_pair(target);
                        self.set_register_pair(target, reg.wrapping_add(1));
                        reg
                    }
                    OpMemAddress16::RegisterDecrease(target) => {
                        let reg = self.get_register_pair(target);
                        self.set_register_pair(target, reg.wrapping_sub(1));
                        reg
                    }
                    OpMemAddress16::Immediate => self.read_immediate16(bus),
                };

                bus.write(addr, self.get_register(source));
            }
            Opcode::LdMemImm(target) => {
                let immediate = self.read_immediate(bus);
                bus.write(self.get_register_pair(target), immediate);
            }
            Opcode::LdhRead(target, source) => {
                let addr = 0xFF00
                    | match source {
                        OpMemAddress8::Register(source) => self.get_register(source),
                        OpMemAddress8::Immediate => self.read_immediate(bus),
                    } as u16;

                self.set_register(target, bus.read(addr));
            }
            Opcode::LdhWrite(target, source) => {
                let addr = 0xFF00
                    | match target {
                        OpMemAddress8::Register(target) => self.get_register(target),
                        OpMemAddress8::Immediate => self.read_immediate(bus),
                    } as u16;

                bus.write(addr, self.get_register(source));
            }
            Opcode::Ld16RImm(target) => {
                let immediate = self.read_immediate16(bus);
                self.set_register_pair(target, immediate);
            }
            Opcode::Ld16MemSp => {
                let addr = self.read_immediate16(bus);
                bus.write(addr, (self.sp & 0x00FF) as u8);
                bus.write(addr + 1, (self.sp >> 8) as u8);
            }
            Opcode::Ld16SpHL => {
                self.sp = self.get_register_pair(RegisterPair::HL);
            }
            Opcode::Push(source) => {
                let source = self.get_register_pair(source);
                self.push_stack(bus, source);
            }
            Opcode::Pop(target) => {
                let val = self.pop_stack(bus);
                self.set_register_pair(target, val);
            }
            Opcode::AluR(alu_op, source) => {
                let val = self.get_register(source);
                self.run_alu(alu_op, val);
            }
            Opcode::AluImm(alu_op) => {
                let val = self.read_immediate(bus);
                self.run_alu(alu_op, val);
            }
            Opcode::AluMem(alu_op) => {
                let val = bus.read(self.get_register_pair(RegisterPair::HL));
                self.run_alu(alu_op, val);
            }
            Opcode::IncR(source) => {
                let val = self.get_register(source);
                let result = val.wrapping_add(1);

                self.f.set(FlagRegister::H, (val & 0x0F) + 1 > 0x0F);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, result == 0);
                self.set_register(source, result);
            }
            Opcode::IncMem => {
                let addr = self.get_register_pair(RegisterPair::HL);
                let val = bus.read(addr);
                let result = val.wrapping_add(1);

                self.f.set(FlagRegister::H, (val & 0x0F) + 1 > 0x0F);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, result == 0);
                bus.write(addr, result);
            }
            Opcode::DecR(source) => {
                let val = self.get_register(source);
                let result = val.wrapping_sub(1);

                self.f.set(FlagRegister::H, (val & 0x0F) == 0);
                self.f.set(FlagRegister::N, true);
                self.f.set(FlagRegister::Z, result == 0);
                self.set_register(source, result);
            }
            Opcode::DecMem => {
                let addr = self.get_register_pair(RegisterPair::HL);
                let val = bus.read(addr);
                let result = val.wrapping_sub(1);

                self.f.set(FlagRegister::H, (val & 0x0F) == 0);
                self.f.set(FlagRegister::N, true);
                self.f.set(FlagRegister::Z, result == 0);
                bus.write(addr, result);
            }
            Opcode::Daa => {
                let mut adjustment = if self.f.contains(FlagRegister::C) {
                    0x60
                } else {
                    0
                };

                if self.f.contains(FlagRegister::H) {
                    adjustment |= 0x06;
                }

                if !self.f.contains(FlagRegister::N) {
                    if (self.a & 0x0F) > 0x09 {
                        adjustment |= 0x06;
                    }

                    if self.a > 0x99 {
                        adjustment |= 0x60;
                    }

                    self.a = self.a.wrapping_add(adjustment);
                } else {
                    self.a = self.a.wrapping_sub(adjustment)
                }

                self.f.set(FlagRegister::C, adjustment >= 0x60);
                self.f.set(FlagRegister::H, false);
                self.f.set(FlagRegister::Z, self.a == 0);
            }
            Opcode::Cpl => {
                self.a = self.a ^ 0xFF;
                self.f.set(FlagRegister::H, true);
                self.f.set(FlagRegister::N, true);
            }
            Opcode::Add16HL(source) => {
                let val = self.get_register_pair(RegisterPair::HL);
                let source = self.get_register_pair(source);
                let (result, carry) = val.overflowing_add(source);
                let half_carry = (val & 0x0FFF) + (source & 0x0FFF) > 0x0FFF;

                self.set_register_pair(RegisterPair::HL, result);
                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, false);
            }
            Opcode::Add16SPSigned => {
                // Reinterpret the immediate as signed, then convert to unsigned u16 equivalent
                let immediate = self.read_immediate(bus) as i8 as u16;
                let carry = (self.sp & 0x00FF) + (immediate & 0x00FF) > 0x00FF;
                let half_carry = (self.sp & 0x000F) + (immediate & 0x000F) > 0x000F;

                self.sp = self.sp.wrapping_add(immediate);
                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, false);
            }
            Opcode::Inc16R(source) => {
                self.set_register_pair(source, self.get_register_pair(source).wrapping_add(1));
            }
            Opcode::Dec16R(source) => {
                self.set_register_pair(source, self.get_register_pair(source).wrapping_sub(1));
            }
            Opcode::Ld16HLSPSigned => {
                // Two's complement conversion
                let immediate = self.read_immediate(bus) as i8 as u16;
                let carry = (self.sp & 0x00FF) + (immediate & 0x00FF) > 0x00FF;
                let half_carry = (self.sp & 0x000F) + (immediate & 0x000F) > 0x000F;

                self.set_register_pair(RegisterPair::HL, self.sp.wrapping_add(immediate));
                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, false);
            }
            Opcode::RlcA => {
                let val = self.get_register(Register::A);
                let result = self.run_rot(Rot::Rlc, val, true);
                self.set_register(Register::A, result);
            }
            Opcode::RlA => {
                let val = self.get_register(Register::A);
                let result = self.run_rot(Rot::Rl, val, true);
                self.set_register(Register::A, result);
            }
            Opcode::RrcA => {
                let val = self.get_register(Register::A);
                let result = self.run_rot(Rot::Rrc, val, true);
                self.set_register(Register::A, result);
            }
            Opcode::RrA => {
                let val = self.get_register(Register::A);
                let result = self.run_rot(Rot::Rr, val, true);
                self.set_register(Register::A, result);
            }
            Opcode::JpImm => {
                let addr = self.read_immediate16(bus);
                self.pc = addr;
            }
            Opcode::JpHL => {
                self.pc = self.get_register_pair(RegisterPair::HL);
            }
            Opcode::JpCond(condition) => {
                let addr = self.read_immediate16(bus);
                if self.check_conditional(condition) {
                    self.cycles += 1;
                    self.pc = addr;
                }
            }
            Opcode::JpRel => {
                let offset = self.read_immediate(bus) as i8;
                self.pc = self.pc.wrapping_add(offset as u16);
            }
            Opcode::JpRelCond(condition) => {
                let offset = self.read_immediate(bus) as i8;
                if self.check_conditional(condition) {
                    self.cycles += 1;
                    self.pc = self.pc.wrapping_add(offset as u16)
                }
            }
            Opcode::Call => {
                let addr = self.read_immediate16(bus);
                self.push_stack(bus, self.pc);
                self.pc = addr;
            }
            Opcode::CallCond(condition) => {
                let addr = self.read_immediate16(bus);
                if self.check_conditional(condition) {
                    self.cycles += 3;
                    self.push_stack(bus, self.pc);
                    self.pc = addr;
                }
            }
            Opcode::Ret => {
                let addr = self.pop_stack(bus);
                self.pc = addr;
            }
            Opcode::RetCond(condition) => {
                if self.check_conditional(condition) {
                    self.cycles += 3;
                    let addr = self.pop_stack(bus);
                    self.pc = addr;
                }
            }
            Opcode::Reti => {
                let addr = self.pop_stack(bus);
                self.pc = addr;

                // IME enable is NOT delayed to the next instruction.
                self.interrupt_master_enable = true;
            }
            Opcode::Rst(addr) => {
                self.push_stack(bus, self.pc);
                self.pc = addr as u16;
            }
            Opcode::Ccf => {
                self.f
                    .set(FlagRegister::C, !self.f.contains(FlagRegister::C));
                self.f.remove(FlagRegister::N);
                self.f.remove(FlagRegister::H);
            }
            Opcode::Scf => {
                self.f.insert(FlagRegister::C);
                self.f.remove(FlagRegister::N);
                self.f.remove(FlagRegister::H);
            }
            Opcode::Halt => {
                let interrupts_status = bus.read(0xFF0F);
                let interrupts_enable = bus.read(0xFFFF);
                let pending = interrupts_enable & interrupts_status & 0x1F;

                // If there is already an interrupt pending AND IME is false, skip halt completely
                if !self.interrupt_master_enable && pending != 0 {
                    self.halt_bug_active = true;
                } else {
                    self.halted = true;
                }
            }
            Opcode::Stop => {
                // TODO: Completely implement stop (sleep portion...?)
                bus.get_timer_registers().reset_div();
                bus.toggle_double_speed();
            }
            Opcode::Di => {
                self.interrupt_master_enable = false;
            }
            Opcode::Ei => {
                // IME enable is delayed to the next instruction.
                self.ime_pending = Some(true);
            }
        }
    }

    fn read_immediate(&mut self, bus: &mut CpuBus) -> u8 {
        let immediate = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        immediate
    }

    fn read_immediate16(&mut self, bus: &mut CpuBus) -> u16 {
        let lsb = self.read_immediate(bus) as u16;
        let msb = self.read_immediate(bus) as u16;
        (msb << 8) | lsb
    }

    fn pop_stack(&mut self, bus: &mut CpuBus) -> u16 {
        let lsb = bus.read(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let msb = bus.read(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        (msb << 8) | lsb
    }

    fn push_stack(&mut self, bus: &mut CpuBus, val: u16) {
        self.sp = self.sp.wrapping_sub(1);
        bus.write(self.sp, (val >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        bus.write(self.sp, (val & 0x00FF) as u8);
    }

    fn run_cb(&mut self, bus: &mut CpuBus) {
        let op = OpcodeCB::from(self.read_immediate(bus));
        self.cycles += op.cycles();

        match op {
            OpcodeCB::RotateR(rot_op, source) => {
                let val = self.get_register(source);
                let result = self.run_rot(rot_op, val, false);
                self.set_register(source, result);
            }
            OpcodeCB::RotateMem(rot_op) => {
                let val = bus.read(self.get_register_pair(RegisterPair::HL));
                let result = self.run_rot(rot_op, val, false);
                bus.write(self.get_register_pair(RegisterPair::HL), result);
            }
            OpcodeCB::BitR(index, source) => {
                let val = self.get_register(source);
                let mask = 1u8 << index;

                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::H, true);
                self.f.set(FlagRegister::Z, (val & mask) == 0);
            }
            OpcodeCB::BitMem(index) => {
                let val = bus.read(self.get_register_pair(RegisterPair::HL));
                let mask = 1u8 << index;

                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::H, true);
                self.f.set(FlagRegister::Z, (val & mask) == 0);
            }
            OpcodeCB::ResR(index, source) => {
                let val = self.get_register(source);
                let mask = !(1u8 << index);
                self.set_register(source, val & mask);
            }
            OpcodeCB::ResMem(index) => {
                let val = bus.read(self.get_register_pair(RegisterPair::HL));
                let mask = !(1u8 << index);
                bus.write(self.get_register_pair(RegisterPair::HL), val & mask);
            }
            OpcodeCB::SetR(index, source) => {
                let val = self.get_register(source);
                let mask = 1u8 << index;
                self.set_register(source, val | mask);
            }
            OpcodeCB::SetMem(index) => {
                let val = bus.read(self.get_register_pair(RegisterPair::HL));
                let mask = 1u8 << index;
                bus.write(self.get_register_pair(RegisterPair::HL), val | mask);
            }
        }
    }

    fn run_alu(&mut self, alu_op: Alu, val: u8) {
        match alu_op {
            Alu::Add => {
                let (result, carry) = self.a.overflowing_add(val);
                let half_carry = (self.a & 0x0F) + (val & 0x0F) > 0x0F;

                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, result == 0);
                self.a = result;
            }
            Alu::Adc => {
                let carry_flag = self.f.contains(FlagRegister::C) as u8;

                // Would use carrying_add if it was in stable
                let (r1, c1) = self.a.overflowing_add(val);
                let (result, c2) = r1.overflowing_add(carry_flag);
                let carry = c1 | c2;
                let half_carry = (self.a & 0x0F) + (val & 0x0F) + carry_flag > 0x0F;

                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, result == 0);
                self.a = result;
            }
            Alu::Sub => {
                let (result, carry) = self.a.overflowing_sub(val);
                let half_carry = (self.a & 0x0F) < (val & 0x0F);

                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, true);
                self.f.set(FlagRegister::Z, result == 0);
                self.a = result;
            }
            Alu::Sbc => {
                let carry_flag = self.f.contains(FlagRegister::C) as u8;

                // Would use carrying_sub if it was in stable
                let (r1, c1) = self.a.overflowing_sub(val);
                let (result, c2) = r1.overflowing_sub(carry_flag);
                let carry = c1 | c2;
                let half_carry = (self.a & 0x0F) < (val & 0x0F) + carry_flag;

                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, true);
                self.f.set(FlagRegister::Z, result == 0);
                self.a = result;
            }
            Alu::And => {
                self.a &= val;
                self.f.set(FlagRegister::C, false);
                self.f.set(FlagRegister::H, true);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, self.a == 0);
            }
            Alu::Xor => {
                self.a ^= val;
                self.f.set(FlagRegister::C, false);
                self.f.set(FlagRegister::H, false);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, self.a == 0);
            }
            Alu::Or => {
                self.a |= val;
                self.f.set(FlagRegister::C, false);
                self.f.set(FlagRegister::H, false);
                self.f.set(FlagRegister::N, false);
                self.f.set(FlagRegister::Z, self.a == 0);
            }
            Alu::Cp => {
                let (result, carry) = self.a.overflowing_sub(val);
                let half_carry = (self.a & 0x0F) < (val & 0x0F);

                self.f.set(FlagRegister::C, carry);
                self.f.set(FlagRegister::H, half_carry);
                self.f.set(FlagRegister::N, true);
                self.f.set(FlagRegister::Z, result == 0);
            }
        }
    }

    fn run_rot(&mut self, rot_op: Rot, val: u8, force_zero: bool) -> u8 {
        let (result, carry) = match rot_op {
            Rot::Rlc => {
                let carry = val & 0x80 == 0x80;
                (val.rotate_left(1), carry)
            }
            Rot::Rrc => {
                let carry = val & 0x01 == 0x01;
                (val.rotate_right(1), carry)
            }
            Rot::Rl => {
                let carry = val & 0x80 == 0x80;
                let result = (val << 1)
                    | (if self.f.contains(FlagRegister::C) {
                        1
                    } else {
                        0
                    });
                (result, carry)
            }
            Rot::Rr => {
                let carry = val & 0x01 == 0x01;
                let result = (val >> 1)
                    | (if self.f.contains(FlagRegister::C) {
                        0x80
                    } else {
                        0
                    });
                (result, carry)
            }
            Rot::Sla => {
                let carry = val & 0x80 == 0x80;
                let result = val << 1;
                (result, carry)
            }
            Rot::Sra => {
                // Rust >> is logical by default on u8, need to take msb manually
                let carry = val & 0x01 == 0x01;
                let result = (val >> 1) | (val & 0x80);
                (result, carry)
            }
            Rot::Swap => (((val & 0x0f) << 4) | ((val & 0xf0) >> 4), false),
            Rot::Srl => {
                let carry = val & 0x01 == 0x01;
                let result = val >> 1;
                (result, carry)
            }
        };

        self.f.set(FlagRegister::C, carry);
        self.f.set(FlagRegister::H | FlagRegister::N, false);

        if force_zero {
            self.f.set(FlagRegister::Z, false);
        } else {
            self.f.set(FlagRegister::Z, result == 0);
        }

        result
    }

    fn check_conditional(&mut self, condition: Condition) -> bool {
        match condition {
            Condition::NonZero => !self.f.contains(FlagRegister::Z),
            Condition::Zero => self.f.contains(FlagRegister::Z),
            Condition::NoCarry => !self.f.contains(FlagRegister::C),
            Condition::Carry => self.f.contains(FlagRegister::C),
        }
    }

    fn get_register(&self, reg: Register) -> u8 {
        match reg {
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::A => self.a,
        }
    }

    fn set_register(&mut self, reg: Register, val: u8) {
        match reg {
            Register::B => self.b = val,
            Register::C => self.c = val,
            Register::D => self.d = val,
            Register::E => self.e = val,
            Register::H => self.h = val,
            Register::L => self.l = val,
            Register::A => self.a = val,
        }
    }

    fn get_register_pair(&self, reg: RegisterPair) -> u16 {
        match reg {
            RegisterPair::BC => ((self.b as u16) << 8) | (self.c as u16),
            RegisterPair::DE => ((self.d as u16) << 8) | (self.e as u16),
            RegisterPair::HL => ((self.h as u16) << 8) | (self.l as u16),
            RegisterPair::SP => self.sp,
            RegisterPair::AF => ((self.a as u16) << 8) | ((self.f.bits & 0xF0) as u16),
        }
    }

    fn set_register_pair(&mut self, reg: RegisterPair, val: u16) {
        match reg {
            RegisterPair::BC => {
                self.b = (val >> 8) as u8;
                self.c = (val & 0x00FF) as u8
            }
            RegisterPair::DE => {
                self.d = (val >> 8) as u8;
                self.e = (val & 0x00FF) as u8
            }
            RegisterPair::HL => {
                self.h = (val >> 8) as u8;
                self.l = (val & 0x00FF) as u8
            }
            RegisterPair::SP => {
                self.sp = val;
            }
            RegisterPair::AF => {
                self.a = (val >> 8) as u8;
                self.f.bits = (val & 0x00F0) as u8
            }
        }
    }

    fn handle_oam_dma(&mut self, bus: &mut CpuBus) {
        // OAM DMA
        let mut oam_dma = bus.get_oam_dma();
        let (is_oam_dma, reset_oam_dma) = match &mut oam_dma {
            OamDma {
                cycle: Some(c),
                source,
            } => {
                // Each cycle, DMA reads and write one byte from source to destination
                let data = bus.read_without_dma_check(((*source as u16) << 8) | (*c as u16), true);
                bus.write_without_dma_check(0xFE00 | ((*c & 0xFF) as u16), data, true);
                *c += 1;

                (true, *c >= 0xA0)
            }
            _ => {
                // No DMA currently
                (false, false)
            }
        };

        // Borrow checker hack to update with the new state
        if reset_oam_dma {
            // Stop DMA
            oam_dma.cycle = None
        }
        if is_oam_dma {
            // Update the value
            bus.set_oam_dma(oam_dma);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cartridge;
    use crate::CgbDoubleSpeed;
    use crate::InterruptState;
    use crate::JoypadState;
    use crate::OamDma;
    use crate::Ppu;
    use crate::RomParserError;
    use crate::SerialPort;
    use crate::TimerRegisters;
    use crate::WRAM_BANK_SIZE;
    use alloc::vec;

    struct MockEmulator {
        pub cartridge: Cartridge,
        pub cpu: Cpu,
        pub wram: [u8; WRAM_BANK_SIZE as usize * 8],
        pub hram: [u8; 0x7F],
        pub interrupts: InterruptState,
        pub double_speed: CgbDoubleSpeed,
        pub oam_dma: OamDma,
        pub timer_registers: TimerRegisters,
        pub serial_port: SerialPort,
        pub joypad_state: JoypadState,
        pub joypad_register: u8,
        pub ppu: Ppu,
    }

    impl MockEmulator {
        pub fn new() -> Result<Self, RomParserError> {
            let mut rom = vec![0; 0x200];
            rom[0x14d] = 231;
            let cartridge = Cartridge::load(&rom, None)?;

            let emulator = Self {
                cartridge,
                cpu: Default::default(),
                wram: [0u8; WRAM_BANK_SIZE as usize * 8],
                hram: [0u8; 0x7F],
                interrupts: Default::default(),
                double_speed: Default::default(),
                oam_dma: Default::default(),
                timer_registers: Default::default(),
                serial_port: Default::default(),
                joypad_state: Default::default(),
                joypad_register: 0,
                ppu: Default::default(),
            };

            Ok(emulator)
        }
    }

    /// Executes `n` instructions and returns
    fn execute_n(emu: &mut MockEmulator, n: usize) {
        let mut bus = borrow_cpu_bus!(emu);
        for _ in 0..n {
            loop {
                // Because of the fetch-execute overlap, running the last cycle fetches the next
                // instruction. We need to run and break in this case to go to the next n
                if emu.cpu.cycles == 1 {
                    emu.cpu.clock(&mut bus);
                    break;
                } else {
                    emu.cpu.clock(&mut bus);
                }
            }
        }
    }

    #[test]
    fn test_ld_rr() {
        let mut emu = MockEmulator::new().unwrap();

        // This could be in rom, but we'd set the pc to 0x150 to skip the header entry point anyway
        emu.cpu.pc = 0xC000;
        emu.wram[0] = 0x40; // B,B
        emu.wram[1] = 0x41; // B,C
        emu.wram[2] = 0x42; // B,D
        emu.wram[3] = 0x43; // B,E
        emu.wram[4] = 0x44; // B,H
        emu.wram[5] = 0x45; // B,L
        emu.wram[6] = 0x47; // B,A
        emu.wram[7] = 0x78; // A,B
        emu.wram[8] = 0x60; // H,B
        emu.wram[9] = 0x6A; // L,D

        emu.cpu.b = 1;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 1);

        emu.cpu.c = 2;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 2);

        emu.cpu.d = 3;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 3);

        emu.cpu.e = 4;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 4);

        emu.cpu.h = 5;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 5);

        emu.cpu.l = 6;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 6);

        emu.cpu.a = 7;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.b, 7);

        emu.cpu.b = 20;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.a, 20);

        emu.cpu.b = 21;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.h, 21);

        emu.cpu.d = 30;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.l, 30);
    }

    #[test]
    fn test_ld_r_imm() {
        let mut emu = MockEmulator::new().unwrap();

        emu.cpu.pc = 0xC000;
        emu.wram[0] = 0x06; // B,n
        emu.wram[1] = 1;
        emu.wram[2] = 0x3E; // A,n
        emu.wram[3] = 255;

        execute_n(&mut emu, 2);
        assert_eq!(emu.cpu.b, 1);
        assert_eq!(emu.cpu.a, 255);
    }

    #[test]
    fn test_ld_r_mem() {
        let mut emu = MockEmulator::new().unwrap();

        emu.cpu.pc = 0xC000;
        emu.wram[0] = 0xFA; // A,(nn)
        emu.wram[1] = 0x00;
        emu.wram[2] = 0xD0;
        emu.wram[3] = 0x7E; // A,(HL)
        emu.wram[0x1000] = 20;
        emu.wram[0x1010] = 42;

        emu.cpu.h = 0xD0;
        emu.cpu.l = 0x10;

        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.a, 20);

        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.a, 42);
    }

    #[test]
    fn test_ldh() {
        let mut emu = MockEmulator::new().unwrap();

        emu.cpu.pc = 0xC000;
        emu.wram[0] = 0xF2; // A,(0xFF00 + C)
        emu.wram[1] = 0xF0; // A,(0xFF00 + n)
        emu.wram[2] = 0xA0;
        emu.hram[0x10] = 42; // At 0xFF80+0x10
        emu.hram[0x20] = 69; // At 0xFF80+0x20
        emu.cpu.c = 0x90;

        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.a, 42);

        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.a, 69);
    }

    #[test]
    fn test_ld16_r_imm() {
        let mut emu = MockEmulator::new().unwrap();

        emu.cpu.pc = 0xC000;
        emu.wram[0] = 0x01; // BC,nn
        emu.wram[1] = 0x10; // lsb
        emu.wram[2] = 0x20; // msb
        emu.wram[3] = 0x11; // DE,nn
        emu.wram[4] = 0x30; // lsb
        emu.wram[5] = 0x40; // msb
        emu.wram[6] = 0x21; // HL,nn
        emu.wram[7] = 0x50; // lsb
        emu.wram[8] = 0x60; // msb
        emu.wram[9] = 0x31; // SP,nn
        emu.wram[10] = 0x70; // lsb
        emu.wram[11] = 0x80; // msb

        execute_n(&mut emu, 4);
        assert_eq!(emu.cpu.b, 0x20);
        assert_eq!(emu.cpu.c, 0x10);
        assert_eq!(emu.cpu.d, 0x40);
        assert_eq!(emu.cpu.e, 0x30);
        assert_eq!(emu.cpu.h, 0x60);
        assert_eq!(emu.cpu.l, 0x50);
        assert_eq!(emu.cpu.sp, 0x8070);
    }

    #[test]
    fn test_push() {
        let mut emu = MockEmulator::new().unwrap();

        emu.cpu.pc = 0xC000;
        emu.wram[0] = 0xC5; // BC

        emu.cpu.sp = 0xC500;
        emu.cpu.b = 0x10;
        emu.cpu.c = 0x20;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.sp, 0xC4FE);
        assert_eq!(emu.wram[0x4FF], 0x10);
        assert_eq!(emu.wram[0x4FE], 0x20);
    }

    #[test]
    fn test_pop() {
        let mut emu = MockEmulator::new().unwrap();

        emu.cpu.pc = 0xC000;
        emu.wram[0] = 0xC1; // BC
        emu.wram[0x4FE] = 0x20;
        emu.wram[0x4FF] = 0x10;

        emu.cpu.sp = 0xC4FE;
        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.sp, 0xC500);
        assert_eq!(emu.cpu.b, 0x10);
        assert_eq!(emu.cpu.c, 0x20);
    }

    #[test]
    fn test_jump() {
        let mut emu = MockEmulator::new().unwrap();

        emu.cpu.f.bits = 0;
        emu.cpu.pc = 0xC000;
        emu.wram[0] = 0xC3; // jp immediate
        emu.wram[1] = 0x00;
        emu.wram[2] = 0xD0;
        emu.wram[0x1000] = 0xCA; // jp zero (fail)
        emu.wram[0x1001] = 0x50;
        emu.wram[0x1002] = 0xD0;
        emu.wram[0x1003] = 0xC2; // jp non-zero
        emu.wram[0x1004] = 0x50;
        emu.wram[0x1005] = 0xD0;
        emu.wram[0x1050] = 0x18; // jp relative
        emu.wram[0x1051] = 0xEE; // -0x12 when signed, pc will be 0x1052 after this
        emu.wram[0x1040] = 0x20; // jp relative non-zero
        emu.wram[0x1041] = 0x1E;

        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.pc, 0xD000 + 1); // +1 because of fetch-execute overlap occured

        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.pc, 0xD003 + 1);

        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.pc, 0xD050 + 1);

        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.pc, 0xD040 + 1);

        execute_n(&mut emu, 1);
        assert_eq!(emu.cpu.pc, 0xD060 + 1);
    }
}
