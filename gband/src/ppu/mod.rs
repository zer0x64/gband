use core::num::Wrapping;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

mod cgb_palette;
mod fifo_mode;
mod lcd_control;
mod lcd_status;
mod palette_table;
mod pixel_fifo;

use cgb_palette::CgbPalette;
pub(crate) use fifo_mode::FifoMode;
use lcd_control::LcdControl;
use lcd_status::LcdStatus;

use crate::bus::PpuBus;
use crate::InterruptReg;

use self::{
    fifo_mode::{DrawingState, OamScanState, PixelFetcherState},
    pixel_fifo::PixelFifo,
};

pub const FRAME_WIDTH: usize = 160;
pub const FRAME_HEIGHT: usize = 144;

pub type Frame = Box<[u8; FRAME_WIDTH * FRAME_HEIGHT * 4]>;

pub struct Ppu {
    cgb_mode: bool,

    x: u8,
    y: u8,
    window_y_counter: u8,
    window_y_flag: bool,
    y_compare: u8,

    window_x: u8,
    window_y: u8,

    scroll_x: u8,
    scroll_y: u8,

    vram: [u8; 0x4000],
    vram_bank_register: bool,
    oam: [u8; 0xa0],
    secondary_oam: [u8; 40],

    cgb_bg_palette: CgbPalette,
    cgb_obj_palette: CgbPalette,

    dmg_bg_palette: u8,
    dmg_obj_palette: [u8; 2],

    dmg_colorized_bg_palette: [[u8; 3]; 4],
    dmg_colorized_obj_palette: [[[u8; 3]; 4]; 2],

    lcd_control_reg: LcdControl,
    lcd_status_reg: LcdStatus,

    background_pixel_pipeline: PixelFifo,
    sprite_pixel_pipeline: PixelFifo,

    cycle: u16,
    paused_cycles: u32,
    fifo_mode: FifoMode,
    frame: Frame,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            cgb_mode: false,

            x: 0,
            y: 0,
            window_y_counter: 0,
            window_y_flag: false,
            y_compare: 0,

            window_x: 0,
            window_y: 0,

            scroll_x: 0,
            scroll_y: 0,

            vram: [0u8; 0x4000],
            vram_bank_register: false,
            oam: [0u8; 0xa0],
            secondary_oam: [0u8; 40],

            lcd_control_reg: Default::default(),
            lcd_status_reg: Default::default(),

            // Boot ROM initializes the Background palettes to white
            cgb_bg_palette: CgbPalette {
                data: [0xFFu8; 0x40],
                ..Default::default()
            },
            cgb_obj_palette: CgbPalette {
                data: [0xFFu8; 0x40],
                ..Default::default()
            },

            dmg_bg_palette: 0,
            dmg_obj_palette: [0; 2],

            dmg_colorized_bg_palette: Default::default(),
            dmg_colorized_obj_palette: Default::default(),

            background_pixel_pipeline: Default::default(),
            sprite_pixel_pipeline: Default::default(),

            cycle: 0,
            paused_cycles: 0,
            fifo_mode: Default::default(),
            frame: allocate_new_frame(),
        }
    }
}

impl Ppu {
    pub fn new(cgb_mode: bool) -> Self {
        Self {
            cgb_mode,
            ..Default::default()
        }
    }

    pub fn set_dmg_colorized_palette(&mut self, title: &[u8; 16]) {
        let hash: Wrapping<u8> = title.iter().map(|x| Wrapping(*x)).sum();

        let palettes = palette_table::palette_fill_from_hash(hash.0, title[3]);

        self.dmg_colorized_bg_palette = palettes[0];
        self.dmg_colorized_obj_palette[0] = palettes[1];
        self.dmg_colorized_obj_palette[1] = palettes[2];
    }

    pub fn clock(&mut self, bus: &mut PpuBus) {
        if !self.lcd_control_reg.contains(LcdControl::LCD_PPU_ENABLE) {
            // Continue cycling to push frames
            self.paused_cycles += 1;
            if self.paused_cycles >= 70224 {
                self.paused_cycles = 0;
            }
            // PPU is disabled, only make sure to return frames
            return;
        }

        self.cycle += 1;

        if self.y < 144 {
            match self.cycle {
                80 => {
                    self.fifo_mode = FifoMode::Drawing(Default::default());
                }
                _ => {}
            }
        }

        if self.cycle >= 456 {
            self.cycle = 0;
            self.x = 0;
            self.y += 1;

            // Signal the CPU that HBLANK is over for HDMA
            bus.set_hdma_hblank(false);

            // TODO: Selection priority
            // During each scanline’s OAM scan, the PPU compares LY (using LCDC bit 2 to determine their size) to each object’s Y position to select up to 10 objects to be drawn on that line. The PPU scans OAM sequentially (from $FE00 to $FE9F), selecting the first (up to) 10 suitably-positioned objects.
            // Since the PPU only checks the Y coordinate to select objects, even off-screen objects count towards the 10-objects-per-scanline limit. Merely setting an object’s X coordinate to X = 0 or X ≥ 168 (160 + 8) will hide it, but it will still count towards the limit, possibly causing another object later in OAM not to be drawn. To keep off-screen objects from affecting on-screen ones, make sure to set their Y coordinate to Y = 0 or Y ≥ 160 (144 + 16). (Y ≤ 8 also works if object size is set to 8x8.)

            match self.y {
                144..=153 => {
                    // We are in VBLANK
                    self.fifo_mode = FifoMode::VBlank;

                    if self.y == 144 {
                        // Request VBLANK interrupt
                        bus.request_interrupt(InterruptReg::VBLANK);

                        if self
                            .lcd_status_reg
                            .contains(LcdStatus::VBANLK_INTERUPT_SOURCE)
                        {
                            bus.request_interrupt(InterruptReg::LCD_STAT);
                        }
                    }
                }
                154 => {
                    // End of the frame
                    self.y = 0;
                    self.window_y_counter = 0;
                    self.window_y_flag = false;
                    self.fifo_mode = FifoMode::OamScan(Default::default());

                    if self.lcd_status_reg.contains(LcdStatus::OAM_INTERUPT_SOURCE) {
                        bus.request_interrupt(InterruptReg::LCD_STAT);
                    }
                }
                _ => {
                    self.fifo_mode = FifoMode::OamScan(Default::default());

                    if self.lcd_status_reg.contains(LcdStatus::OAM_INTERUPT_SOURCE) {
                        bus.request_interrupt(InterruptReg::LCD_STAT);
                    }
                }
            };

            if self.y == self.y_compare {
                if self
                    .lcd_status_reg
                    .contains(LcdStatus::LYC_EQ_LC_INTERUPT_SOURCE)
                {
                    bus.request_interrupt(InterruptReg::LCD_STAT);
                }
            };
        };

        self.render(bus);
    }

    pub fn ready_frame(&mut self) -> Option<Frame> {
        let is_ready = if self.lcd_control_reg.contains(LcdControl::LCD_PPU_ENABLE) {
            self.y == 0 && self.cycle == 0
        } else {
            self.paused_cycles == 0
        };

        if is_ready {
            let new_frame = allocate_new_frame();

            // Replace current frame with the newly allocated one
            let frame = core::mem::replace(&mut self.frame, new_frame);

            Some(frame)
        } else {
            None
        }
    }

    pub fn write_vram(&mut self, addr: u16, data: u8) {
        match self.fifo_mode {
            FifoMode::Drawing(_) => {
                // Calls are blocked during this mode
                // Do nothing
                // TODO: There are timing issues right now so the write block breaks rendering right now.
                // Delete those lines when the timing issues are fixed
                let addr = addr & 0x1FFF | self.get_current_vram_bank();
                self.vram[addr as usize] = data;
            }
            _ => {
                let addr = addr & 0x1FFF | self.get_current_vram_bank();
                self.vram[addr as usize] = data;
            }
        }
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        match self.fifo_mode {
            FifoMode::Drawing(_) => {
                // Calls are blocked during this mode
                // Do nothing and return trash
                0xFF
            }
            _ => self.read_vram_unblocked(addr),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.lcd_control_reg.contains(LcdControl::LCD_PPU_ENABLE)
    }

    pub fn get_mode(&self) -> &FifoMode {
        &self.fifo_mode
    }

    pub fn disable(&mut self) {
        self.cycle = 0;
        self.window_y_flag = false;
        self.window_y_counter = 0;
        self.x = 0;
        self.y = 0;
        self.fifo_mode = Default::default();
        self.background_pixel_pipeline = Default::default();
        self.sprite_pixel_pipeline = Default::default();
    }

    fn read_vram_unblocked(&self, addr: u16) -> u8 {
        let addr = addr & 0x1FFF | self.get_current_vram_bank();
        self.vram[addr as usize]
    }

    fn read_vram_without_banking(&self, addr: u16) -> u8 {
        let addr = addr & 0x3FFF;
        self.vram[addr as usize]
    }

    pub fn write_oam(&mut self, addr: u16, data: u8, _force: bool) {
        // TODO: Redo write block
        // match self.fifo_mode {
        //     FifoMode::OamScan { .. } | FifoMode::Drawing(_) => {
        //         // Calls are blocked during this mode
        //         // Do nothing, except if this is called by the OAM DMA
        //         if !force {
        //             return;
        //         }
        //     }
        //     _ => {
        //         // Continue normally
        //     }
        // }

        let addr = addr & 0xFF;
        self.oam[addr as usize] = data;
    }

    pub fn read_oam(&self, addr: u16, _force: bool) -> u8 {
        // TODO: Redo read block
        // match self.fifo_mode {
        //     FifoMode::OamScan(_) | FifoMode::Drawing(_) => {
        //         // Calls are blocked during this mode
        //         // Do nothing and return trash, except if this is called by the OAM DMA
        //         if !force {
        //             return 0xFF;
        //         }
        //     }
        //     _ => {
        //         // Continue normally
        //     }
        // }

        let addr = addr & 0xFF;
        self.oam[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xFF40 => self.write_lcd_control(data),
            0xFF41 => self.write_lcd_status(data),
            0xFF42 => self.scroll_y = data,
            0xFF43 => self.scroll_x = data,
            0xFF44 => {
                // ly is Read-Only
            }
            0xFF45 => self.y_compare = data,
            0xFF47 => self.dmg_bg_palette = data,
            0xFF48 | 0xFF49 => self.dmg_obj_palette[(addr & 1) as usize] = data,
            0xFF4A => self.window_y = data,
            0xFF4B => self.window_x = data,
            0xFF4C => {
                // rKEY0 is blocked after boot
            }
            0xFF4F => self.vram_bank_register = data & 1 > 0,
            0xFF68 => self.cgb_bg_palette.write_spec(data),
            0xFF69 => self.cgb_bg_palette.write_data(data, self.fifo_mode),
            0xFF6A => self.cgb_obj_palette.write_spec(data),
            0xFF6B => self.cgb_obj_palette.write_data(data, self.fifo_mode),
            _ => {
                // Address not recognised, do nothing
            }
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.read_lcd_control(),
            0xFF41 => self.read_lcd_status(),
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.y,
            0xFF45 => self.y_compare,
            0xFF47 => self.dmg_bg_palette,
            0xFF48 | 0xFF49 => self.dmg_obj_palette[(addr & 1) as usize],
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            0xFF4C => {
                // rKEY0 is blocked after boot
                0xFF
            }
            0xFF4F => {
                let bank_bit = if self.vram_bank_register { 1 } else { 0 };
                0xFE | bank_bit
            }
            0xFF68 => self.cgb_bg_palette.read_spec(),
            0xFF69 => self.cgb_bg_palette.read_data(self.fifo_mode),
            0xFF6A => self.cgb_obj_palette.read_spec(),
            0xFF6B => self.cgb_obj_palette.read_data(self.fifo_mode),
            _ => {
                // Address not recognised, do nothing
                0
            }
        }
    }

    fn write_lcd_control(&mut self, data: u8) {
        self.lcd_control_reg =
            LcdControl::from_bits(data).expect("any data should be valid for LCDC bitflags")
    }

    fn read_lcd_control(&self) -> u8 {
        self.lcd_control_reg.bits()
    }

    fn write_lcd_status(&mut self, data: u8) {
        // Only those bits are writeable.
        let mask = 0b01111000;
        let status_reg = self.lcd_status_reg.bits() & !mask;
        let status_reg = status_reg | (data & mask);

        self.lcd_status_reg = LcdStatus::from_bits(status_reg)
            .expect("the reg can take 8 bits, so no value should fail");
    }

    fn read_lcd_status(&self) -> u8 {
        let mut status_reg = self.lcd_status_reg;

        // Those bits are constantly changed, so might as well update them only when needed
        status_reg.set(LcdStatus::LYC_EQ_LC, self.y == self.y_compare);
        status_reg.set_mode(self.fifo_mode);

        status_reg.bits()
    }

    fn render(&mut self, bus: &mut PpuBus) {
        // Work on a copy to fix borrow checker issue
        let mut fifo_mode = self.fifo_mode;

        match &mut fifo_mode {
            FifoMode::OamScan(OamScanState {
                oam_pointer,
                secondary_oam_pointer,
                is_visible,
            }) => {
                if self.cycle & 1 == 0 {
                    // On even cycle, fetch the y value and check if it's visible
                    let y = self.oam[*oam_pointer];

                    let sprite_size = if self.lcd_control_reg.contains(LcdControl::OBJ_SIZE) {
                        16
                    } else {
                        8
                    };

                    // The index is y + 16, so the sprite can be hidden off at 0. This is why we add 16 here
                    let y_remainder = self.y.wrapping_sub(y).wrapping_add(16);

                    *is_visible = (y_remainder < sprite_size) && (self.oam[*oam_pointer + 1] > 0);
                } else {
                    // On odd cycle, copy it to the secondary OAM
                    if *is_visible {
                        // Line is visible
                        if *secondary_oam_pointer < self.secondary_oam.len() {
                            self.secondary_oam[*secondary_oam_pointer..*secondary_oam_pointer + 4]
                                .copy_from_slice(&self.oam[*oam_pointer..*oam_pointer + 4]);
                            *secondary_oam_pointer += 4;
                        }
                    }

                    *oam_pointer += 4
                }
            }
            FifoMode::Drawing(state) => {
                // if self
                //     .lcd_control_reg
                //     .contains(LcdControl::BACKGROUND_WINDOW_ENABLE_PRIORITY)
                // {
                // NOTE: assuming non-GBC mode only for now

                // Check for window
                if self.y == self.window_y {
                    self.window_y_flag = true;
                }

                if !state.is_window && self.lcd_control_reg.contains(LcdControl::WINDOW_ENABLE) {
                    if self.window_y_flag && self.x.wrapping_add(7) >= self.window_x {
                        // We start rendering the window
                        // We flush the entire state and signal that we start to render the window
                        state.reset();

                        state.is_window = true;
                        state.fetcher_x = 0;

                        self.background_pixel_pipeline.empty();
                    }
                }

                // Check for sprites
                if !state.is_sprite && self.lcd_control_reg.contains(LcdControl::OBJ_ENABLE) {
                    // This condition is only for when on DMG!
                    for (index, sprite) in self.secondary_oam.chunks_exact(4).enumerate() {
                        let sprite = <&[u8; 4]>::try_from(sprite)
                            .expect("secondary OAM should always be chunks of 4");

                        // The sprite address is x + 8, so it can be hidden if set at 0
                        let x_remainder = self.x.wrapping_sub(sprite[1]).wrapping_add(8);
                        if x_remainder < 8 {
                            // Start a sprite fetch
                            state.reset();

                            state.is_sprite = true;
                            state.sprite_idx = (index << 2) as u8;

                            break;
                        }
                    }
                }

                match state.pixel_fetcher {
                    PixelFetcherState::GetTile => {
                        // Get the tile used in this part of the map
                        // Here we use a specific fetcher indexing
                        // https://gbdev.io/pandocs/pixel_fifo.html
                        if state.cycle == 0 {
                            (state.tile_idx, state.tile_attr) = if state.is_sprite {
                                // For sprites, we simply fetch it from the OAM entry
                                (
                                    self.secondary_oam[(state.sprite_idx + 2) as usize],
                                    self.secondary_oam[(state.sprite_idx + 3) as usize],
                                )
                            } else if state.is_window {
                                // For window, we use the internal window Y counter and the X fetch counter
                                let x_index = (state.fetcher_x) & 0x1F;
                                let y_index = self.window_y_counter >> 3;
                                let tile_map_idx =
                                    ((y_index as u16) << 5) | (x_index as u16 & 0x1F);

                                let idx = self.read_win_tile_index(tile_map_idx);
                                let attr = if self.cgb_mode {
                                    self.read_win_tile_attributes(tile_map_idx)
                                } else {
                                    0
                                };

                                (idx, attr)
                            } else {
                                // For background, we use the scanline number as Y and the X fetch counter
                                let x_index = ((self.scroll_x >> 3) + (state.fetcher_x)) & 0x1F;
                                let y_index = self.y.wrapping_add(self.scroll_y) >> 3;
                                let tile_map_idx = ((y_index as u16) << 5) | (x_index as u16);

                                let idx = self.read_bg_tile_index(tile_map_idx);
                                let attr = if self.cgb_mode {
                                    self.read_bg_tile_attributes(tile_map_idx)
                                } else {
                                    0
                                };

                                (idx, attr)
                            };

                            state.cycle += 1;
                        } else {
                            state.advance_fetcher_state()
                        }
                    }
                    PixelFetcherState::GetTileLow => {
                        if state.cycle == 0 {
                            state.buffer = [0u16; 8];
                            self.fetcher_get_tile(state, false)
                        } else {
                            state.advance_fetcher_state()
                        }
                    }
                    PixelFetcherState::GetTileHigh => {
                        if state.cycle == 0 {
                            self.fetcher_get_tile(state, true)
                        } else {
                            state.advance_fetcher_state()
                        }
                    }
                    PixelFetcherState::Push => {
                        // X flip
                        if state.tile_attr & 0x20 > 0 {
                            state.buffer.reverse();
                        }

                        // Add palette and priority bits
                        for b in &mut state.buffer {
                            *b |= state.tile_attr as u16;
                        }

                        if state.is_sprite {
                            // Add sprite index
                            for b in &mut state.buffer {
                                *b |= (state.sprite_idx as u16) << 12;
                            }

                            self.sprite_pixel_pipeline.load(state.buffer, self.cgb_mode);

                            if self.x == 0 {
                                self.sprite_pixel_pipeline
                                    .drain(8 - self.secondary_oam[(state.sprite_idx + 1) as usize]);
                            }

                            state.is_sprite = false;

                            // Remove the sprite
                            self.secondary_oam[(state.sprite_idx + 1) as usize] = 0;
                        } else {
                            if self.background_pixel_pipeline.is_empty() {
                                // Hang until pipeline is empty to load it
                                self.background_pixel_pipeline.load(state.buffer, false);

                                if !state.is_window {
                                    self.background_pixel_pipeline
                                        .drain((self.scroll_x.wrapping_add(self.x)) & 0x7);
                                } else {
                                    if self.x == 0 {
                                        self.background_pixel_pipeline
                                            .drain(7u8.wrapping_sub(self.window_x) & 0x7);
                                    }
                                }

                                state.fetcher_x += 1;
                            }
                        }

                        state.advance_fetcher_state()
                    }
                }

                // Rendering...
                if !self.background_pixel_pipeline.is_empty() & !state.is_sprite {
                    let background_pixel = self.background_pixel_pipeline.pop();
                    let sprite_pixel = self.sprite_pixel_pipeline.pop();

                    let sprite_palette = (sprite_pixel as usize & 0x10) >> 4;

                    let background_priority = if self.cgb_mode {
                        if !self
                            .lcd_control_reg
                            .contains(LcdControl::BACKGROUND_WINDOW_ENABLE_PRIORITY)
                        {
                            // If LCDC.0 is on, sprite always have priority
                            false
                        } else if (background_pixel & 0x80) == 0x80 {
                            // If the background specifies priority, it has priority
                            true
                        } else {
                            // Else, the priority is determined from the sprite attibutes
                            (sprite_pixel & 0x80) == 0x80
                        }
                    } else {
                        (sprite_pixel & 0x80) == 0x80
                    };

                    let pixel = if self.cgb_mode {
                        if (sprite_pixel & 0x300 == 0)
                            || (background_priority && (background_pixel & 0x300 != 0))
                        {
                            // Render the background pixel
                            self.cgb_bg_palette.get_rgb(
                                background_pixel as usize & 0x7,
                                (background_pixel as usize >> 8) & 3,
                            )
                        } else {
                            // Rendering the sprite pixel
                            self.cgb_obj_palette.get_rgb(
                                sprite_pixel as usize & 0x7,
                                (sprite_pixel as usize >> 8) & 3,
                            )
                        }
                    } else {
                        if (sprite_pixel & 0x300 == 0)
                            || (background_priority && (background_pixel & 0x300 != 0))
                        {
                            // Pixel is transparent or under the background. Rendering background instead
                            // Index the pixel in the palette
                            if self
                                .lcd_control_reg
                                .contains(LcdControl::BACKGROUND_WINDOW_ENABLE_PRIORITY)
                            {
                                let index = (self.dmg_bg_palette
                                    >> (((background_pixel >> 8) as u8 & 3) << 1))
                                    & 0x3;
                                self.dmg_colorized_bg_palette[index as usize]
                            } else {
                                // Renders white if background rendering is disabled
                                [0xFF, 0xFF, 0xFF]
                            }
                        } else {
                            // Rendering the sprite pixel
                            // Index the pixel in the palette
                            let index = (self.dmg_obj_palette[sprite_palette]
                                >> (((sprite_pixel >> 8) as u8 & 3) << 1))
                                & 0x3;

                            self.dmg_colorized_obj_palette[sprite_palette][index as usize]
                        }
                    };

                    let base = ((self.y as usize) * FRAME_WIDTH + (self.x as usize)) * 4;
                    if base + 3 < self.frame.len() {
                        self.frame[base..base + 3].copy_from_slice(&pixel);

                        // Alpha channel
                        self.frame[base + 3] = 0xff;

                        self.x += 1;

                        if self.x >= FRAME_WIDTH as u8 {
                            // We enter HBlank here

                            // Reset some buffers
                            self.background_pixel_pipeline = Default::default();
                            self.sprite_pixel_pipeline = Default::default();
                            self.secondary_oam = [0u8; 40];

                            if state.is_window {
                                self.window_y_counter += 1;
                            };

                            fifo_mode = FifoMode::HBlank;

                            if self
                                .lcd_status_reg
                                .contains(LcdStatus::HBANLK_INTERUPT_SOURCE)
                            {
                                bus.request_interrupt(InterruptReg::LCD_STAT);
                            }

                            // Signal to the CPU we are in HBlank for HDMA transfer
                            bus.set_hdma_hblank(true);
                        };
                    }
                }
            }
            _ => {
                // Don't render anything in HBLANK/VBLANK
            }
        }

        self.fifo_mode = fifo_mode;
    }

    fn read_bg_win_tile(&self, bank: u8, id: u8, offset: u8) -> u8 {
        // See: https://gbdev.io/pandocs/Tile_Data.html
        if self
            .lcd_control_reg
            .contains(LcdControl::BACKGROUND_WINDOW_TILE_DATA_AREA)
        {
            self.read_obj_tile(bank, id, offset)
        } else {
            let is_id_negative = id & 0x80 == 0x80;

            let addr_to_read = if !is_id_negative {
                (0x9000 | ((id as u16) << 4) | (offset as u16)) & 0x1FFF | ((bank as u16) << 13)
            } else {
                0x8800
                    | (((id as u16) << 4) & 0x7FF)
                    | (offset as u16) & 0x1FFF
                    | ((bank as u16) << 13)
            };
            self.read_vram_without_banking(addr_to_read)
        }
    }

    fn read_obj_tile(&self, bank: u8, id: u8, offset: u8) -> u8 {
        let base_addr = 0x8000;
        let addr_to_read =
            base_addr | (u16::from(id) << 4) | offset as u16 & 0x1FFF | ((bank as u16) << 13);
        self.read_vram_without_banking(addr_to_read)
    }

    fn read_bg_tile_index(&self, id: u16) -> u8 {
        // See: https://gbdev.io/pandocs/Tile_Maps.html
        if self
            .lcd_control_reg
            .contains(LcdControl::BACKGROUND_TILE_MAP_AREA)
        {
            let addr = 0x9C00 | id;
            self.read_vram_without_banking(addr)
        } else {
            let addr = 0x9800 | id;
            self.read_vram_without_banking(addr)
        }
    }

    fn read_win_tile_index(&self, id: u16) -> u8 {
        // See: https://gbdev.io/pandocs/Tile_Maps.html
        if self
            .lcd_control_reg
            .contains(LcdControl::WINDOW_TILE_MAP_AREA)
        {
            let addr = 0x9C00 | id;
            self.read_vram_without_banking(addr)
        } else {
            let addr = 0x9800 | id;
            self.read_vram_without_banking(addr)
        }
    }

    fn read_bg_tile_attributes(&self, id: u16) -> u8 {
        // See: https://gbdev.io/pandocs/Tile_Maps.html
        if self
            .lcd_control_reg
            .contains(LcdControl::BACKGROUND_TILE_MAP_AREA)
        {
            let addr = 0x9C00 | id & 0x1FFF | 0x2000;
            self.read_vram_without_banking(addr)
        } else {
            let addr = 0x9800 | id & 0x1FFF | 0x2000;
            self.read_vram_without_banking(addr)
        }
    }

    fn read_win_tile_attributes(&self, id: u16) -> u8 {
        // See: https://gbdev.io/pandocs/Tile_Maps.html
        if self
            .lcd_control_reg
            .contains(LcdControl::WINDOW_TILE_MAP_AREA)
        {
            let addr = 0x9C00 | id & 0x1FFF | 0x2000;
            self.read_vram_without_banking(addr)
        } else {
            let addr = 0x9800 | id & 0x1FFF | 0x2000;
            self.read_vram_without_banking(addr)
        }
    }

    fn get_current_vram_bank(&self) -> u16 {
        if self.cgb_mode && self.vram_bank_register {
            0x2000
        } else {
            0
        }
    }

    fn fetcher_get_tile(&self, state: &mut DrawingState, hi: bool) {
        // Decides if we load the lower or higher bits
        let plane = if hi { 1 } else { 0 };
        let bank = if self.cgb_mode {
            (state.tile_attr >> 3) & 1
        } else {
            0
        };

        let mut tile_data = if state.is_sprite {
            let sprite_size = if self.lcd_control_reg.contains(LcdControl::OBJ_SIZE) {
                15
            } else {
                7
            };

            let mut row = self
                .y
                .wrapping_sub(self.secondary_oam[state.sprite_idx as usize])
                .wrapping_add(16)
                & sprite_size;

            // Y flip
            if state.tile_attr & 0x40 > 0 {
                row = sprite_size - row;
            }

            // For 8x16 sprites, get the right index
            let tile_id = if self.lcd_control_reg.contains(LcdControl::OBJ_SIZE) {
                (state.tile_idx & 0xFE) | ((row & 0x08) >> 3)
            } else {
                state.tile_idx
            };

            self.read_obj_tile(bank, tile_id, (row << 1) | plane)
        } else {
            let mut row = if state.is_window {
                // For sprite, we select using the internal window Y counter
                self.window_y_counter & 0x7
            } else {
                // For background, we select using the scanline number as Y
                self.y.wrapping_add(self.scroll_y) & 0x7
            };

            if state.tile_attr & 0x40 > 0 {
                row = 7 - row;
            }

            self.read_bg_win_tile(bank, state.tile_idx, (row << 1) | plane)
        };

        // Put the tile data where it belongs in the buffer
        for val in &mut state.buffer {
            *val |= (tile_data as u16 & 1) << (8 | plane);
            tile_data >>= 1;
        }

        state.cycle += 1;
    }
}

fn allocate_new_frame() -> Frame {
    //   Hackish way to create fixed size boxed array.
    // I don't know of any way to do it without
    // having the data allocated on the stack at some point or using unsafe
    unsafe {
        // Safety: allocated vector has the right size for a frame array
        // (that is `FRAME_WIDTH * FRAME_HEIGHT`)
        let v: Vec<u8> = vec![0xFF; FRAME_WIDTH * FRAME_HEIGHT * 4];
        Box::from_raw(
            Box::into_raw(v.into_boxed_slice()) as *mut [u8; FRAME_WIDTH * FRAME_HEIGHT * 4]
        )
    }
}
