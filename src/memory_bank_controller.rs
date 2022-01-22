use crate::cartridge_header::CartridgeHeader;
use crate::cartridge_type::CartridgeType;
// use bitflags::bitflags;

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

    fn set_at(&mut self, location: usize, value: u8) {
        self.banks[self.active_bank][location] = value;
    }
}

pub fn make_controller(cartridge: &[u8]) -> impl MemoryBankController {
    let header = CartridgeHeader::from_binary(cartridge);

    match header.cartridge_type {
        CartridgeType::MBC1 | CartridgeType::MBC1Ram | CartridgeType::MBC1RamBattery => {
            MBC1::from_cartridge(&header, cartridge)
        }
        // TODO: figure out how to use options with mutable values
        _ => MBC1::from_cartridge(&header, cartridge),
    }
}

pub struct MBC1 {
    banking_mode: u8, // Only need 2 bits
    ram_enabled: bool,
    ram: BankedMemory,
    rom: BankedMemory,
    video_ram: Vec<u8>,
    work_ram: Vec<u8>,
    sprite_attribute_table: Vec<u8>,
    io_registers: Vec<u8>,
    high_ram: Vec<u8>,
    interrupt_enable_register: u8,
}

impl MBC1 {
    fn from_cartridge(header: &CartridgeHeader, cartridge: &[u8]) -> Self {
        MBC1 {
            banking_mode: 0,
            ram_enabled: false,
            ram: BankedMemory {
                active_bank: 0,
                banks: vec![vec![0; header.ram_size.size]; header.ram_size.banks],
            },
            rom: BankedMemory {
                active_bank: 1,
                banks: cartridge
                    .chunks(ROM_BANK_SIZE)
                    .map(|x| x.to_vec())
                    .collect(),
            },
            video_ram: vec![0x0000; 0x9fff - 0x8000],
            work_ram: vec![0x0000; 0xdfff - 0xc000],
            sprite_attribute_table: vec![0x0000; 0xfe9f - 0xfe00],
            io_registers: vec![0x0000; 0xff7f - 0xff00],
            high_ram: vec![0x0000; 0xfffe - 0xff80],
            interrupt_enable_register: 0x0000,
        }
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

impl MemoryBankController for MBC1 {
    fn write_memory(&mut self, location: usize, value: u8) {
        match location {
            // Ram is enabled when the lowest 4 bits written to this range
            // are equal to 0x00a0
            0x0000..=0x1fff => self.ram_enabled = (value & 0b00001111) == 0x00a0,

            // The selected bank number is indicated by the lowest 5 bits
            // TODO: Mask bank number based on full size of rom
            0x2000..=0x3fff => self.rom.active_bank = usize::from(value & 0b0001_1111),

            // Ram bank is set to the lowest 2 bits
            // TODO: Write code for "large" MBC1M carts which handle this
            // differently
            0x4000..=0x5fff => self.ram.active_bank = usize::from(value & 0b00000011),

            0x6000..=0x7fff => self.banking_mode = value & 0b00000011,

            0x8000..=0x9fff => self.video_ram[location - 0x8000] = value,
            0xa000..=0xbfff => self.ram.set_at(location - 0xa000, value),
            0xc000..=0xdfff => self.work_ram[location - 0xc000] = value,
            0xfe00..=0xfe9f => self.sprite_attribute_table[location - 0xfe00] = value,
            0xff00..=0xff7f => self.io_registers[location - 0xff00] = value,
            0xff80..=0xfffe => self.high_ram[location - 0xff80] = value,
            0xffff => self.interrupt_enable_register = value,

            _ => {
                panic!("Cannot write to memory location {}", location);
            }
        }
    }

    fn read_memory(&self, location: usize) -> u8 {
        match location {
            0x0000..=0x3fff => self.rom.value_in_bank(0, location),
            0x4000..=0x7fff => self.rom.value_at(location - 0x4000),
            0x8000..=0x9fff => self.video_ram[location - 0x8000],
            0xa000..=0xbfff => self.ram.value_at(location - 0xa000),
            0xc000..=0xdfff => self.work_ram[location - 0xc000],
            0xe000..=0xfdff => self.work_ram[location - 0xe000],
            0xfe00..=0xfe9f => self.sprite_attribute_table[location - 0xfe00],
            0xfea0..=0xfeff => panic!("Unusable"),
            0xff00..=0xff7f => self.io_registers[location - 0xff00],
            0xff80..=0xfffe => self.high_ram[location - 0xff80],
            0xffff => self.interrupt_enable_register,
            _ => panic!("Writing to location {:#06x}", location),
        }
    }
}
