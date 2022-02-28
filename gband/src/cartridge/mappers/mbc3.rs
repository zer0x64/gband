use super::Mapper;
use crate::cartridge::CartridgeReadTarget;

pub struct Mbc3 {
    ram_rtc_enable: bool,
    ram_or_rtc_bank_number: u8,
    rom_bank_number: u8,
}

impl Mbc3 {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Default for Mbc3 {
    fn default() -> Self {
        Self {
            ram_or_rtc_bank_number: 0x00,
            ram_rtc_enable: false,
            rom_bank_number: 0x01,
        }
    }
}

impl Mapper for Mbc3 {
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
                let mask = 0x3FFF;
                let addr = (addr & mask) as usize;
                let bank = (self.rom_bank_number as usize) << 14usize;

                CartridgeReadTarget::Rom(bank | addr)
            }
            0xA000..=0xBFFF => {
                // RAM or RTC range
                // Only RAM supported at the moment
                if self.ram_rtc_enable {
                    let mask = 0x1FFF;
                    let addr = (addr & mask) as usize;

                    let bank = (self.ram_or_rtc_bank_number as usize) << 13usize;
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
                // RAM and RTC enable
                self.ram_rtc_enable = data == 0x0A;
                None
            }
            0x2000..=0x3FFF => {
                // ROM bank number
                let bank_number = data & 0x7F;
                if bank_number == 0 {
                    self.rom_bank_number = 0x01;
                } else {
                    self.rom_bank_number = bank_number;
                }
                None
            }
            0x4000..=0x5FFF => {
                // RAM bank number OR RTC register select
                // data 0x00-0x03 sets RAM bank
                // data 0x08-0x0C sets RTC register

                // Temporary mask while RTC isn't implemented
                self.ram_or_rtc_bank_number = data & 0x03;
                None
            }
            0x6000..=0x7FFF => {
                // Latch lock data
                // Writing 0x00 then 0x01 will write the current time to the RTC register
                None
            }
            0xA000..=0xBFFF => {
                // RAM and RTC range

                if self.ram_rtc_enable {
                    let mask = 0x1FFF;
                    let addr = (addr & mask) as usize;

                    let bank = (self.ram_or_rtc_bank_number as usize) << 13usize;
                    Some(bank | addr)

                } else {
                    // RAM and RTC are disabled, nothing to do
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
