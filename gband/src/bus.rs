use crate::dma::*;
use crate::Cartridge;
use crate::CgbDoubleSpeed;
use crate::InterruptReg;
use crate::InterruptState;
use crate::JoypadState;
use crate::Ppu;
use crate::SerialPort;
use crate::TimerRegisters;
use crate::WRAM_BANK_SIZE;

use crate::ppu::FifoMode;

// TODO: Revert macro_export added for criterion
#[macro_export]
macro_rules! borrow_cpu_bus {
    ($owner:ident) => {{
        $crate::bus::CpuBus::borrow(
            &mut $owner.wram,
            &mut $owner.wram_bank,
            &mut $owner.hram,
            &mut $owner.interrupts,
            &mut $owner.double_speed,
            &mut $owner.oam_dma,
            &mut $owner.hdma,
            &mut $owner.timer_registers,
            &mut $owner.cartridge,
            &mut $owner.ppu,
            &mut $owner.cgb_mode,
            &mut $owner.serial_port,
            &$owner.joypad_state,
            &mut $owner.joypad_register,
        )
    }};
}

pub struct CpuBus<'a> {
    wram: &'a mut [u8; WRAM_BANK_SIZE as usize * 8],
    wram_bank: &'a mut u8,
    hram: &'a mut [u8; 0x7F],
    interrupts: &'a mut InterruptState,
    double_speed: &'a mut CgbDoubleSpeed,
    oam_dma: &'a mut OamDma,
    hdma: &'a mut HDma,
    timer_registers: &'a mut TimerRegisters,
    cartridge: &'a mut Cartridge,
    ppu: &'a mut Ppu,
    cgb_mode: &'a mut bool,
    serial_port: &'a mut SerialPort,
    joypad_state: &'a JoypadState,
    joypad_register: &'a mut u8,
}

impl<'a> CpuBus<'a> {
    #[allow(clippy::too_many_arguments)] // it's fine, it's used by a macro
    pub fn borrow(
        wram: &'a mut [u8; WRAM_BANK_SIZE as usize * 8],
        wram_bank: &'a mut u8,
        hram: &'a mut [u8; 0x7F],
        interrupts: &'a mut InterruptState,
        double_speed: &'a mut CgbDoubleSpeed,
        oam_dma: &'a mut OamDma,
        hdma: &'a mut HDma,
        timer_registers: &'a mut TimerRegisters,
        cartridge: &'a mut Cartridge,
        ppu: &'a mut Ppu,
        cgb_mode: &'a mut bool,
        serial_port: &'a mut SerialPort,
        joypad_state: &'a JoypadState,
        joypad_register: &'a mut u8,
    ) -> Self {
        Self {
            wram,
            wram_bank,
            hram,
            interrupts,
            double_speed,
            oam_dma,
            hdma,
            timer_registers,
            cartridge,
            ppu,
            cgb_mode,
            serial_port,
            joypad_state,
            joypad_register,
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
                self.serial_port.set_buffer(data);
            }
            0xFF02 => {
                // Serial transfer control (SC)
                self.serial_port.set_control(data);
            }
            0xFF04..=0xFF07 => self.timer_registers.write(addr, data),
            0xFF0F => self.interrupts.status = InterruptReg::from_bits_truncate(0xE0 | data),
            0xFF46 => {
                // OAM DMA
                self.request_oam_dma(data)
            }
            0xFF40 => {
                // LCD control reg
                let was_enabled = self.ppu.is_enabled();
                self.ppu.write(addr, data);
                let is_enabled = self.ppu.is_enabled();

                if was_enabled && !is_enabled {
                    // The first 16 bytes of HDMA are copied when LCD is disabled
                    if self.hdma.control & 0x80 == 0 && self.hdma.hblank_mode {
                        if let &FifoMode::Drawing(_) = self.ppu.get_mode() {
                            self.hdma.hblank_latch = true;
                        };
                    };

                    self.ppu.disable();
                };
            }
            0xFF40..=0xFF45 | 0xFF47..=0xFF4C | 0xFF4E..=0xFF50 | 0xFF56..=0xFF6F => {
                // PPU control regs
                self.ppu.write(addr, data)
            }
            0xFF4D => {
                // KEY1
                self.double_speed
                    .set(CgbDoubleSpeed::PENDING, (data & 1) != 0)
            }
            0xFF51..=0xFF55 => {
                // HDMA
                self.write_hdma(addr, data)
            }
            0xFF70 => *self.wram_bank = data,
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = data,
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
            0xFF01 => {
                // Serial transfer data (SB)
                self.serial_port.get_buffer()
            }
            0xFF02 => {
                // Serial transfer control (SC)
                self.serial_port.get_control()
            }
            0xFF04..=0xFF07 => self.timer_registers.read(addr),
            0xFF0F => self.interrupts.status.bits(),
            0xFF26 => {
                // NR52, mock for now to make Zelda games work
                0x00
            }
            0xFF46 => {
                // OAM DMA
                self.read_oam_dma()
            }
            0xFF40..=0xFF45 | 0xFF47..=0xFF4C | 0xFF4E..=0xFF50 | 0xFF56..=0xFF6F => {
                // PPU control reg
                self.ppu.read(addr)
            }
            0xFF4D => {
                // KEY1
                self.double_speed.bits()
            }
            0xFF51..=0xFF55 => {
                // HDMA
                self.read_hdma(addr)
            }
            0xFF70 => *self.wram_bank,
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            0xFFFF => self.interrupts.enable.bits(),
            _ => {
                // TODO: handle full memory map
                0xFF
            }
        }
    }

    pub fn write_ram(&mut self, addr: u16, data: u8) {
        // In CGB mode, there is WRAM bank switching
        if *self.cgb_mode {
            let bank = if addr & WRAM_BANK_SIZE > 0 {
                // This is the bank location
                match *self.wram_bank & 7 {
                    0 => 1,
                    x => x,
                }
            } else {
                0
            };

            let addr = addr & (WRAM_BANK_SIZE - 1);
            let bank = (bank as u16) << 12;

            self.wram[(bank | addr) as usize] = data;
        } else {
            self.wram[(addr & (WRAM_BANK_SIZE * 2 - 1)) as usize] = data;
        }
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        // In CGB mode, there is WRAM bank switching
        if *self.cgb_mode {
            let bank = if addr & WRAM_BANK_SIZE > 0 {
                // This is the bank location
                match *self.wram_bank & 7 {
                    0 => 1,
                    x => x,
                }
            } else {
                0
            };

            let addr = addr & (WRAM_BANK_SIZE - 1);
            let bank = (bank as u16) << 12;

            self.wram[(bank | addr) as usize]
        } else {
            self.wram[(addr & (WRAM_BANK_SIZE * 2 - 1)) as usize]
        }
    }

    pub fn write_cartridge(&mut self, addr: u16, data: u8) {
        self.cartridge.write(addr, data)
    }

    pub fn read_cartridge(&self, addr: u16) -> u8 {
        self.cartridge.read(addr)
    }

    fn write_hdma(&mut self, addr: u16, data: u8) {
        match addr {
            0xFF51 => {
                self.hdma.source = (self.hdma.source & 0xFF) | ((data as u16) << 8);
            }
            0xFF52 => {
                self.hdma.source = (self.hdma.source & 0xFF00) | (data as u16);
            }
            0xFF53 => {
                self.hdma.destination = (self.hdma.destination & 0xFF) | ((data as u16) << 8);
            }
            0xFF54 => {
                self.hdma.destination = (self.hdma.destination & 0xFF00) | (data as u16);
            }
            0xFF55 => {
                // Check if HDMA is already active
                if self.hdma.is_active() && data & 0x80 == 0 {
                    // HDMA is already active, stop it
                    self.hdma.control |= 0x80;
                } else {
                    // HDMA is not active, start it
                    self.hdma.start(data)
                }
            }
            _ => {}
        }
    }

    fn read_hdma(&self, addr: u16) -> u8 {
        match addr {
            0xFF51 => (self.hdma.source >> 8) as u8,
            0xFF52 => (self.hdma.source & 0xFF) as u8,
            0xFF53 => (self.hdma.destination >> 8) as u8,
            0xFF54 => (self.hdma.destination & 0xFF) as u8,
            0xFF55 => self.hdma.control,
            _ => 0xFF,
        }
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

    pub fn toggle_double_speed(&mut self) {
        if self.double_speed.contains(CgbDoubleSpeed::PENDING) {
            self.double_speed.toggle(CgbDoubleSpeed::ENABLED);
            self.double_speed.remove(CgbDoubleSpeed::PENDING);
        }
    }

    pub fn request_oam_dma(&mut self, source: u8) {
        // Mirror 0xE0-0xFF to 0xC0-0xDF by removing a specific bit
        let mask = if source >= 0xE0 { !0x20 } else { 0xFF };

        *self.oam_dma = OamDma::new(source & mask);
    }

    pub fn read_oam_dma(&self) -> u8 {
        self.oam_dma.source
    }

    pub fn get_oam_dma(&self) -> OamDma {
        self.oam_dma.clone()
    }

    pub fn get_hdma(&self) -> HDma {
        self.hdma.clone()
    }

    pub fn get_cgb_mode(&self) -> bool {
        *self.cgb_mode
    }

    pub fn get_double_speed_mode(&self) -> CgbDoubleSpeed {
        *self.double_speed
    }

    pub fn set_oam_dma(&mut self, oam_dma: OamDma) {
        *self.oam_dma = oam_dma;
    }

    pub fn set_hdma(&mut self, hdma: HDma) {
        *self.hdma = hdma
    }

    pub fn get_timer_registers(&mut self) -> &mut TimerRegisters {
        self.timer_registers
    }

    pub fn get_serial_port(&mut self) -> &mut SerialPort {
        self.serial_port
    }

    pub fn request_interrupt(&mut self, interrupt: InterruptReg) {
        self.interrupts.status.insert(interrupt)
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

    #[cfg(feature = "debugger")]
    pub fn get_cartridge_rom_bank(&self) -> u8 {
        self.cartridge.get_rom_bank()
    }

    #[cfg(feature = "debugger")]
    pub fn get_cartridge_ram_bank(&self) -> u8 {
        self.cartridge.get_ram_bank()
    }
}

#[macro_export]
macro_rules! borrow_ppu_bus {
    ($owner:ident) => {{
        $crate::bus::PpuBus::borrow(&mut $owner.interrupts, &mut $owner.hdma)
    }};
}

pub struct PpuBus<'a> {
    interrupts: &'a mut InterruptState,
    hdma: &'a mut HDma,
}

impl<'a> PpuBus<'a> {
    #[allow(clippy::too_many_arguments)] // it's fine, it's used by a macro
    pub fn borrow(interrupts: &'a mut InterruptState, hdma: &'a mut HDma) -> Self {
        Self { interrupts, hdma }
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

    pub fn set_hdma_hblank(&mut self, value: bool) {
        if value {
            // Only set the latch if HDMA is in progress and in HBLANK mode.
            if self.hdma.hblank_mode && self.hdma.control & 0x80 == 0 {
                self.hdma.hblank_latch = true;
            }
        } else {
            self.hdma.hblank_latch = false;
        }
    }
}
