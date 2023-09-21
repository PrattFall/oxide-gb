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

#[cfg(test)]
mod tests {
    use crate::lcdc::LCDC;

    #[test]
    fn bg_and_window_enabled() {
        assert!(LCDC::BG_AND_WINDOW_ENABLE.bg_and_window_enabled());
        assert!(!LCDC::OBJ_ENABLE.bg_and_window_enabled());
    }

    #[test]
    fn sprites_enabled() {
        assert!(LCDC::OBJ_ENABLE.sprites_enabled());
        assert!(!LCDC::BG_TILE_MAP_AREA.sprites_enabled());
    }

    #[test]
    fn sprite_size() {
        assert!(LCDC::OBJ_SIZE.sprite_size() == (8, 16));
        assert!(LCDC::OBJ_ENABLE.sprite_size() == (8, 8));
    }

    #[test]
    fn bg_tile_map_area() {
        assert!(LCDC::BG_TILE_MAP_AREA.bg_tile_map_area() == (0x9c00..=0x9fff));
        assert!(LCDC::OBJ_SIZE.bg_tile_map_area() == (0x9800..=0x9bff));
    }

    #[test]
    fn bg_and_window_tile_data_area() {
        assert!(
            LCDC::BG_AND_WINDOW_TILE_DATA_AREA.bg_and_window_tile_data_area() == (0x8800..=0x97ff)
        );
        assert!(LCDC::OBJ_SIZE.bg_and_window_tile_data_area() == (0x8000..=0x8fff));
    }

    #[test]
    fn window_enabled() {
        assert!(LCDC::WINDOW_ENABLE.window_enabled());
        assert!(!LCDC::OBJ_SIZE.window_enabled());
    }

    #[test]
    fn window_tile_map_area() {
        assert!(LCDC::WINDOW_TILE_MAP_AREA.window_tile_map_area() == (0x9c00..=0x9fff));
        assert!(LCDC::OBJ_SIZE.window_tile_map_area() == (0x9800..=0x9bff));
    }

    #[test]
    fn lcd_and_ppu_enabled() {
        assert!(LCDC::LCD_AND_PPU_ENABLE.lcd_and_ppu_enabled());
        assert!(!LCDC::OBJ_SIZE.lcd_and_ppu_enabled());
    }
}
