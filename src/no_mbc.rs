use crate::{
    cartridge::Cartridge, cartridge_header::RAM_BANK_SIZE,
    memory_bank_controller::MemoryBankController,
};

pub struct NoMBC {
    rom: Vec<u8>,
    ram: Option<Vec<u8>>,
    video_ram: Vec<u8>,
    work_ram: Vec<u8>,
    sprite_attribute_table: Vec<u8>,
    io_registers: Vec<u8>,
    high_ram: Vec<u8>,
    interrupt_enable_register: u8,
}

impl From<Cartridge> for NoMBC {
    fn from(cartridge: Cartridge) -> Self {
        NoMBC {
            ram: cartridge.header.ram_size.map(|_| vec![0x00; RAM_BANK_SIZE]),
            rom: cartridge.data,
            video_ram: vec![0x0000; 0x9fff - 0x8000],
            work_ram: vec![0x0000; 0xdfff - 0xc000],
            sprite_attribute_table: vec![0x0000; 0xfe9f - 0xfe00],
            io_registers: vec![0x0000; 0xff7f - 0xff00],
            high_ram: vec![0x0000; 0xfffe - 0xff80],
            interrupt_enable_register: 0x0000,
        }
    }
}

// Technically a lie, but it fulfills the same purpose
impl MemoryBankController for NoMBC {
    fn write_memory(&mut self, location: usize, value: u8) {
        match location {
            0x8000..=0x9fff => self.video_ram[location - 0x8000] = value,
            0xa000..=0xbfff => self
                .ram
                .as_mut()
                .map(|ram| ram[location - 0xa000] = value)
                .unwrap(),
            0xc000..=0xdfff => self.work_ram[location - 0xc000] = value,
            0xfe00..=0xfe9f => self.sprite_attribute_table[location - 0xfe00] = value,
            0xff00..=0xff7f => self.io_registers[location - 0xff00] = value,
            0xff80..=0xfffe => self.high_ram[location - 0xff80] = value,
            0xffff => self.interrupt_enable_register = value,

            _ => {
                println!("Cannot write to memory location {:#06x}", location);
            }
        }
    }

    fn read_memory(&self, location: usize) -> u8 {
        match location {
            0x0000..=0x7fff => self.rom[location],
            0x8000..=0x9fff => self.video_ram[location - 0x8000],
            0xa000..=0xbfff => self
                .ram
                .as_ref()
                .map(|ram| ram[location - 0xa000])
                .unwrap_or(0),
            0xc000..=0xdfff => self.work_ram[location - 0xc000],
            0xe000..=0xfdff => self.work_ram[location - 0xe000],
            0xfe00..=0xfe9f => self.sprite_attribute_table[location - 0xfe00],
            0xfea0..=0xfeff => panic!("Ram Banks between 0xfea0 and 0xfeff are unusable"),
            0xff00..=0xff7f => self.io_registers[location - 0xff00],
            0xff80..=0xfffe => self.high_ram[location - 0xff80],
            0xffff => self.interrupt_enable_register,
            _ => panic!("Writing to location {:#06x}", location),
        }
    }
}
