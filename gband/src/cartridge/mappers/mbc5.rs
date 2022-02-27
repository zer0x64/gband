use super::Mapper;
use crate::cartridge::CartridgeReadTarget;

pub struct Mbc5 {
    ram_enable: bool,
    rom_bank_number: u8,
    rom_bank_number_9th: u8,
    ram_bank_number: u8,
}

impl Mbc5 {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Default for Mbc5 {
    fn default() -> Self {
        Self {
            ram_enable: false,
            rom_bank_number: 0x00,
            rom_bank_number_9th: 0x00,
            ram_bank_number: 0x00,
        }
    }
}

impl Mapper for Mbc5 {
    fn map_read(&self, addr: u16) -> CartridgeReadTarget {
        match addr {
            0x0000..=0x3FFF => {
                // First bank
                // Fixed to bank 0
                let mask = 0x3FFF;
                CartridgeReadTarget::Rom((addr & mask) as usize)
            }
            0x4000..=0x7FFF => {
                // Switchable ROM banks
                let mask = 0x3fff;
                let addr = (addr & mask) as usize;

                let mut bank = (self.rom_bank_number_9th as usize) << 22usize;
                bank |= (self.rom_bank_number as usize) << 14usize;
                CartridgeReadTarget::Rom(bank | addr)
            }
            0xA000..=0xBFFF => {
                // RAM range.
                // Can only be used when enabled
                if self.ram_enable {
                    let mask = 0x1fff;
                    let addr = (addr & mask) as usize;

                    let bank = (self.ram_bank_number as usize) << 13usize;
                    CartridgeReadTarget::Ram(bank | addr)
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
            0x0000..=0x1FFF => {
                // Enables or diables the RAM
                self.ram_enable = data & 0xF == 0x0A;
                None
            }
            0x2000..=0x2FFF => {
                // Set first 8 bits of ROM Bank Number
                // Used to bank switch range 0x4000 - 0x7FFF
                self.rom_bank_number = data;
                None
            }
            0x3000..=0x3FFF => {
                // Set 9th bit of ROM bank number
                self.rom_bank_number_9th = data & 0x1;
                None
            }
            0x4000..=0x5FFF => {
                // Set RAM bank number
                self.ram_bank_number = data & 0xF;
                None
            }
            0xA000..=0xBFFF => {
                // RAM banks
                if self.ram_enable {
                    // Switchable RAM banks
                    let mask = 0x1fff;
                    let addr = (addr & mask) as usize;

                    let bank = (self.ram_bank_number as usize) << 13usize;
                    Some(bank | addr)

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
