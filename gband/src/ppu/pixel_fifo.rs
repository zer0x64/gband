#[derive(Default, Clone)]
pub struct PixelFifo {
    pub fifo: [u16; 8],
    pub n_pixels: u8,
}

impl PixelFifo {
    pub fn is_empty(&self) -> bool {
        self.n_pixels == 0
    }

    pub fn empty(&mut self) {
        self.n_pixels = 0;
        self.fifo = Default::default();
    }

    pub fn drain(&mut self, n_pixels: u8) {
        for _ in 0..n_pixels {
            // Drain the extra pixels
            let _ = self.pop();
        }
    }

    pub fn pop(&mut self) -> u16 {
        if self.n_pixels == 0 {
            self.fifo[7]
        } else {
            let res = self.fifo[7];
            self.fifo.rotate_right(1);
            self.fifo[0] = 0;
            self.n_pixels -= 1;

            res
        }
    }

    pub fn load(&mut self, value: [u16; 8]) {
        for i in 0..8 {
            // Only overwrite transparent pixels
            if self.fifo[i] & 3 == 0 {
                self.fifo[i] = value[i];
            }
        }

        self.n_pixels = 8;
    }
}
