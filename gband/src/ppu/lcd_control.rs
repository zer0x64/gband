use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct LcdControl: u8 {
        const BACKGROUND_WINDOW_ENABLE_PRIORITY = 0x01;
        const OBJ_ENABLE = 0x02;
        const OBJ_SIZE = 0x04;
        const BACKGROUND_TILE_MAP_AREA = 0x08;
        const BACKGROUND_WINDOW_TILE_DATA_AREA = 0x10;
        const WINDOW_ENABLE = 0x20;
        const WINDOW_TILE_MAP_AREA = 0x40;
        const LCD_PPU_ENABLE = 0x80;
    }
}
