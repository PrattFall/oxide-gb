use crate::memory::MemoryMap;

pub trait MemoryBankController {
    fn write_memory(&self, location: usize, value: u8);
    fn read_memory(&self, location: usize) -> u8;
}

pub struct MBC1 {
    rom_bank: usize,
    ram_bank: usize,

    memory: MemoryMap,
}

impl MemoryBankController for MBC1 {
    fn write_memory(&self, location: usize, value: u8) {
        self.memory.set_value(location, value);
    }

    fn read_memory(&self, location: usize) -> u8 {
        self.memory.get_value(location)
    }
}
