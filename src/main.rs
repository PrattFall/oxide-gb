#[macro_use]
extern crate glium;

mod banked_memory;
mod cartridge;
mod cartridge_header;
mod cartridge_type;
mod cpu;
mod cpu_registers;
mod flag_register;
mod lcdc;
mod mbc;
mod ops;
mod pixel;
mod prefix_ops;
mod render_opengl;
mod tile;
mod tile_dictionary;
mod utils;
mod video;

use std::env;
use std::fs::File;
use std::io;
use std::path::Path;

use crate::render_opengl::render;
use crate::video::Video;

fn read_cartridge() -> impl AsRef<Path> {
    let mut cartridge_file = None;

    for arg in env::args() {
        cartridge_file = Some(arg);
    }

    if cartridge_file.is_none() {
        panic!("Must provide a cartridge file path");
    }

    cartridge_file.unwrap()
}

fn main() -> io::Result<()> {
    let _v = Video::default();

    render(File::open(read_cartridge()).unwrap());

    Ok(())
}
