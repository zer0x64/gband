use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

pub const FRAME_WIDTH: usize = 160;
pub const FRAME_HEIGHT: usize = 144;

pub type Frame = Box<[u8; FRAME_WIDTH * FRAME_HEIGHT]>;

pub struct Ppu {
    frame: Option<Frame>,
}

impl Default for Ppu {
    fn default() -> Self {
        // TODO

        let mut ppu = Self { frame: None };

        ppu.allocate_new_frame();

        ppu
    }
}

impl Ppu {
    pub fn clock(&mut self) {
        // TODO
    }

    pub fn ready_frame(&mut self) -> Option<Frame> {
        // TODO: Only returns when the frame is actually done

        // Returns the current frame buffer
        let frame = self
            .frame
            .take()
            .expect("the frame buffer should never be unallocated");

        // Allocate a new frame buffer
        self.allocate_new_frame();

        Some(frame)
    }

    fn allocate_new_frame(&mut self) {
        //   Hackish way to create fixed size boxed array.
        // I don't know of any way to do it without
        // having the data allocated on the stack at some point or using unsafe
        let v: Vec<u8> = vec![0u8; FRAME_WIDTH * FRAME_HEIGHT];
        let b = v.into_boxed_slice();

        // Safety: This only uses constants and the fonction doesn't have arguments
        self.frame = unsafe {
            Some(Box::from_raw(
                Box::into_raw(b) as *mut [u8; FRAME_WIDTH * FRAME_HEIGHT]
            ))
        }
    }
}
