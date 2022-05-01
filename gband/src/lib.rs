#![no_std]

extern crate alloc;

#[macro_use]
pub mod bus; // TODO: Revert pub added for criterion

mod cartridge;
mod cgb_double_speed;
mod cpu;
mod dma;
mod interrupt;
mod joypad_state;
mod ppu;
mod rgb_palette;
mod serial;
mod serial_transport;
mod timer_regs;
pub mod utils;

pub use cartridge::RomParserError;
pub use cgb_double_speed::CgbDoubleSpeed;
pub use cpu::Cpu;
pub use interrupt::{InterruptReg, InterruptState};
pub use joypad_state::JoypadState;
pub use ppu::{Frame, Ppu, FRAME_HEIGHT, FRAME_WIDTH};
pub use serial_transport::*;

// TODO: Revert pub added for criterion
pub use cartridge::Cartridge;
pub use dma::*;
pub use serial::SerialPort;
pub use timer_regs::TimerRegisters;

const WRAM_BANK_SIZE: u16 = 0x1000; // 4KiB

pub struct Emulator {
    // == Cartridge Related Hardware== //
    cartridge: Cartridge,

    // == CPU Related Hardware == //
    cpu: Cpu,
    wram: [u8; WRAM_BANK_SIZE as usize * 8],
    wram_bank: u8,

    // 0x7F instead of 0x80 is not a mistake, as the last byte is used to access interupts
    hram: [u8; 0x7F],
    interrupts: InterruptState,
    double_speed: CgbDoubleSpeed,
    oam_dma: OamDma,
    hdma: HDma,
    timer_registers: TimerRegisters,

    // == PPU Related Hardware == //
    ppu: Ppu,
    cgb_mode: bool,

    // == IP Related Hardware == //
    serial_port: SerialPort,

    // == IO Hardware ==
    joypad_state: JoypadState,
    joypad_register: u8,

    // == Emulation Specific Data == //
    clock_count: u8,
}

impl Emulator {
    pub fn new(rom: &[u8], save_data: Option<&[u8]>) -> Result<Self, RomParserError> {
        let cartridge = Cartridge::load(rom, save_data)?;
        let cgb_mode = cartridge.is_cgb();
        let mut ppu = Ppu::new(cgb_mode);
        ppu.set_dmg_colorized_palette(&cartridge.header.title);

        let emulator = Self {
            cartridge,
            cpu: Default::default(),
            interrupts: Default::default(),
            double_speed: Default::default(),
            timer_registers: Default::default(),

            wram: [0u8; WRAM_BANK_SIZE as usize * 8],
            wram_bank: 0xFF,
            hram: [0u8; 0x7F],
            oam_dma: Default::default(),
            hdma: Default::default(),

            ppu,
            cgb_mode,

            serial_port: Default::default(),

            joypad_state: Default::default(),
            joypad_register: Default::default(),

            clock_count: 0,
        };

        Ok(emulator)
    }

    pub fn clock(&mut self) -> Option<Frame> {
        self.clock_count += 1;

        // clock_count is at ~4MHz
        // PPU is clocked at ~4MHz
        let mut ppu_bus = borrow_ppu_bus!(self);
        self.ppu.clock(&mut ppu_bus);

        // We clock CPU on M-cycles, at ~1MHz on regular mode and ~2MHz on CGB double speed mode
        // This means we clock it every 2 or 4 cycles
        let double_speed = self.double_speed.contains(CgbDoubleSpeed::ENABLED);
        if (double_speed && self.clock_count == 2) || self.clock_count == 4 {
            let mut cpu_bus = borrow_cpu_bus!(self);
            self.cpu.clock(&mut cpu_bus);

            if self.clock_count == 4 {
                self.clock_count = 0;
            }
        };

        // Return a frame if available
        self.ppu.ready_frame()
    }

    pub fn set_serial(&mut self, serial: alloc::boxed::Box<dyn SerialTransport>) {
        self.serial_port.set_serial(serial)
    }

    pub fn set_joypad(&mut self, state: JoypadState) {
        self.joypad_state = state
    }

    pub fn get_save_data(&self) -> Option<&[u8]> {
        self.cartridge.get_save_data()
    }

    #[cfg(feature = "debugger")]
    pub fn disassemble(
        &mut self,
        _start: u16,
        _end: u16,
    ) -> alloc::vec::Vec<(u8, u16, alloc::string::String)> {
        let mut bus = borrow_cpu_bus!(self);
        crate::cpu::debugger::disassemble(&mut bus)
    }

    #[cfg(feature = "debugger")]
    pub fn mem_dump(&mut self, start: u16, end: u16) -> alloc::vec::Vec<u8> {
        let mut data = alloc::vec::Vec::new();

        for addr in start..=end {
            let mut bus = borrow_cpu_bus!(self);
            data.push(bus.read(addr));
        }

        data
    }

    #[cfg(feature = "debugger")]
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }
}

#[test]
fn test() {
    let mut rom = [0u8; 0x150];
    rom[0x14d] = 231;
    let mut emu = Emulator::new(&rom, None).unwrap();

    for _ in 0..10 {
        emu.clock();
    }
}
