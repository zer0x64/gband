use num_enum::TryFromPrimitive;

#[derive(Debug, Clone, Copy)]
pub enum RomParserError {
    // TODO
    TooShort,
    UnknownMapper,
    MapperNotImplemented,
    InvalidChecksum,
}

impl core::fmt::Display for RomParserError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", &self)
    }
}

#[derive(Clone, Copy)]
pub enum CgbFlag {
    NoCgb,
    CgbSupported,
    CgbOnly,
}

#[derive(Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum CartridgeType {
    RomOnly = 0x00,
    Mbc1 = 0x01,
    Mbc1Ram = 0x02,
    Mbc1RamBattery = 0x03,
    Mbc2 = 0x05,
    Mbc2Battery = 0x06,
    RomRam = 0x08,
    RomRamBattery = 0x09,
    Mmm01 = 0x0B,
    Mmm01Ram = 0x0C,
    Mmm01RamBattery = 0x0D,
    Mbc3TimerBattery = 0x0F,
    Mbc3TimerRamBattery = 0x10,
    Mbc3 = 0x11,
    Mbc3Ram = 0x12,
    Mbc3RamBattery = 0x13,
    Mbc5 = 0x19,
    Mbc5Ram = 0x1A,
    Mbc5RamBattery = 0x1B,
    Mbc5Rumble = 0x1C,
    Mbc5RumbleRam = 0x1D,
    Mbc5RumbleRamBattery = 0x1E,
    Mbc6 = 0x20,
    Mbc7SensorRumbleRamBattery = 0x22,
    PocketCamera = 0xFC,
    BandaiTama5 = 0xFD,
    Huc3 = 0xFE,
    Huc1RamBattery = 0xFF,
}

#[derive(Clone, Copy)]
pub enum RamBanks {
    None,
    Mbc2,
    TwoKb,
    Banks(usize),
}

#[derive(Clone)]
pub struct Header {
    pub logo: [u8; 0x30],
    pub title: [u8; 11],
    pub manufacturer_code: [u8; 4],
    pub cgb_flag: CgbFlag,
    pub licensee_code: [u8; 2],
    pub sgb_flag: bool,
    pub cartridge_type: CartridgeType,
    pub rom_banks: usize,
    pub ram_banks: RamBanks,
    pub is_japanese: bool,
    pub old_licensee_code: u8,
    pub mask_rom_version_number: u8,
    pub header_checksum: u8,
    pub global_checksum: [u8; 2],
}

impl TryFrom<&[u8]> for Header {
    type Error = RomParserError;

    fn try_from(data: &[u8]) -> Result<Self, RomParserError> {
        if data.len() < 0x50 {
            return Err(RomParserError::TooShort);
        };

        let mut logo = [0u8; 0x30];
        logo.copy_from_slice(&data[0x04..0x34]);

        let mut title = [0u8; 11];
        title.copy_from_slice(&data[0x34..0x3f]);

        let mut manufacturer_code = [0u8; 4];
        manufacturer_code.copy_from_slice(&data[0x3f..0x43]);

        let cgb_flag = match data[0x43] >> 6 {
            0b10 => CgbFlag::CgbSupported,
            0b11 => CgbFlag::CgbOnly,
            _ => CgbFlag::NoCgb,
        };

        let mut licensee_code = [0u8; 2];
        licensee_code.copy_from_slice(&data[0x44..0x46]);

        let sgb_flag = data[0x46] == 0x03;

        let cartridge_type = if let Ok(t) = CartridgeType::try_from(data[0x47]) {
            t
        } else {
            return Err(RomParserError::UnknownMapper);
        };

        let is_mbc1 = match cartridge_type {
            CartridgeType::Mbc1 | CartridgeType::Mbc1Ram | CartridgeType::Mbc1RamBattery => true,
            _ => false,
        };

        let rom_banks: usize = match data[0x48] {
            0x05 => {
                if is_mbc1 {
                    63
                } else {
                    64
                }
            }
            0x06 => {
                if is_mbc1 {
                    125
                } else {
                    128
                }
            }
            0x52 => 72,
            0x53 => 80,
            0x54 => 96,
            b => 0b10usize.overflowing_shl(b.into()).0,
        };

        let ram_banks = match data[0x49] {
            0 => match cartridge_type {
                CartridgeType::Mbc2 | CartridgeType::Mbc2Battery => RamBanks::Mbc2,
                _ => RamBanks::None,
            },
            1 => RamBanks::TwoKb,
            2 => RamBanks::Banks(1),
            3 => RamBanks::Banks(4),
            4 => RamBanks::Banks(16),
            5 => RamBanks::Banks(8),

            // Soft fail without RAM
            _ => RamBanks::None,
        };

        let is_japanese = data[0x4a] == 0;

        let old_licensee_code = data[0x4b];
        let mask_rom_version_number = data[0x4c];
        let header_checksum = data[0x4d];

        let mut checksum: usize = 0;
        for b in data[0x34..0x4d].iter() {
            checksum = checksum.wrapping_sub(*b as usize).wrapping_sub(1);
        }

        if (checksum & 0xff) as u8 != header_checksum {
            return Err(RomParserError::InvalidChecksum);
        };

        // Note: The global checksum isn't verify on the actual gameboy.
        let mut global_checksum = [0u8; 2];
        global_checksum.copy_from_slice(&data[0x4e..0x50]);

        Ok(Header {
            logo,
            title,
            manufacturer_code,
            cgb_flag,
            licensee_code,
            sgb_flag,
            cartridge_type,
            rom_banks,
            ram_banks,
            is_japanese,
            old_licensee_code,
            mask_rom_version_number,
            header_checksum,
            global_checksum,
        })
    }
}
