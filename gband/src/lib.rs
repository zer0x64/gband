#![no_std]

extern crate alloc;

#[macro_use]
mod bus;

mod cartridge;
mod cpu;
mod ppu;
mod enums;

pub use cpu::Cpu;
pub use ppu::{ Ppu, Frame };
pub use cartridge::RomParserError;

use cartridge::Cartridge;
use enums::ExecutionMode;

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

    // == Emulation Specific Data == //
    execution_mode: ExecutionMode,
    clock_count: u8,
}

impl Emulator {
    pub fn new(rom: &[u8], save_data: Option<&[u8]>) ->  Result<Self, RomParserError> {
        let cartridge = Cartridge::load(rom, save_data)?;
        let execution_mode = cartridge.execution_mode;

        let mut emulator = Self {
            cartridge,
            cpu: Default::default(),
            wram: [0u8; WRAM_BANK_SIZE as usize * 4],

            ppu: Default::default(),

            execution_mode,
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
}


#[test]
fn test() {
    let rom = [0u8; 10];
    let mut emu = Emulator::new(&rom, None).unwrap();

    for _ in 0..10 {
        emu.clock();
    }
}