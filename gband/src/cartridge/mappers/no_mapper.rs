use super::Mapper;
use crate::cartridge::CartridgeReadTarget;

pub struct NoMapper;

impl Mapper for NoMapper {
    fn map_read(&self, addr: u16) -> CartridgeReadTarget {
        match addr {
            0x00..=0x7FFF => {
                // The max size here is 32KiB
                let mask = 0x7fff;
                CartridgeReadTarget::Rom((addr & mask) as usize)
            }
            0xA000..=0xBFFF => {
                let mask = 0x1fff;
                CartridgeReadTarget::Ram((addr & mask) as usize)
            }
            _ => {
                log::warn!("Read on cartridge at {addr}, which isn't supposed to be mapped to the cartridge");

                let mask = 0x7fff;
                CartridgeReadTarget::Rom((addr & mask) as usize)
            }
        }
    }

    fn map_write(&self, addr: u16, _data: u8) -> Option<usize> {
        match addr {
            0x00..=0x7FFF => {
                // Nothing to do
                None
            }
            0xA000..=0xBFFF => {
                let mask = 0x1fff;
                Some((addr & mask) as usize)
            }
            _ => {
                log::warn!("Write on cartridge at {addr}, which isn't supposed to be mapped to the cartridge");
                None
            }
        }
    }
}
