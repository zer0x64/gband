use crate::oam_dma::OamDma;
use crate::Cartridge;
use crate::Cpu;
use crate::InterruptReg;
use crate::InterruptState;
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
            &mut $owner.interrupts,
            &mut $owner.oam_dma,
            &mut $owner.cartridge,
            &mut $owner.ppu,
            &$owner.joypad_state,
            &mut $owner.joypad_register,
            &mut $owner.serial_port_buffer,
        )
    }};
}

pub struct CpuBus<'a> {
    wram: &'a mut [u8; WRAM_BANK_SIZE as usize * 8],
    hram: &'a mut [u8; 0x7F],
    interrupts: &'a mut InterruptState,
    oam_dma: &'a mut OamDma,
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
        interrupts: &'a mut InterruptState,
        oam_dma: &'a mut OamDma,
        cartridge: &'a mut Cartridge,
        ppu: &'a mut Ppu,
        joypad_state: &'a JoypadState,
        joypad_register: &'a mut u8,
        serial_port_buffer: &'a mut alloc::vec::Vec<u8>,
    ) -> Self {
        Self {
            wram,
            hram,
            interrupts,
            oam_dma,
            cartridge,
            ppu,
            joypad_state,
            joypad_register,
            serial_port_buffer,
        }
    }
}

impl CpuBus<'_> {
    pub fn write(&mut self, addr: u16, data: u8) {
        match self.oam_dma {
            OamDma {
                cycle: Some(_),
                source,
            } => {
                // Wraps regular CPU writes to disallow conflicting bus access during OAM_DMA
                if !Self::check_oam_dma_bus_conflict(*source, addr) {
                    self.write_without_dma_check(addr, data, false)
                }
            }
            _ => self.write_without_dma_check(addr, data, false),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match self.oam_dma.clone() {
            OamDma {
                cycle: Some(_),
                source,
            } => {
                // Wraps regular CPU reads to disallow conflicting bus access during OAM_DMA
                if !Self::check_oam_dma_bus_conflict(source, addr) {
                    self.read_without_dma_check(addr, false)
                } else {
                    0xFF
                }
            }
            _ => self.read_without_dma_check(addr, false),
        }
    }

    pub fn write_without_dma_check(&mut self, addr: u16, data: u8, called_from_dma: bool) {
        match addr {
            0x0000..=0x7fff => {
                // Cartridge
                self.write_cartridge(addr, data)
            }
            0x8000..=0x9FFF => {
                // VRAM
                self.ppu.write_vram(addr, data)
            }
            0xA000..=0xBFFF => {
                // Cartridge RAM
                self.write_cartridge(addr, data)
            }
            0xC000..=0xFDFF => {
                // WRAM
                self.write_ram(addr, data)
            }
            0xFE00..=0xFE9F => {
                // OAM
                self.ppu.write_oam(addr, data, called_from_dma)
            }
            0xFF00 => {
                // Joypad
                self.write_joypad_reg(data)
            }
            0xFF01 => {
                // Serial transfer data (SB)
                if data == 10u8 {
                    if !self.serial_port_buffer.is_empty() {
                        log::info!(
                            "Serial port: {}",
                            self.serial_port_buffer
                                .iter()
                                .flat_map(|c| (*c as char).escape_default())
                                .collect::<alloc::string::String>()
                        );
                        self.serial_port_buffer.clear();
                    }
                } else {
                    self.serial_port_buffer.push(data);
                }
            }
            0xFF02 => {
                // Serial transfer control (SC)
            }
            0xFF46 => {
                // OAM DMA
                self.request_oam_dma(data)
            }
            0xFF40..=0xFF45 | 0xFF47..=0xFF6F => {
                // PPU control reg
                self.ppu.write(addr, data)
            }
            0xFF0F => self.interrupts.status = InterruptReg::from_bits_truncate(data),
            0xFF80..=0xFFFE => self.hram[(addr & 0x7E) as usize] = data,
            0xFFFF => self.interrupts.enable = InterruptReg::from_bits_truncate(data),
            _ => {
                // TODO: handle full memory map
            }
        }
    }

    pub fn read_without_dma_check(&self, addr: u16, called_from_dma: bool) -> u8 {
        match addr {
            0x0000..=0x7fff => {
                // Cartridge
                self.read_cartridge(addr)
            }
            0x8000..=0x9FFF => {
                // VRAM
                self.ppu.read_vram(addr)
            }
            0xA000..=0xBFFF => {
                // Cartridge RAM
                self.read_cartridge(addr)
            }
            0xC000..=0xFDFF => {
                // WRAM
                self.read_ram(addr)
            }
            0xFE00..=0xFE9F => {
                // OAM
                self.ppu.read_oam(addr, called_from_dma)
            }
            0xFF00 => {
                // Joypad
                self.read_joypad_reg()
            }
            0xFF0F => self.interrupts.status.bits(),
            0xFF46 => {
                // OAM DMA
                self.read_oam_dma()
            }
            0xFF40..=0xFF45 | 0xFF47..=0xFF6F => {
                // PPU control reg
                self.ppu.read(addr)
            }
            0xFF80..=0xFFFE => self.hram[(addr & 0x7E) as usize],
            0xFFFF => self.interrupts.enable.bits(),
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

    pub fn request_oam_dma(&mut self, source: u8) {
        // Mirror 0xE0-0xFF to 0xC0-0xDF by removing a specific bit
        let mask = if source & 0xc0 == 0xc0 { !0x20 } else { 0 };

        *self.oam_dma = OamDma::new(source & mask);
    }

    pub fn read_oam_dma(&self) -> u8 {
        self.oam_dma.source
    }

    pub fn get_oam_dma(&self) -> OamDma {
        self.oam_dma.clone()
    }

    pub fn set_oam_dma(&mut self, oam_dma: OamDma) {
        *self.oam_dma = oam_dma;
    }

    fn check_oam_dma_bus_conflict(source: u8, addr: u16) -> bool {
        // Bus on CGB are emulated.
        match (source, addr) {
            // ROM and SRAM shares the same bus
            (0x00..=0x7F | 0xA0..=0xBF, 0x0000..=0x7FFF | 0xA000..=0xBFFF) => true,
            // WRAM has it's own bus.
            (0xC0..=0xFD, 0xC000..=0xFDFF) => true,
            // VRAM has it's own bus, which is always blocked because it's the destination
            (_, 0x8000..=0x9FFF) => true,
            _ => false,
        }
    }
}

#[macro_export]
macro_rules! borrow_ppu_bus {
    ($owner:ident) => {{
        $crate::bus::PpuBus::borrow(&mut $owner.interrupts)
    }};
}

pub struct PpuBus<'a> {
    interrupts: &'a mut InterruptState,
}

impl<'a> PpuBus<'a> {
    #[allow(clippy::too_many_arguments)] // it's fine, it's used by a macro
    pub fn borrow(interrupts: &'a mut InterruptState) -> Self {
        Self { interrupts }
    }
}

impl PpuBus<'_> {
    pub fn get_interrupt_state(&self) -> InterruptState {
        *self.interrupts
    }

    pub fn set_interrupt_state(&mut self, interrupts: InterruptState) {
        *self.interrupts = interrupts
    }

    pub fn request_interrupt(&mut self, interrupt: InterruptReg) {
        self.interrupts.status.insert(interrupt)
    }
}
