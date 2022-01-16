use crate::bus::CpuBus;

pub struct Cpu {
    pub pc: u16,
    pub cycles: u8,
}

impl Default for Cpu {
    fn default() -> Self {
        // TODO
        Self { pc: 0, cycles: 0 }
    }
}

impl Cpu {
    pub fn clock(&mut self, _bus: &mut CpuBus) {
        // TODO
    }
}
