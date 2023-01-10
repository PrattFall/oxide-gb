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
mod lcdc;
mod tile;
mod utils;
mod video;

use std::fs::File;
use std::io;
use std::time::{Duration, Instant};
use std::thread;

use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::mbc::MBC;
use crate::render::render;
use crate::video::Video;


fn main() -> io::Result<()> {
    let cpu_mhz = 4194304;
    let cpu_wait = Duration::from_secs(1) / cpu_mhz;

    let f = File::open("test_games/test.gb").unwrap();
    let cartridge = Cartridge::from(f);
    let mut memory = MBC::from(cartridge);
    let mut cpu = Cpu::default();
    let mut input = String::new();
    let v = Video::new();

    // Skip over the Boot Rom
    cpu.program_counter = 0x100;

    render();

    loop {
        let start = Instant::now();
        cpu.apply_operation(&mut memory);
        let elapsed = start.elapsed();

        if elapsed < cpu_wait {
            thread::sleep(cpu_wait - elapsed);
        } else {
            println!("{:?} > {:?}", elapsed, cpu_wait);
        }

        std::io::stdin().read_line(&mut input).unwrap();

        if input.trim() == "q" {
            break;
        }
    }

    Ok(())
}
