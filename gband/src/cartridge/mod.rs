mod header;
mod mappers;

use alloc::boxed::Box;
use header::{CartridgeType, Header, RamBanks};
use mappers::*;

pub use header::RomParserError;

pub enum CartridgeReadTarget {
    Error,
    Rom(usize),
    Ram(usize),
}

pub struct Cartridge {
    pub header: Header,
    pub rom: alloc::vec::Vec<u8>,
    pub ram: Option<alloc::vec::Vec<u8>>,
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn load(rom: &[u8], save_data: Option<&[u8]>) -> Result<Self, RomParserError> {
        if rom.len() < 0x150 {
            return Err(RomParserError::TooShort);
        };

        let header = Header::try_from(&rom[0x100..0x150])?;
        log::info!("{header:x?}");

        let rom = rom.to_vec();

        let mut ram = match header.ram_banks {
            RamBanks::Banks(n) => {
                // 1 bank is 8 KiB
                Some(alloc::vec![0u8; n*8*1024])
            }
            RamBanks::Mbc2 => Some(alloc::vec![0u8; 512]),
            _ => None,
        };

        // Load in save data
        match (&mut ram, save_data) {
            (Some(r), Some(s)) => {
                if r.len() == s.len() {
                    r.copy_from_slice(s);
                } else {
                    log::warn!(
                        "Couldn't load save as the size doesn't match. Ram: {:x}, Save: {:x}",
                        r.len(),
                        s.len()
                    )
                }
            }
            _ => {}
        };

        let ram_banks = match header.ram_banks {
            RamBanks::Banks(x) => x,
            _ => 0,
        };

        let mapper: Box<dyn Mapper> = match header.cartridge_type {
            CartridgeType::RomOnly | CartridgeType::RomRam | CartridgeType::RomRamBattery => {
                Box::new(NoMapper)
            }
            CartridgeType::Mbc1 | CartridgeType::Mbc1Ram | CartridgeType::Mbc1RamBattery => {
                Box::new(Mbc1::new(header.rom_banks, ram_banks))
            }
            _ => return Err(RomParserError::MapperNotImplemented),
        };

        Ok(Self {
            header,
            rom,
            ram,
            mapper,
        })
    }

    pub fn read(&self, addr: u16) -> u8 {
        match self.mapper.map_read(addr) {
            CartridgeReadTarget::Error => 0,
            CartridgeReadTarget::Rom(addr) => self.rom[addr % self.rom.len()],
            CartridgeReadTarget::Ram(addr) => match &self.ram {
                Some(ram) => ram[addr % ram.len()],
                None => {
                    log::warn!(
                        "Tried to read Cartridge RAM at {addr}, but the cartridge has no ram!"
                    );
                    0
                }
            },
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        if let Some(addr) = self.mapper.map_write(addr, data) {
            match &mut self.ram {
                Some(ram) => {
                    let size = ram.len();
                    ram[addr % size] = data;
                }
                None => {
                    log::warn!(
                        "Tried to write Cartridge RAM at {addr}, but the cartridge has no ram!"
                    );
                }
            }
        };
    }

    pub fn get_save_data(&self) -> Option<&[u8]> {
        match &self.ram {
            Some(r) => Some(r),
            None => None,
        }
    }
}
