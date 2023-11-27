use crate::tile::Tile;

pub struct TileDictionary {
    tiles: [Tile; 256]
}

impl Default for TileDictionary {
    fn default() -> Self {
        let tile: Tile = Tile::default();

        TileDictionary {
            tiles: [tile; 256]
        }
    }
}

impl TileDictionary {
    pub fn set(&mut self, index: usize, value: Tile) {
        self.tiles[index] = value;
    }
}