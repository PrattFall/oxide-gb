use crate::cartridge_header::CartridgeHeader;
use crate::cartridge_type::CartridgeType;

const ROM_BANK_SIZE: usize = 16000;

pub trait MemoryBankController {
    fn write_memory(&mut self, location: usize, value: u8);
    fn read_memory(&self, location: usize) -> u8;
}

pub struct BankedMemory {
    pub active_bank: usize,
    pub banks: Vec<Vec<u8>>,
}

impl BankedMemory {
    fn value_at(&self, location: usize) -> u8 {
        self.banks[self.active_bank][location]
    }

    fn value_in_bank(&self, bank: usize, location: usize) -> u8 {
        self.banks[bank][location]
    }
}

pub fn make_controller(cartridge: &[u8]) -> impl MemoryBankController {
    let header = CartridgeHeader::from_binary(cartridge);

    match header.cartridge_type {
        CartridgeType::MBC1 | CartridgeType::MBC1Ram | CartridgeType::MBC1RamBattery => {
            MBC1::from_cartridge(&header, cartridge)
        }
        _ => MBC1::from_cartridge(&header, cartridge),
    }
}

pub struct MBC1 {
    rom: BankedMemory,
    ram: BankedMemory,
    ram_enabled: bool,
    banking_mode: u8, // Only need 2 bits
}

impl MBC1 {
    fn from_cartridge(header: &CartridgeHeader, cartridge: &[u8]) -> Self {
        MBC1 {
            banking_mode: 0,
            ram_enabled: false,
            rom: BankedMemory {
                active_bank: 1,
                banks: cartridge
                    .chunks(ROM_BANK_SIZE)
                    .map(|x| x.to_vec())
                    .collect(),
            },
            ram: BankedMemory {
                active_bank: 0,
                banks: vec![vec![0; header.ram_size.size]; header.ram_size.banks],
            },
        }
    }
}

impl MemoryBankController for MBC1 {
    fn write_memory(&mut self, location: usize, value: u8) {
        match location {
            // Ram is enabled when the lowest 4 bits written to this range
            // are equal to 0x00a0
            0x0000..=0x1fff => self.ram_enabled = (value & 0xFF) == 0x00a0,

            // The selected bank number is indicated by the lowest 5 bits
            // TODO: Mask bank number based on full size of rom
            0x2000..=0x3fff => self.rom.active_bank = usize::from(value & 0b0001_1111),

            // Ram bank is set to the lowest 2 bits
            // TODO: Write code for "large" MBC1M carts which handle this
            // differently
            0x4000..=0x5fff => self.ram.active_bank = usize::from(value & 0xF),

            0x6000..=0x7fff => {}
            _ => {
                panic!("Cannot write to memory location {}", location);
            }
        }
    }

    fn read_memory(&self, location: usize) -> u8 {
        match location {
            0x0000..=0x7fff => self.rom.value_in_bank(0, location),
            _ => 0x0000,
        }
    }
}
