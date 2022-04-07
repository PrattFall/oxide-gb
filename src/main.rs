use std::fs::File;
use std::io;

mod cartridge;
mod cartridge_header;
mod cartridge_type;
mod cpu;
mod cpu_registers;
mod flag_register;
mod mbc;
mod utils;
mod tile;

use crate::cpu::Cpu;
use crate::mbc::MBC;

fn main() -> io::Result<()> {
    let f = File::open("test_games/test.gb")?;
    let cartridge = cartridge::Cartridge::from(f);
    let mut memory = MBC::from(cartridge);
    let mut cpu = Cpu::new(memory);

    // Skip over the Boot Rom
    cpu.program_counter = 0x100;

    loop {
        cpu.apply_operation();
    }
}
