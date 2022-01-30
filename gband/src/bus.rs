use crate::Cartridge;
use crate::JoypadState;
use crate::Ppu;
use crate::WRAM_BANK_SIZE;

// TODO: Revert macro_export added for criterion
#[macro_export]
macro_rules! borrow_cpu_bus {
    ($owner:ident) => {{
        $crate::bus::CpuBus::borrow(&mut $owner.wram,
            &mut $owner.cartridge,
            &mut $owner.ppu,
            &$owner.joypad_state,
            &mut $owner.joypad_register)
    }};
}

pub struct CpuBus<'a> {
    wram: &'a mut [u8; WRAM_BANK_SIZE as usize * 8],
    cartridge: &'a mut Cartridge,
    ppu: &'a mut Ppu,
    joypad_state: &'a JoypadState,
    joypad_register: &'a mut u8,
}

impl<'a> CpuBus<'a> {
    #[allow(clippy::too_many_arguments)] // it's fine, it's used by a macro
    pub fn borrow(
        wram: &'a mut [u8; WRAM_BANK_SIZE as usize * 8],
        cartridge: &'a mut Cartridge,
        ppu: &'a mut Ppu,
        joypad_state: &'a JoypadState,
        joypad_register: &'a mut u8,
    ) -> Self {
        Self {
            wram,
            cartridge,
            ppu,
            joypad_state,
            joypad_register,
        }
    }
}

impl CpuBus<'_> {
    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7fff => {
                // Cartridge
                self.write_cartridge(addr, data)
            },
            0xA000..=0xBFFF => {
                // Cartridge RAM
                self.write_cartridge(addr, data)
            },
            0xC000..=0xFDFF => {
                // WRAM
                self.write_ram(addr, data)
            },
            0xFF00 => {
                // Joypad
                self.write_joypad_reg(data)
            },
            _ => {
                // TODO: handle full memory map
            }
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => {
                // Cartridge
                self.read_cartridge(addr)
            },
            0xA000..=0xBFFF => {
                // Cartridge RAM
                self.read_cartridge(addr)
            },
            0xC000..=0xFDFF => {
                // WRAM
                self.read_ram(addr)
            },
            0xFF00 => {
                // Joypad
                self.read_joypad_reg()
            },
            _ => {
                // TODO: handle full memory map
                0
            }
        }
    }

    pub fn write_ram(&mut self, addr: u16, data: u8) {
        // TODO: Bank switching
        // For now, allow access to the first 2 banks, which are classic GB banks (not switchable)
        self.wram[(addr & (WRAM_BANK_SIZE * 2 - 1)) as usize] = data;
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        // TODO: Bank switching
        // For now, allow access to the first 2 banks, which are classic GB banks (not switchable)
        self.wram[(addr & (WRAM_BANK_SIZE * 2 - 1)) as usize]
    }

    pub fn write_cartridge(&mut self, addr: u16, data: u8) {
        self.cartridge.write(addr, data)
    }

    pub fn read_cartridge(&self, addr: u16) -> u8 {
        self.cartridge.read(addr)
    }

    pub fn write_joypad_reg(&mut self, data: u8) {
        let state: u8 = (*self.joypad_state).bits();

        // Defaults to no button pressed
        *self.joypad_register = 0;

        if data & 0x10 == 0 {
            // If bit 4 is set to 0, handle D-pad
            *self.joypad_register |= state & 0x0F;
        };

        if data & 0x20 == 0 {
            // If bit 5 is set to 0, handle the other buttons
            *self.joypad_register |= (state & 0xF0) >> 4;
        }

        // Invert button presses
        *self.joypad_register = !*self.joypad_register;

        // Mask to get only the buttons + add the input bits
        *self.joypad_register = (*self.joypad_register & 0x0F) | (data & 0x30)
    }

    pub fn read_joypad_reg(&self) -> u8 {
        *self.joypad_register
    }
}
