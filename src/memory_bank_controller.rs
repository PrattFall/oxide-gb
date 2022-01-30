// use bitflags::bitflags;
use crate::utils::u8s_to_u16;

pub const ROM_BANK_SIZE: usize = 16000;

pub trait MemoryBankController {
    fn write_memory(&mut self, location: usize, value: u8);
    fn read_memory(&self, location: usize) -> u8;
    fn get_next_u8(&self, from_location: usize) -> u8 {
        self.read_memory(from_location + 1)
    }
    fn get_next_u16(&self, from_location: usize) -> u16 {
        u8s_to_u16(
            self.read_memory(from_location + 1),
            self.read_memory(from_location + 2),
        )
    }
}

pub struct BankedMemory {
    pub active_bank: usize,
    pub banks: Vec<Vec<u8>>,
}

impl BankedMemory {
    pub fn value_at(&self, location: usize) -> u8 {
        self.banks[self.active_bank][location]
    }

    pub fn value_in_bank(&self, bank: usize, location: usize) -> u8 {
        self.banks[bank][location]
    }

    pub fn set_at(&mut self, location: usize, value: u8) {
        self.banks[self.active_bank][location] = value;
    }
}

// bitflags! {
//     pub struct SpriteFlags: u8 {
//         const BGAndWindowOverObject = 0b01000000;
//         const XFlip = 0b00100000;
//         const YFlip = 0b00010000;
//         const PalleteNumberGB = 0b00001000;
//     }
// }

// pub struct Sprite {
//     pub y_position: u8,
//     pub x_position: u8,
//     pub tile_index: u8,
//     pub flags: u8,
// }

// impl Sprite {
//     fn from_bytes(bytes: (u8, u8, u8, u8)) -> Self {
//         Sprite {
//             y_position: bytes.0,
//             x_position: bytes.1,
//             tile_index: bytes.2,
//             flags: bytes.3,
//         }
//     }

//     fn from_buffer(buffer: &[u8]) -> Vec<Self> {
//         buffer
//             .chunks(4)
//             .map(|chunk| (chunk[0], chunk[1], chunk[2], chunk[3]))
//             .map(Sprite::from_bytes)
//             .collect()
//     }
// }
