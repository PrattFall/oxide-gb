use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use crate::cartridge_header::CartridgeHeader;

pub struct Cartridge {
    pub header: CartridgeHeader,
    pub data: Vec<u8>,
}

impl From<File> for Cartridge {
    fn from(file: File) -> Self {
        let mut reader = BufReader::new(file);
        let mut cartridge_buffer: Vec<u8> = Vec::new();

        reader.read_to_end(&mut cartridge_buffer).unwrap();

        let header = CartridgeHeader::from_binary(&cartridge_buffer);

        Cartridge {
            header: header,
            data: cartridge_buffer,
        }
    }
}
