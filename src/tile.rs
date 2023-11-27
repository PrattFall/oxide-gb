use std::ops::{Index, IndexMut};

use crate::lcdc::LCDC;
use crate::mbc::MBC;
use crate::pixel::Pixel;
use crate::utils::BitWise;

const TILE_DIMENSION: usize = 8;
const TILE_SIZE_BYTES: usize = 16;

#[derive(Clone, Copy, Debug)]
pub struct TileRow {
    pixels: [Pixel; TILE_DIMENSION],
}

impl Default for TileRow {
    fn default() -> Self {
        TileRow {
            pixels: [Pixel::Lightest; TILE_DIMENSION],
        }
    }
}

impl From<(u8, u8)> for TileRow {
    fn from((b1, b2): (u8, u8)) -> Self {
        let mut pixels = [Pixel::Lightest; TILE_DIMENSION];

        for i in 0..8u8 {
            pixels[i as usize] = match (b2.is_bit_set(1 << i), b1.is_bit_set(1 << i)) {
                (true, true) => Pixel::Lightest,
                (true, false) => Pixel::Light,
                (false, true) => Pixel::Dark,
                (false, false) => Pixel::Darkest,
            };
        }

        TileRow { pixels }
    }
}

impl PartialEq for TileRow {
    fn eq(&self, other: &Self) -> bool {
        if other.pixels.len() != self.pixels.len() {
            return false;
        }

        for i in 0..self.pixels.len() {
            if other.pixels[i] != self.pixels[i] {
                return false;
            }
        }

        true
    }
}

impl Index<&'_ usize> for TileRow {
    type Output = Pixel;

    fn index(&self, col: &usize) -> &Pixel {
        &self.pixels[*col]
    }
}

impl IndexMut<&'_ usize> for TileRow {
    fn index_mut(&mut self, col: &usize) -> &mut Pixel {
        &mut self.pixels[*col]
    }
}

#[derive(Default, Clone, Copy)]
pub struct Tile {
    rows: [TileRow; TILE_DIMENSION],
}

impl Index<&'_ usize> for Tile {
    type Output = TileRow;

    fn index(&self, row: &usize) -> &TileRow {
        &self.rows[*row]
    }
}

impl IndexMut<&'_ usize> for Tile {
    fn index_mut(&mut self, row: &usize) -> &mut TileRow {
        &mut self.rows[*row]
    }
}

impl Tile {
    // TODO: Actually handle i8 when lcdc.4 is active
    // https://gbdev.io/pandocs/Tile_Data.html
    pub fn from_ram(lcdc: LCDC, ram: &MBC, tile_index: usize) -> Tile {
        let prefix = *lcdc.bg_and_window_tile_data_area().start() as usize;

        let vram_start = prefix + tile_index * TILE_SIZE_BYTES;
        let vram_offset = vram_start + TILE_SIZE_BYTES;

        let bytes = ram.read_slice(vram_start, vram_offset);
        let byte_pairs = bytes.chunks(2);

        if byte_pairs.len() != 8 {
            panic!("Not enough byte pairs for a tile! Check that `vram_start` and `vram_offset` are correct");
        }

        let mut result: Tile = Tile::default();

        for (i, pair) in byte_pairs.enumerate() {
            match pair {
                [b1, b2] => {
                    result[&i] = TileRow::from((*b1, *b2));
                }
                _ => panic!("Uneven bytes found when accessing tile"),
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::pixel::Pixel;
    use crate::tile::TileRow;

    #[test]
    fn test_bytes_to_tile_row() {
        let expected = TileRow {
            pixels: [
                Pixel::Darkest,
                Pixel::Light,
                Pixel::Lightest,
                Pixel::Lightest,
                Pixel::Lightest,
                Pixel::Lightest,
                Pixel::Light,
                Pixel::Darkest,
            ],
        };

        assert!(TileRow::from((0x3c, 0x7e)) == expected);
    }
}
