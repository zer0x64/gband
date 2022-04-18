#[derive(Default, Clone)]
pub struct OamDma {
    pub cycle: Option<u8>,
    pub source: u8,
}

impl OamDma {
    pub fn new(source: u8) -> Self {
        Self {
            source,
            cycle: Some(0),
        }
    }
}

#[derive(Clone)]
pub struct HDma {
    pub source: u16,
    pub destination: u16,
    pub control: u8,
    pub hblank_mode: bool,

    pub cycle: u8,
    pub hblank_latch: bool,
}

impl Default for HDma {
    fn default() -> Self {
        Self {
            source: 0xFFFF,
            destination: 0xFFFF,
            control: 0xFF,
            cycle: 0,
            hblank_mode: false,
            hblank_latch: false,
        }
    }
}

impl HDma {
    pub fn start(&mut self, data: u8) {
        // HDMA is not active, start it
        self.reset();

        let mode = data & 0x80 == 0x80;
        let length = data & 0x7F;

        self.hblank_mode = mode;
        self.control = length;
    }

    pub fn is_currently_in_hdma(&self) -> bool {
        self.is_active() && (!self.hblank_mode || self.hblank_latch)
    }

    pub fn is_active(&self) -> bool {
        self.control & 0x80 == 0
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.hblank_latch = false;
    }
}
