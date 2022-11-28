use crate::lcdc::LCDC;
use crate::mbc::MBC;
use crate::utils::BitWise;

pub type Pixel = (u8, u8, u8, u8);

pub const DARKEST_GREEN: Pixel = (15, 56, 15, 0);
pub const DARK_GREEN: Pixel = (48, 98, 48, 0);
pub const LIGHT_GREEN: Pixel = (139, 172, 15, 0);
pub const LIGHTEST_GREEN: Pixel = (155, 188, 15, 0);

const TILE_SIZE_BYTES: u8 = 16;

type Tile = [[Pixel; 8]; 8];
type TileDictionary = [Tile; 256];

pub struct Video {
    pub lcdc: LCDC,
    tiles: TileDictionary,
}

impl Video {
    pub fn new() -> Self {
        Video {
            lcdc: LCDC::default(),
            tiles: [[[LIGHTEST_GREEN; 8]; 8]; 256]
        }
    }

    fn build_tile_map(ram: MBC, tile_index: u8) -> (Vec<Tile>, Vec<Tile>) {
        (vec![], vec![])
    }

    pub fn collect_tiles(&self, ram: &MBC) {
        for i in 0..256 {
            self.tiles[i] = self.get_tile(&ram, i as u8)
        }
    }

    fn tile_row(b1: u8, b2: u8) -> Vec<Pixel> {
        (0..8u8)
            .map(
                |i| match (b2.is_bit_set(i), b1.is_bit_set(i)) {
                    (true, true) => LIGHTEST_GREEN,
                    (true, false) => LIGHT_GREEN,
                    (false, true) => DARK_GREEN,
                    (false, false) => DARKEST_GREEN,
                },
            )
            .collect()
    }

    // TODO: Actually handle i8 when lcdc.4 is active
    fn get_tile(&self, ram: &MBC, tile_index: u8) -> Tile {
        let prefix = *self.lcdc.bg_and_window_tile_data_area().start() as usize;
        let vram_start = prefix + (tile_index * TILE_SIZE_BYTES) as usize;
        let vram_offset = vram_start + TILE_SIZE_BYTES as usize;

        ram.read_slice(vram_start, vram_offset)
            .chunks(2)
            .map(|chunk| match chunk {
                [b1, b2] => Video::tile_row(*b1, *b2),
                _ => {
                    panic!("Uneven bytes found when accessing tile")
                }
            })
            .collect()
    }
}
