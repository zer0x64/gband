use super::Mapper;
use crate::cartridge::CartridgeReadTarget;

pub struct Mbc2 {
    bank_mask: usize,
    ram_enable: bool,
    rom_bank_number: u8,
}

impl Mbc2 {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Default for Mbc2 {
    fn default() -> Self {
        Self {
            bank_mask: 0xF,
            ram_enable: false,
            rom_bank_number: 0x01,
        }
    }
}

impl Mapper for Mbc2 {
    fn map_read(&self, addr: u16) -> CartridgeReadTarget {
        match addr {
            0x0000..=0x3FFF => {
                // First bank
                // Fixed to bank 0
                let mask = 0x3FFF;
                CartridgeReadTarget::Rom((addr & mask) as usize)
            }
            0x4000..=0x7FFF => {
                // Switchable bank
                // Maximum of 16 banks supported
                let mask = 0x3FFF;
                let addr = (addr & mask) as usize;
                let bank = (self.rom_bank_number as usize) << 14usize;

                CartridgeReadTarget::Rom(bank | addr)
            }
            0xA000..=0xBFFF => {
                // 0xA000-0xA1FF -> 512 x 4bits built-in RAM
                // 0xA200-0xBFFF -> Repeat of 0xA000-0xA1FF
                if self.ram_enable {
                    let mask = 0x1FF;
                    CartridgeReadTarget::RamHalf((addr & mask) as usize)
                } else {
                    CartridgeReadTarget::Error
                }
            }
            _ => {
                log::warn!("Read on cartridge at {addr}, which isn't supposed to be mapped to the cartridge");
                CartridgeReadTarget::Error
            }
        }
    }

    fn map_write(&mut self, addr: u16, data: u8) -> Option<usize> {
        match addr {
            0x0000..=0x3FFF => {
                // RAM enabling and ROM bank selection
                let rom_select = addr & 0x100 == 0x100;

                if rom_select {
                    // Selecting ROM bank number
                    // Used to bank switch range 0x4000 - 0x7FFF
                    let bank_number = data & (self.bank_mask as u8);

                    if bank_number == 0 {
                        self.rom_bank_number = 1;
                    } else {
                        self.rom_bank_number = bank_number;
                    }
                    None
                } else {
                    // Enabling or disabling RAM
                    self.ram_enable = data == 0x0A;
                    None
                }
            }
            0xA000..=0xBFFF => {
                // Built-in RAM range
                // Only 0xA000-0xA1FF is actually used
                if self.ram_enable {
                    let mask = 0x1FF;
                    Some((addr & mask) as usize)
                } else {
                    // RAM is disabled, nothing to do
                    None
                }
            }
            _ => {
                log::warn!("Write on cartridge at {addr}, which isn't supposed to be mapped to the cartridge");
                None
            }
        }
    }
}
