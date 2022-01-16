use crate::enums::ExecutionMode;

mod header;

use header::Header;

pub use header::RomParserError;

pub struct Cartridge {
    pub header: Header,
    pub execution_mode: ExecutionMode,
    pub rom: alloc::vec::Vec<u8>,
    // TODO
}

impl Cartridge {
    pub fn load(rom: &[u8], _save_data: Option<&[u8]>) -> Result<Self, RomParserError> {
        // TODO
        if rom.len() < 0x150 {
            return Err(RomParserError::TooShort);
        };

        let header = Header::try_from(&rom[0x100..0x150])?;

        let rom = rom.to_vec();

        Ok(Self {
            header,
            execution_mode: ExecutionMode::GB,
            rom,
        })
    }

    pub fn read(&self, addr: u16) -> u8 {
        // We do it dirty for now, will need to redo after we write MBC1
        self.rom[addr as usize]
    }

    pub fn write(&mut self, _addr: u16, _data: u8) {
        // Nothing to do for now, will need to to revisit after doing complex mappers
    }
}
