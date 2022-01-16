use crate::rgb_palette::RGB_PALETTE;
use crate::Frame;
use crate::{FRAME_HEIGHT, FRAME_WIDTH};

pub fn frame_to_rgba(frame: &Frame, output: &mut [u8; FRAME_WIDTH * FRAME_HEIGHT * 4]) {
    let empasized_palette = &mut RGB_PALETTE.clone();

    for i in 0..frame.len() {
        let f = empasized_palette[(frame[i] & 0x3f) as usize];
        output[i * 4] = f[0]; // R
        output[i * 4 + 1] = f[1]; // G
        output[i * 4 + 2] = f[2]; // B

        // Alpha is always 0xff because it's opaque
        output[i * 4 + 3] = 0xff; // A
    }
}
