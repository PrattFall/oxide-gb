use crate::tile::Tile;

const MAX_TILES: usize = 384;

pub struct TileDictionary {
    tiles: [Tile; MAX_TILES]
}

impl Default for TileDictionary {
    fn default() -> Self {
        let tile: Tile = Tile::default();

        TileDictionary {
            tiles: [tile; MAX_TILES]
        }
    }
}

impl TileDictionary {
    pub fn set(&mut self, index: usize, value: Tile) {
        self.tiles[index] = value;
    }
}