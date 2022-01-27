#![no_std]

extern crate alloc;

#[macro_use]
mod bus;

mod cartridge;
mod joypad_state;
mod cpu;
mod ppu;
mod rgb_palette;
pub mod utils;

pub use cartridge::RomParserError;
pub use joypad_state::JoypadState;
pub use cpu::Cpu;
pub use ppu::{Frame, Ppu, FRAME_HEIGHT, FRAME_WIDTH};

use cartridge::Cartridge;

const WRAM_BANK_SIZE: u16 = 0x400;

pub struct Emulator {
    // == Cartridge Related Hardware== //
    cartridge: Cartridge,

    // == CPU Related Hardware == //
    cpu: Cpu,
    wram: [u8; WRAM_BANK_SIZE as usize * 4],

    // == PPU Related Hardware == //
    ppu: Ppu,

    // == IP Related Hardware == //

    // == IO Hardware ==
    joypad_state: JoypadState,
    joypad_register: u8,

    // == Emulation Specific Data == //
    clock_count: u8,
}

impl Emulator {
    pub fn new(rom: &[u8], save_data: Option<&[u8]>) -> Result<Self, RomParserError> {
        let cartridge = Cartridge::load(rom, save_data)?;

        let emulator = Self {
            cartridge,
            cpu: Default::default(),
            wram: [0u8; WRAM_BANK_SIZE as usize * 4],

            ppu: Default::default(),

            joypad_state: Default::default(),
            joypad_register: Default::default(),

            clock_count: 0,
        };

        Ok(emulator)
    }

    pub fn clock(&mut self) -> Option<Frame> {
        // Clock PPU every 2 cycles
        if self.clock_count & 1 == 0 {
            self.ppu.clock();
        };

        // Clock CPU every 4 cycles
        if self.clock_count & 2 == 0 {
            let mut cpu_bus = borrow_cpu_bus!(self);
            self.cpu.clock(&mut cpu_bus);

            self.clock_count = 0;
        };

        self.clock_count += 1;

        self.ppu.ready_frame()
    }

    pub fn set_joypad(&mut self, state: JoypadState) {
        self.joypad_state = state
    }

    pub fn get_save_data(&self) -> Option<&[u8]> {
        self.cartridge.get_save_data()
    }

    #[cfg(feature = "debugger")]
    pub fn disassemble(
        &self,
        _start: u16,
        _end: u16,
    ) -> alloc::vec::Vec<(Option<u8>, u16, alloc::string::String)> {
        // TODO
        alloc::vec::Vec::new()
    }

    #[cfg(feature = "debugger")]
    pub fn mem_dump(&mut self, start: u16, end: u16) -> alloc::vec::Vec<u8> {
        let mut data = alloc::vec::Vec::new();

        // TODO
        /*for addr in start..=end {
            let mut bus = borrow_cpu_bus!(self);
            data.push(self.cpu.mem_dump(&mut bus, addr));
        }*/

        data
    }

    #[cfg(feature = "debugger")]
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }
}

#[test]
fn test() {
    let rom = [0u8; 10];
    let mut emu = Emulator::new(&rom, None).unwrap();

    for _ in 0..10 {
        emu.clock();
    }
}
