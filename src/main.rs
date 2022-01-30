// Gameboy Specs:
// - 8kb Work-RAM
// - Sharp LR35902
// - Sharp SM83
// - 8-bit data bus / 16-bit address bus
//      - 64-kb of memory access
//          - Cartridge space
//          - WRAM and Display RAM
//          - I/O (joypad, audio, graphics, and LCD)
//          - Interrupt controls
// - Resolution: 160x144
// - 4 shades of grey

use std::fs::File;
use std::io;

mod cartridge;
mod cartridge_header;
mod cartridge_type;
mod cpu_registers;
mod flag_register;
mod mbc1;
mod memory_bank_controller;
mod no_mbc;
mod sm83;
mod utils;

use crate::cartridge_type::CartridgeType;
use crate::mbc1::MBC1;
use crate::memory_bank_controller::MemoryBankController;
use crate::no_mbc::NoMBC;

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
    let mut cpu = sm83::SharpSM83::default();

    cpu.program_counter = 0x00;
    cpu.debug = true;

    loop {
        cpu.apply_operation(&mut *memory);
    }
}
