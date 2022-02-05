use super::FifoMode;
use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct LcdStatus: u8 {
        const MODE_LOW = 0x01;
        const MODE_HI = 0x02;
        const LYC_EQ_LC = 0x04;
        const HBANLK_INTERUPT_SOURCE = 0x08;
        const VBANLK_INTERUPT_SOURCE = 0x10;
        const OAM_INTERUPT_SOURCE = 0x20;
        const LYC_EQ_LC_INTERUPT_SOURCE = 0x40;
        const UNUSED = 0x80;
    }
}

impl LcdStatus {
    pub fn set_mode(&mut self, mode: FifoMode) {
        let mode: u8 = mode.into();
        let val = self.bits() & 0xfc | mode;
        self.bits = val;
    }
}
