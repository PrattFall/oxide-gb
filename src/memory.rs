use crate::cartridge_header::RamSpec;

type MemoryBank = Vec<u8>;

pub struct MemoryBankController {
    current_bank: u8,
    num_banks: u8,
    banks: Vec<MemoryBank>,
}

impl MemoryBankController {
    pub fn new(spec: RamSpec) -> MemoryBankController {
        MemoryBankController {
            current_bank: 0,
            num_banks: spec.banks,
            banks: vec![vec![0; usize::from(spec.size)]; usize::from(spec.banks)],
        }
    }

    pub fn get_byte(&self, location: u16) -> u8 {
        self.banks[usize::from(self.current_bank)][usize::from(location)]
    }

    pub fn set_byte(&mut self, location: u16, value: u8) {
        self.banks[usize::from(self.current_bank)][usize::from(location)] = value;
    }
}
