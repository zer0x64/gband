use crate::Cartridge;
use crate::Ppu;
use crate::WRAM_BANK_SIZE;

macro_rules! borrow_cpu_bus {
    ($owner:ident) => {{
        $crate::bus::CpuBus::borrow(&mut $owner.wram, &mut $owner.cartridge, &mut $owner.ppu)
    }};
}

pub struct CpuBus<'a> {
    wram: &'a mut [u8; WRAM_BANK_SIZE as usize * 4],
    cartridge: &'a mut Cartridge,
    ppu: &'a mut Ppu,
}

impl<'a> CpuBus<'a> {
    #[allow(clippy::too_many_arguments)] // it's fine, it's used by a macro
    pub fn borrow(
        wram: &'a mut [u8; WRAM_BANK_SIZE as usize * 4],
        cartridge: &'a mut Cartridge,
        ppu: &'a mut Ppu,
    ) -> Self {
        Self {
            wram,
            cartridge,
            ppu,
        }
    }
}

impl CpuBus<'_> {
    pub fn write_ram(&mut self, addr: u16, data: u8) {
        // TODO: Bank switching
        self.wram[(addr & (WRAM_BANK_SIZE * 4 - 1)) as usize] = data;
    }

    pub fn read_ram(&mut self, addr: u16) -> u8 {
        // TODO: Bank switching
        self.wram[(addr & ((WRAM_BANK_SIZE * 4) - 1)) as usize]
    }
}
