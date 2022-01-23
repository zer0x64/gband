use super::CartridgeReadTarget;

mod mbc1;
mod no_mapper;

pub use mbc1::Mbc1;
pub use no_mapper::NoMapper;

pub trait Mapper: Send + Sync {
    fn map_read(&self, addr: u16) -> CartridgeReadTarget;
    fn map_write(&mut self, addr: u16, data: u8) -> Option<usize>;
}
