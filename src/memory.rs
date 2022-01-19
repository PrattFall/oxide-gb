pub type MemoryBank = Vec<u8>;

pub struct MemoryMap {
    active_rom_bank: usize,
    active_work_ram: usize,
    echo_ram: MemoryBank,
    external_ram: MemoryBank,
    high_ram: MemoryBank,
    interrupt_enable_register: u8,
    io_registers: MemoryBank,
    rom_bank_fixed: MemoryBank,
    rom_banks: Vec<MemoryBank>,
    sprite_attributes: MemoryBank,
    video_ram: MemoryBank,
    work_ram: Vec<MemoryBank>,
    work_ram_fixed: MemoryBank,
}

impl MemoryMap {
    pub fn set_value(&mut self, location: usize, value: u8) {
        match location {
            0x0000..=0x7FFF => {
                panic!("That's ROM, idiot");
            }
            0x8000..=0x9FFF => {
                self.video_ram[location - 0x8000] = value;
            }
            0xA000..=0xBFFF => {
                self.external_ram[location - 0xA000] = value;
            }
            0xC000..=0xCFFF => {
                self.work_ram_fixed[location - 0xC000] = value;
            }
            0xD000..=0xDFFF => {
                self.work_ram[self.active_work_ram][location - 0xD000] = value;
            }
            0xE000..=0xFDFF => {
                panic!("Echo RAM");
            }
            0xFE00..=0xFE9F => {
                self.sprite_attributes[location - 0xFE00] = value;
            }
            0xFEA0..=0xFEFF => {
                panic!("Not usable!");
            }
            0xFF00..=0xFF7F => {
                self.io_registers[location - 0xFF00] = value;
            }
            0xFF80..=0xFFFE => {
                self.high_ram[location - 0xFF80] = value;
            }
            0xFFFF => {
                self.interrupt_enable_register = value;
            }
            _ => {
                panic!("Not Implemented");
            }
        }
    }

    pub fn get_value(&mut self, location: usize) -> u8 {
        match location {
            0x0000..=0x3FFF => self.rom_bank_fixed[location],
            0x4000..=0x7FFF => self.rom_banks[self.active_rom_bank][location - 0x4000],
            0x8000..=0x9FFF => self.video_ram[location - 0x8000],
            0xA000..=0xBFFF => self.external_ram[location - 0xA000],
            0xC000..=0xCFFF => self.work_ram_fixed[location - 0xC000],
            0xD000..=0xDFFF => self.work_ram[self.active_work_ram][location - 0xD000],
            0xE000..=0xFDFF => self.echo_ram[location - 0xE000],
            0xFE00..=0xFE9F => self.sprite_attributes[location - 0xFE00],
            0xFEA0..=0xFEFF => panic!("NO"),
            0xFF00..=0xFF7F => self.io_registers[location - 0xFF00],
            0xFF80..=0xFFFE => self.high_ram[location - 0xFF00],
            0xFFFF => self.interrupt_enable_register,
            _ => panic!("Not Implemented"),
        }
    }

    pub fn set_rom_bank(&mut self, bank: usize) {
        self.active_rom_bank = bank;
    }

    pub fn set_work_ram_bank(&mut self, bank: usize) {
        self.active_work_ram = bank;
    }
}
