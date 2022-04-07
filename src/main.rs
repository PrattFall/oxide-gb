use std::fs::File;
use std::io;

mod cartridge;
mod cartridge_header;
mod cartridge_type;
mod cpu;
mod cpu_registers;
mod flag_register;
mod mbc1;
mod memory_bank_controller;
mod no_mbc;
mod utils;
mod tile;
mod render;

use crate::cartridge_type::CartridgeType;
use crate::cpu::Cpu;
use crate::mbc1::MBC1;
use crate::memory_bank_controller::MemoryBankController;
use crate::no_mbc::NoMBC;
use crate::render::render;

pub fn make_controller(cartridge: cartridge::Cartridge) -> Box<dyn MemoryBankController> {
    match cartridge.header.cartridge_type {
        CartridgeType::MBC1 | CartridgeType::MBC1Ram | CartridgeType::MBC1RamBattery => {
            Box::new(MBC1::from(cartridge))
        }
        _ => Box::new(NoMBC::from(cartridge)),
    }
}

fn main() -> io::Result<()> {
    let f = File::open("test_games/test.gb")?;
    let cartridge = cartridge::Cartridge::from(f);
    let mut memory = make_controller(cartridge);
    let mut cpu = Cpu::default();

    // Skip over the Boot Rom
    cpu.program_counter = 0x100;

    render().unwrap();

    // loop {
    //     cpu.apply_operation(&mut *memory);
    // }

    Ok(())
}
