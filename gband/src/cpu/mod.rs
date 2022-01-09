use crate::bus::CpuBus;

pub struct Cpu { }

impl Default for Cpu {
    fn default() -> Self {
        // TODO
        Self {}
    }
}

impl Cpu {
    pub fn clock(&mut self, _bus: &mut CpuBus) {
        // TODO
    }
}
