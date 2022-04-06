#[derive(Default, Clone)]
pub struct PixelFifo {
    pub fifo: [u16; 8],
    pub n_pixels: u8,
}

impl PixelFifo {
    pub fn is_empty(&self) -> bool {
        self.n_pixels == 0
    }

    pub fn pop(&mut self) -> Option<u16> {
        if self.n_pixels == 0 {
            None
        } else {
            let res = self.fifo[7];
            self.fifo.rotate_right(1);
            self.n_pixels -= 1;

            Some(res)
        }
    }

    pub fn load(&mut self, value: [u16; 8]) {
        self.fifo = value;
        self.n_pixels = 8;
    }
}
