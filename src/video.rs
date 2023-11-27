use crate::lcdc::LCDC;
use crate::mbc::MBC;
use crate::pixel::Pixel;
use crate::tile::Tile;
use crate::tile_dictionary::TileDictionary;

pub type Frame = Vec<Vec<Pixel>>;

pub const SCREEN_HEIGHT: u8 = 144;
pub const SCREEN_WIDTH: u8 = 160;

// The maximum rendered background size. Larger than the height and width because
// there is overdraw.
pub const BACKGROUND_SIZE: usize = 256;

pub struct VideoBackground {
    pixels: [[Pixel; BACKGROUND_SIZE]; BACKGROUND_SIZE],
}

impl Default for VideoBackground {
    fn default() -> Self {
        VideoBackground {
            pixels: [[Pixel::Lightest; BACKGROUND_SIZE]; BACKGROUND_SIZE],
        }
    }
}

#[derive(Default)]
pub struct Video {
    tiles: TileDictionary,
}

impl Video {
    pub fn blank_frame() -> Frame {
        let row = vec![Pixel::Lightest; SCREEN_WIDTH.into()];
        vec![row; SCREEN_HEIGHT.into()]
    }

    fn build_tile_map(ram: MBC, tile_index: u8) -> (Vec<Tile>, Vec<Tile>) {
        (vec![], vec![])
    }

    pub fn collect_tiles(&mut self, lcdc: LCDC, ram: &MBC) {
        for i in 0..256 {
            self.tiles.set(i,Tile::from_ram(lcdc, &ram, i));
        }
    }

    fn compose_tiles(&self, tiles: Vec<Tile>) -> Frame {
        let mut result = Video::blank_frame();
        let mut i: usize = 0;

        for tile_row in 0..32 {
            for tile_col in 0..32 {
                for row in 0..8 {
                    for col in 0..8 {
                        result[tile_row * 8 + row][tile_col * 8 + col] = tiles[i][&row][&col];
                        i += 1;
                    }
                }
            }
        }

        result
    }
}
