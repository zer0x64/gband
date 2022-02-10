use bitflags::bitflags;

bitflags! {
    pub struct CgbDoubleSpeed: u8 {
        const PENDING = 0x01;
        const UNUSED = 0x7E;
        const ENABLED = 0x80;
    }
}

impl Default for CgbDoubleSpeed {
    fn default() -> Self {
        CgbDoubleSpeed::UNUSED
    }
}
