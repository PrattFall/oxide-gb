use bitflags::bitflags;
use std::ops::RangeInclusive;

bitflags! {
    #[derive(Default)]
    pub struct LCDC: u8 {
        const BG_AND_WINDOW_ENABLE         = 1 << 0;
        const OBJ_ENABLE                   = 1 << 1;
        const OBJ_SIZE                     = 1 << 2;
        const BG_TILE_MAP_AREA             = 1 << 3;
        const BG_AND_WINDOW_TILE_DATA_AREA = 1 << 4;
        const WINDOW_ENABLE                = 1 << 5;
        const WINDOW_TILE_MAP_AREA         = 1 << 6;
        const LCD_AND_PPU_ENABLE           = 1 << 7;
    }
}

impl LCDC {
    pub fn bg_and_window_enabled(&self) -> bool {
        self.contains(LCDC::BG_AND_WINDOW_ENABLE)
    }

    pub fn sprites_enabled(&self) -> bool {
        self.contains(LCDC::OBJ_ENABLE)
    }

    pub fn sprite_size(&self) -> (u8, u8) {
        if self.contains(LCDC::OBJ_SIZE) {
            (8, 16)
        } else {
            (8, 8)
        }
    }

    pub fn bg_tile_map_area(&self) -> RangeInclusive<u16> {
        if self.contains(LCDC::BG_TILE_MAP_AREA) {
            0x9c00..=0x9fff
        } else {
            0x9800..=0x9bff
        }
    }

    pub fn bg_and_window_tile_data_area(&self) -> RangeInclusive<u16> {
        if self.contains(LCDC::BG_AND_WINDOW_TILE_DATA_AREA) {
            0x8800..=0x97ff
        } else {
            0x8000..=0x8fff
        }
    }

    pub fn window_enabled(&self) -> bool {
        self.contains(LCDC::WINDOW_ENABLE)
    }

    pub fn window_tile_map_area(&self) -> RangeInclusive<u16> {
        if self.contains(LCDC::WINDOW_TILE_MAP_AREA) {
            0x9c00..=0x9fff
        } else {
            0x9800..=0x9bff
        }
    }

    pub fn lcd_and_ppu_enabled(&self) -> bool {
        self.contains(LCDC::LCD_AND_PPU_ENABLE)
    }
}
