use std::ascii;
use std::str;

use crate::cartridge_type::CartridgeType;

// const NINTENDO_LOGO_LOCATION: usize = 0x104;
// const NINTENDO_LOGO_END: usize = 0x133;

const GAME_TITLE_LOCATION: usize = 0x134;
const GAME_TITLE_END: usize = 0x143;
const MANUFACTURER_CODE_LOCATION: usize = 0x13F;
const MANUFACTURER_CODE_END: usize = 0x142;
const CGB_FLAG_LOCATION: usize = 0x143;
const SGB_FLAG_LOCATION: usize = 0x146;
const ROM_SIZE_LOCATION: usize = 0x148;
const RAM_SIZE_LOCATION: usize = 0x149;

fn buffer_slice_to_string(buffer: &[u8]) -> String {
    let mut visible = String::new();

    for b in buffer {
        let part: Vec<u8> = ascii::escape_default(*b).collect();
        visible.push_str(str::from_utf8(&part).unwrap());
    }

    visible
}

#[derive(Debug)]
pub enum ColorGameboySupport {
    NoSupport,
    Both,
    OnlyColor,
}

#[derive(Debug)]
pub enum SuperGameboySupport {
    NoSupport,
    Support,
}

// Seems like it might not be often used
#[derive(Debug)]
pub enum DestinationCode {
    Japanese,
    NonJapanese,
}

fn read_cgb_flag(buffer: &[u8]) -> ColorGameboySupport {
    match buffer[CGB_FLAG_LOCATION] {
        0x80 => ColorGameboySupport::Both,
        0xC0 => ColorGameboySupport::OnlyColor,
        _ => ColorGameboySupport::NoSupport,
    }
}

fn read_sgb_flag(buffer: &[u8]) -> SuperGameboySupport {
    match buffer[SGB_FLAG_LOCATION] {
        0x03 => SuperGameboySupport::Support,
        _ => SuperGameboySupport::NoSupport,
    }
}

fn read_rom_size_bytes(buffer: &[u8]) -> u32 {
    match buffer[ROM_SIZE_LOCATION] {
        0x00 => 32000,
        0x01 => 64000,
        0x02 => 128000,
        0x03 => 256000,
        0x04 => 512000,
        0x05 => 1000000,
        0x06 => 2000000,
        0x07 => 4000000,
        0x08 => 8000000,
        0x52 => 1100000,
        0x53 => 1200000,
        0x54 => 1500000,
        _ => u32::MAX,
    }
}

#[derive(Clone, Copy)]
pub struct RamSpec {
    pub size: usize,
    pub banks: usize,
}

fn read_ram_size(buffer: &[u8]) -> Option<RamSpec> {
    match buffer[RAM_SIZE_LOCATION] {
        0x02 => Some(RamSpec {
            size: 0x8000,
            banks: 1,
        }),
        0x03 => Some(RamSpec {
            size: 0x8000,
            banks: 4,
        }),
        0x04 => Some(RamSpec {
            size: 0x8000,
            banks: 16,
        }),
        0x05 => Some(RamSpec {
            size: 0x8000,
            banks: 8,
        }),
        _ => None,
    }
}

// There are only 2 destination codes.
// Technically it's 0x00 and 0x01, but I assume anything else aside from
// 0x00 won't be "Japanese"
fn read_destination_code(buffer: &[u8]) -> DestinationCode {
    match buffer[0x14A] {
        0x00 => DestinationCode::Japanese,
        _ => DestinationCode::NonJapanese,
    }
}

pub struct CartridgeHeader {
    pub title: String,
    pub manufacturer: String,
    pub cartridge_type: CartridgeType,
    pub color_gameboy_support: ColorGameboySupport,
    pub super_gameboy_support: SuperGameboySupport,
    pub rom_size_bytes: u32,
    pub ram_size: Option<RamSpec>,
    pub destination_code: DestinationCode,
}

impl CartridgeHeader {
    pub fn from_binary(b: &[u8]) -> CartridgeHeader {
        CartridgeHeader {
            title: buffer_slice_to_string(&b[GAME_TITLE_LOCATION..GAME_TITLE_END]),
            manufacturer: buffer_slice_to_string(
                &b[MANUFACTURER_CODE_LOCATION..MANUFACTURER_CODE_END],
            ),
            cartridge_type: CartridgeType::from_cartridge(b),
            color_gameboy_support: read_cgb_flag(b),
            super_gameboy_support: read_sgb_flag(b),
            rom_size_bytes: read_rom_size_bytes(b),
            ram_size: read_ram_size(b),
            destination_code: read_destination_code(b),
        }
    }
}
