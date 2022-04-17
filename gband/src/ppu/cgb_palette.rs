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

    pub fn write_data(&mut self, data: u8, _mode: FifoMode) {
        // TODO: Reimplement write lock
        // match mode {
        //     FifoMode::Drawing(_) => {
        //         // Write blocked during rendering
        //     }
        //     _ => {
        //         self.data[self.index] = data;
        //     }
        // };

        self.data[self.index] = data;
        if self.autoincrement {
            // Note: Autoincrement happens even if write is blocked, this is not a bug
            self.index += 1;
            self.index &= 0x3F;
        }
    }

    pub fn read_spec(&self) -> u8 {
        (self.index as u8) | if self.autoincrement { 0x80 } else { 0x00 }
    }

    pub fn read_data(&self, _mode: FifoMode) -> u8 {
        // TODO: Reimplement write lock
        // match mode {
        //     FifoMode::Drawing(_) => {
        //         // Read blocked during rendering
        //         0xFF
        //     }
        //     _ => self.data[self.index],
        // }

        self.data[self.index]
        // Note: There is no autoincrement on read
    }

    pub fn get_rgb(&self, palette_index: usize, color_index: usize) -> [u8; 3] {
        let mut pixel = [0u8; 3];

        let lo = self.data[(palette_index << 3) | (color_index << 1)] as u16;
        let hi = self.data[(palette_index << 3) | (color_index << 1) | 1] as u16;

        let mut color555 = (hi << 8) | lo;

        let r555 = (color555 & 0x1f) as u8;
        pixel[0] = (r555 << 3) | (r555 >> 2);
        color555 >>= 5;

        let g555 = (color555 & 0x1f) as u8;
        pixel[1] = (g555 << 3) | (g555 >> 2);
        color555 >>= 5;

        let b555 = (color555 & 0x1f) as u8;
        pixel[2] = (b555 << 3) | (b555 >> 2);

        pixel
    }
}
