#[macro_use]
extern crate glium;

mod cartridge;
mod cartridge_header;
mod cartridge_type;
mod cpu;
mod cpu_registers;
mod flag_register;
mod mbc;
mod render;
mod tile;
mod utils;
mod lcdc;
mod video;

use std::fs::File;
use std::io;

use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::mbc::MBC;
// use crate::render::render;
use crate::video::Video;

fn main() -> io::Result<()> {
    let f = File::open("test_games/test.gb").unwrap();
    let cartridge = Cartridge::from(f);
    let mut memory = MBC::from(cartridge);
    let mut cpu = Cpu::default();
    let v = Video { lcdc: 0b00001000 };

    // Skip over the Boot Rom
    cpu.program_counter = 0x100;

    // render();

    loop {
        {
            cpu.apply_operation(&mut memory);
        }

        let tile = v.get_tile(memory.video_ram.clone(), 0);
        println!("{:?}", tile);
    }
}
