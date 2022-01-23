use super::Mapper;
use crate::cartridge::CartridgeReadTarget;

pub struct Mbc1 {
    n_rom_banks: usize,
    n_ram_banks: usize,
    bank_mask: usize,
    ram_enable: bool,
    rom_bank_number: u8,
    ram_bank_number_or_upper_rom_bank: u8,
    banking_mode_select: bool,
}

impl Mbc1 {
    pub fn new(n_rom_banks: usize, n_ram_banks: usize) -> Self {
        let bank_mask = n_rom_banks - 1;
        Self {
            n_rom_banks,
            n_ram_banks,
            bank_mask,
            ..Default::default()
        }
    }
}

impl Default for Mbc1 {
    fn default() -> Self {
        Self {
            n_rom_banks: 0,
            n_ram_banks: 0,
            bank_mask: 0,
            ram_enable: false,
            rom_bank_number: 0x01,
            ram_bank_number_or_upper_rom_bank: 0x00,
            banking_mode_select: false,
        }
    }
}

impl Mapper for Mbc1 {
    fn map_read(&self, addr: u16) -> CartridgeReadTarget {
        match addr {
            0x0000..=0x3FFF => {
                // First bank.
                // Fixed to bank 0, except in advanced banking on large cartridge
                //  where 2 additional bits can be used to select higher banks
                let mask = 0x3fff;
                let addr = (addr & mask) as usize;

                if self.n_rom_banks > 64 && self.banking_mode_select {
                    // Banking using the 2 higher bits
                    let bank = (self.ram_bank_number_or_upper_rom_bank as usize) << 19usize;
                    CartridgeReadTarget::Rom(bank | addr)
                } else {
                    // Not banking
                    CartridgeReadTarget::Rom(addr)
                }
            }
            0x4000..=0x7FFF => {
                // Switchable bank
                let mask = 0x3fff;
                let addr = (addr & mask) as usize;

                // ROM banking
                let mut bank = (self.rom_bank_number as usize) << 14usize;
                if self.n_rom_banks > 64 {
                    // Large ROM, using the additionnal bits
                    bank |= (self.ram_bank_number_or_upper_rom_bank as usize) << 19usize;
                };

                // Ram is disabled, so don't write to it
                CartridgeReadTarget::Rom(bank | addr)
            }
            0xA000..=0xBFFF => {
                // RAM range.
                // Can only be used when enabled
                if self.ram_enable {
                    let mask = 0x1fff;
                    let addr = (addr & mask) as usize;

                    if self.n_ram_banks > 1 && self.banking_mode_select {
                        // RAM bank switching when having enough RAM and advanced banking is enabled.
                        let bank = (self.ram_bank_number_or_upper_rom_bank as usize) << 13usize;
                        CartridgeReadTarget::Ram(bank | addr)
                    } else {
                        CartridgeReadTarget::Ram(addr)
                    }
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
                self.ram_enable = data & 0xf == 0x0A;
                None
            }
            0x2000..=0x3FFF => {
                // Set ROM Bank Number
                // Used to bank switch range 0x4000 - 0x7FFF
                let bank_number = data & (self.bank_mask as u8) & 0x1F;

                if bank_number == 0 {
                    // This register cannot be 0 and default to 1 if we try to set it to 0
                    self.rom_bank_number = 1;
                } else {
                    self.rom_bank_number = bank_number;
                }
                None
            }
            0x4000..=0x5FFF => {
                // Two additionnal bits used for bank switching on cartridge with large ROM or RAM
                self.ram_bank_number_or_upper_rom_bank = data & 0b11;
                None
            }
            0x6000..=0x7FFF => {
                // Select simple or Advanced banking mode.
                // Advanced mode is used to enable RAM bank switching or switching
                //  range 0x000-0x3FFF on cartridges with large ROM.
                self.banking_mode_select = data & 1 == 1;
                None
            }
            0xA000..=0xBFFF => {
                // RAM range.
                // Can only be used when enabled
                if self.ram_enable {
                    let mask = 0x1fff;
                    let addr = (addr & mask) as usize;

                    if self.n_ram_banks > 1 && self.banking_mode_select {
                        // RAM bank switching
                        let bank = (self.ram_bank_number_or_upper_rom_bank as usize) << 13usize;
                        Some(bank | addr)
                    } else {
                        Some(addr)
                    }
                } else {
                    // Ram is disabled, so don't write to it
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
