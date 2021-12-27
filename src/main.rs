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
mod sm83;

use crate::cartridge_header::CartridgeHeader;

// struct PPU {}
// struct LCD {}
// struct Tile {}
// struct Background {}
// struct Window {}
// struct Sprite {}
// struct APU {}

fn main() -> io::Result<()> {
    let f = File::open("Tetris (World) (Rev 1).gb")?;
    let mut reader = BufReader::new(f);
    let mut cartridge_buffer: Vec<u8> = Vec::new();

    reader.read_to_end(&mut cartridge_buffer)?;

    let header = CartridgeHeader::from_binary(&cartridge_buffer);
    let mut cpu = sm83::SharpSM83::new();
    let mut memory = vec![0x00; usize::from(u16::MAX)];

    cpu.program_counter = 0x100;
    cpu.debug = true;

    loop {
        let op = &cartridge_buffer[usize::from(cpu.program_counter)];

        cpu.apply_operation(op, &cartridge_buffer, &mut memory);

        if cpu.program_counter >= 0x7ff0 {
            //arbitrary breaking point
            break;
        }
    }

    Ok(())
}
