use crate::enums::ExecutionMode;

#[derive(Debug, Clone, Copy)]
pub enum RomParserError {
    // TODO
    TooShort,
    InvalidMagicBytes,
    MapperNotImplemented,
}

impl core::fmt::Display for RomParserError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", &self)
    }
}

pub struct Cartridge {
    pub execution_mode: ExecutionMode,
    // TODO
}

impl Cartridge {
    pub fn load(_rom: &[u8], _save_data: Option<&[u8]>) -> Result<Self, RomParserError> {
        // TODO
        Ok(Self { execution_mode: ExecutionMode::GB })
    }
}
