use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct JoypadState: u8 {
        const START = 0x80;
        const SELECT = 0x40;
        const B = 0x20;
        const A = 0x10;
        const DOWN = 0x08;
        const UP = 0x04;
        const LEFT = 0x02;
        const RIGHT = 0x01;
    }
}
