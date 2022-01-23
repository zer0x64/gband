use super::CartridgeReadTarget;

mod no_mapper;

pub use no_mapper::NoMapper;

pub trait Mapper: Send + Sync {
    fn map_read(&self, addr: u16) -> CartridgeReadTarget;
    fn map_write(&self, addr: u16, data: u8) -> Option<usize>;
}
