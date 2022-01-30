use std::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use gband::{borrow_cpu_bus, Cpu, Ppu, Cartridge, JoypadState, RomParserError};

struct MockEmulator {
    pub cartridge: Cartridge,
    pub cpu: Cpu,
    pub wram: [u8; 0x1000 as usize * 8],
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
            wram: [0u8; 0x1000 as usize * 8],
            joypad_state: Default::default(),
            joypad_register: 0,
            ppu: Default::default(),
        };

        Ok(emulator)
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cpu");
    group.warm_up_time(Duration::from_millis(500));
    group.sample_size(100);
    group.measurement_time(Duration::from_millis(500));

    for opcode in [0x0A, 0x39, 0x3E, 0x40, 0x41, 0x50, 0x5E, 0x66, 0x7F, 0x80, 0xEA] {
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
