use crate::cartridge::Cartridge;
use crate::utils::u8s_to_u16;
use crate::banked_memory::BankedMemory;

pub const RAM_ENABLE_VALUE: u8 = 0xa;

pub struct MBC {
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

impl From<Cartridge> for MBC {
    fn from(cartridge: Cartridge) -> Self {
        MBC {
            banking_mode: 0,
            ram_enabled: false,
            ram: BankedMemory::of_size(cartridge.header.ram_size),
            rom: BankedMemory::from(cartridge),
            video_ram: vec![0x0000; 0xa000 - 0x8000],
            work_ram: vec![0x0000; 0xe000 - 0xc000],
            sprite_attribute_table: vec![0x0000; 0xfea0 - 0xfe00],
            io_registers: vec![0x0000; 0xff80 - 0xff00],
            high_ram: vec![0x0000; 0xffff - 0xff80],
            interrupt_enable_register: 0x0000,
        }
    }
}

impl MBC {
    pub fn get_next_u8(&self, from_location: usize) -> u8 {
        self.read(from_location + 1)
    }

    pub fn get_next_u16(&self, from_location: usize) -> u16 {
        u8s_to_u16(self.read(from_location + 2), self.read(from_location + 1))
    }

    pub fn write(&mut self, location: usize, value: u8) {
        // println!(
        //     "Writing value ({:#06x}) to location ({:#06x})",
        //     value, location
        // );
        match location {
            // Ram is enabled when the lowest 4 bits written to this range
            // are equal to 0x00a0
            0x0000..=0x1fff => self.ram_enabled = (value & 0b0000_1111) == RAM_ENABLE_VALUE,

            // The selected bank number is indicated by the lowest 5 bits
            // TODO: Mask bank number based on full size of rom
            0x2000..=0x3fff => self.rom.active_bank = usize::from(value & 0b0001_1111),

            // Ram bank is set to the lowest 2 bits
            // TODO: Write code for "large" MBC1M carts which handle this differently
            0x4000..=0x5fff => self.ram.active_bank = usize::from(value & 0b0000_0011),

            // Banking Mode is set to the lowest 2 bits
            // TODO: Handle banking mode in other operations
            0x6000..=0x7fff => self.banking_mode = value & 0b0000_0011,

            0x8000..=0x9fff => self.video_ram[location - 0x8000] = value,
            0xa000..=0xbfff => self.ram.set_at(location - 0xa000, value),
            0xc000..=0xdfff => self.work_ram[location - 0xc000] = value,
            0xfe00..=0xfe9f => self.sprite_attribute_table[location - 0xfe00] = value,
            0xff00..=0xff7f => self.io_registers[location - 0xff00] = value,
            0xff80..=0xfffe => self.high_ram[location - 0xff80] = value,
            0xffff => self.interrupt_enable_register = value,

            _ => {
                println!("Cannot write to memory location {}", location);
            }
        }
    }

    pub fn read(&self, location: usize) -> u8 {
        match location {
            0x0000..=0x3fff => self.rom.value_in_bank(0, location),
            0x4000..=0x7fff => self.rom.value_at(location - 0x4000),
            0x8000..=0x9fff => self.video_ram[location - 0x8000],
            0xa000..=0xbfff => self.ram.value_at(location - 0xa000),
            0xc000..=0xdfff => self.work_ram[location - 0xc000],
            0xe000..=0xfdff => self.work_ram[location - 0xe000],
            0xfe00..=0xfe9f => self.sprite_attribute_table[location - 0xfe00],
            0xfea0..=0xfeff => panic!("Ram Banks between 0xfea0 and 0xfeff are prohibited"),
            0xff00..=0xff7f => self.io_registers[location - 0xff00],
            0xff80..=0xfffe => self.high_ram[location - 0xff80],
            0xffff => self.interrupt_enable_register,
            _ => panic!("Cannot read from location {:#06x}", location),
        }
    }

    pub fn read_slice(&self, start: usize, end: usize) -> Vec<u8> {
        (start..=end).map(|location| self.read(location)).collect()
    }
}

mod tests {
    use crate::banked_memory;

    use super::{MBC, RAM_ENABLE_VALUE};

    fn get_mock_mbc() -> MBC {
        MBC {
            banking_mode: 0,
            ram_enabled: false,
            ram: banked_memory::BankedMemory::of_size(Some(0)),
            rom: banked_memory::BankedMemory::of_size(Some(0)),
            video_ram: vec![0x0000; 0xa000 - 0x8000],
            work_ram: vec![0x0000; 0xe000 - 0xc000],
            sprite_attribute_table: vec![0x0000; 0xfea0 - 0xfe00],
            io_registers: vec![0x0000; 0xff80 - 0xff00],
            high_ram: vec![0x0000; 0xffff - 0xff80],
            interrupt_enable_register: 0x0000,
        }
    }

    #[test]
    fn test_write_ram_enable() {
        let mut value = get_mock_mbc();

        assert!(!value.ram_enabled);

        value.write(0x0000, RAM_ENABLE_VALUE);

        assert!(value.ram_enabled);
    }
}