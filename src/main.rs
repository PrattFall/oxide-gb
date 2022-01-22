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
use std::io::prelude::*;
use std::io::BufReader;

mod cartridge_header;
mod cartridge_type;
mod flag_register;
mod memory_bank_controller;
mod sm83;

use memory_bank_controller::make_controller;

// struct PPU {}
// struct LCD {}
// struct Tile {}
// struct Background {}
// struct Window {}
// struct Sprite {}
// struct APU {}

fn main() -> io::Result<()> {
    let f = File::open("test_games/test.gb")?;
    let mut reader = BufReader::new(f);
    let mut cartridge_buffer: Vec<u8> = Vec::new();

    reader.read_to_end(&mut cartridge_buffer)?;

    let mut cpu = sm83::SharpSM83::new();
    let mut memory = make_controller(&cartridge_buffer);

    // Skip to the start of the actual program for now
    cpu.program_counter = 0x100;
    cpu.debug = true;

    loop {
        cpu.apply_operation(&cartridge_buffer, &mut memory);

        if cpu.program_counter >= 0x7ff0 {
            //arbitrary breaking point
            break;
        }
    }

    Ok(())
}
