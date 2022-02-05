use super::fifo_mode::FifoMode;

pub struct CgbPalette {
    pub data: [u8; 0x40],
    pub autoincrement: bool,
    pub index: usize,
}

impl Default for CgbPalette {
    fn default() -> Self {
        CgbPalette {
            data: [0u8; 0x40],
            autoincrement: false,
            index: 0,
        }
    }
}

impl CgbPalette {
    pub fn write_spec(&mut self, data: u8) {
        self.autoincrement = data & 0x80 == 0x80;
        self.index = (data & 0x3F) as usize;
    }

    pub fn write_data(&mut self, data: u8, mode: FifoMode) {
        match mode {
            FifoMode::Drawing => {
                // Write blocked during rendering
            }
            _ => {
                self.data[self.index] = data;
            }
        };

        if self.autoincrement {
            // Note: Autoincrement happens even if write is blocked, this is not a bug
            self.index += 1;
            self.index &= 0x3F;
        }
    }

    pub fn read_spec(&self) -> u8 {
        (self.index as u8) | if self.autoincrement { 0x80 } else { 0x00 }
    }

    pub fn read_data(&self, mode: FifoMode) -> u8 {
        match mode {
            FifoMode::Drawing => {
                // Read blocked during rendering
                0xFF
            }
            _ => self.data[self.index],
        }

        // Note: There is no autoincrement on read
    }
}
