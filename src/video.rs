use crate::utils::BitWise;

pub type Pixel = (u8, u8, u8, u8);

pub const DARKEST_GREEN: Pixel = (15, 56, 15, 0);
pub const DARK_GREEN: Pixel = (48, 98, 48, 0);
pub const LIGHT_GREEN: Pixel = (139, 172, 15, 0);
pub const LIGHTEST_GREEN: Pixel = (155, 188, 15, 0);

const TILE_SIZE_BYTES: u8 = 16;

pub struct Video {
    pub lcdc: u8,
}


impl Video {
    fn tile_row(b1: u8, b2: u8) -> Vec<Pixel> {
        (0..8)
            .enumerate()
            .map(
                |(i, _)| match (b2.is_bit_set(i as u8), b1.is_bit_set(i as u8)) {
                    (true, true) => LIGHTEST_GREEN,
                    (true, false) => LIGHT_GREEN,
                    (false, true) => DARK_GREEN,
                    (false, false) => DARKEST_GREEN,
                },
                )
            .collect()
    }

    pub fn get_tile(&self, video_ram: Vec<u8>, tile_index: u8) -> Vec<Vec<Pixel>> {
        let prefix = if self.lcdc.is_bit_set(4) {
            0x0000
        } else {
            0x1000
        };

        let vram_start = prefix + (tile_index * TILE_SIZE_BYTES) as usize;
        let vram_offset = vram_start + TILE_SIZE_BYTES as usize;

        video_ram[vram_start..vram_offset]
            .chunks(2)
            .map(|chunk| match chunk {
                [b1, b2] => {
                    println!("thing");
                    Video::tile_row(*b1, *b2)
                }
                _ => {
                    panic!("Uneven bytes found when accessing tile")
                }
            })
            .collect()
    }
}
