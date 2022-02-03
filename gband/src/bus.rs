use crate::Cartridge;
use crate::JoypadState;
use crate::Ppu;
use crate::WRAM_BANK_SIZE;

// TODO: Revert macro_export added for criterion
#[macro_export]
macro_rules! borrow_cpu_bus {
    ($owner:ident) => {{
        $crate::bus::CpuBus::borrow(
            &mut $owner.wram,
            &mut $owner.hram,
            &mut $owner.cartridge,
            &mut $owner.ppu,
            &$owner.joypad_state,
            &mut $owner.joypad_register,
            &mut $owner.serial_port_buffer
        )
    }};
}

pub struct CpuBus<'a> {
    wram: &'a mut [u8; WRAM_BANK_SIZE as usize * 8],
    hram: &'a mut [u8; 0x7F],
    cartridge: &'a mut Cartridge,
    ppu: &'a mut Ppu,
    joypad_state: &'a JoypadState,
    joypad_register: &'a mut u8,
    serial_port_buffer: &'a mut alloc::vec::Vec<u8>,
}

impl<'a> CpuBus<'a> {
    #[allow(clippy::too_many_arguments)] // it's fine, it's used by a macro
    pub fn borrow(
        wram: &'a mut [u8; WRAM_BANK_SIZE as usize * 8],
        hram: &'a mut [u8; 0x7F],
        cartridge: &'a mut Cartridge,
        ppu: &'a mut Ppu,
        joypad_state: &'a JoypadState,
        joypad_register: &'a mut u8,
        serial_port_buffer: &'a mut alloc::vec::Vec<u8>,
    ) -> Self {
        Self {
            wram,
            hram,
            cartridge,
            ppu,
            joypad_state,
            joypad_register,
            serial_port_buffer
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
            0xFF01 => {
                // Serial transfer data (SB)
                if data == 10u8 {
                    if !self.serial_port_buffer.is_empty() {
                        log::info!("Serial port: {}", self.serial_port_buffer
                            .iter()
                            .flat_map(|c| (*c as char).escape_default())
                            .collect::<alloc::string::String>()
                        );
                        self.serial_port_buffer.clear();
                    }
                } else {
                    self.serial_port_buffer.push(data);
                }
            },
            0xFF02 => {
                // Serial transfer control (SC)
            },
            0xFF80..=0xFFFE => {
                self.hram[(addr & 0x7E) as usize] = data
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
            0xFF80..=0xFFFE => {
                self.hram[(addr & 0x7E) as usize]
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
