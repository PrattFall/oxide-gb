use crate::cartridge::Cartridge;
use crate::cartridge_header::RAM_BANK_SIZE;

pub const ROM_BANK_SIZE: usize = 16000;

pub struct BankedMemory {
    pub active_bank: usize,
    pub banks: Vec<Vec<u8>>,
}

impl From<Cartridge> for BankedMemory {
    fn from(cartridge: Cartridge) -> Self {
        BankedMemory {
            active_bank: 1,
            banks: cartridge
                .data
                .chunks(ROM_BANK_SIZE)
                .map(|x| x.to_vec())
                .collect(),
        }
    }
}

impl BankedMemory {
    pub fn new(active_bank: usize, bank_size: usize, bank_count: usize) -> Self {
        BankedMemory {
            active_bank,
            banks: vec![vec![0; bank_size]; bank_count],
        }
    }

    pub fn of_size(size: Option<u8>) -> Self {
        match size {
            Some(ram_size) => BankedMemory::new(0, RAM_BANK_SIZE, ram_size.into()),
            None => BankedMemory::new(0, 0x8000, 1),
        }
    }

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
