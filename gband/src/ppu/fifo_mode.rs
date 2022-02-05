use num_enum::IntoPrimitive;

#[derive(Clone, Copy, IntoPrimitive)]
#[repr(u8)]
pub enum FifoMode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

impl Default for FifoMode {
    fn default() -> Self {
        Self::OamScan
    }
}
