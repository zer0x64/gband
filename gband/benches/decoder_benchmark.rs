use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use gband::{
    borrow_cpu_bus, Cartridge, Cpu, InterruptState, JoypadState, OamDma, Ppu, RomParserError,
};
use std::time::Duration;

struct MockEmulator {
    pub cartridge: Cartridge,
    pub cpu: Cpu,
    pub wram: [u8; 0x1000 as usize * 8],
    pub hram: [u8; 0x7F],
    pub interrupts: InterruptState,
    pub oam_dma: OamDma,
    pub joypad_state: JoypadState,
    pub joypad_register: u8,
    pub ppu: Ppu,
    pub serial_port_buffer: Vec<u8>,
}

impl MockEmulator {
    pub fn new() -> Result<Self, RomParserError> {
        let mut rom = vec![0; 0x200];
        rom[0x14d] = 231;
        let cartridge = Cartridge::load(&rom, None)?;

        let emulator = Self {
            cartridge,
            cpu: Default::default(),
            wram: [0u8; 0x1000 as usize * 8],
            hram: [0u8; 0x7F],
            interrupts: Default::default(),
            oam_dma: Default::default(),
            joypad_state: Default::default(),
            joypad_register: 0,
            ppu: Default::default(),
            serial_port_buffer: Vec::with_capacity(256),
        };

        Ok(emulator)
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cpu");
    group.warm_up_time(Duration::from_millis(500));
    group.sample_size(100);
    group.measurement_time(Duration::from_millis(500));

    // LdRMem, Add16HL, LdRImm, LdRR, LdRR, LdRR, LdRMem, LdRMem, LdRR, AluR, LdMemR
    //     10,      57,     62,   64,   65,   80,     94,    102,  127,  128,    234
    for opcode in [
        0x0A, 0x39, 0x3E, 0x40, 0x41, 0x50, 0x5E, 0x66, 0x7F, 0x80, 0xEA,
    ] {
        group.bench_with_input(BenchmarkId::new("fetch", opcode), &opcode, |b, opcode| {
            let mut emulator = MockEmulator::new().unwrap();
            emulator.wram[0] = *opcode;
            emulator.wram[1] = 69;
            emulator.wram[2] = 42;

            b.iter(|| {
                let mut cpu_bus = borrow_cpu_bus!(emulator);
                emulator.cpu.fetch(&mut cpu_bus);
            })
        });

        group.bench_with_input(BenchmarkId::new("execute", opcode), &opcode, |b, opcode| {
            let mut emulator = MockEmulator::new().unwrap();
            emulator.wram[0] = *opcode;
            emulator.wram[1] = 69;
            emulator.wram[2] = 42;

            b.iter(|| {
                let mut cpu_bus = borrow_cpu_bus!(emulator);
                emulator.cpu.execute(&mut cpu_bus);
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
