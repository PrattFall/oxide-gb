const CARTRIDGE_TYPE_LOCATION: usize = 0x147;

#[derive(Debug)]
pub enum CartridgeType {
    RomOnly,
    MBC1,
    MBC1Ram,
    MBC1RamBattery,
    MBC2,
    MBC2Battery,
    RomRam,
    RomRamBattery,
    MMM01,
    MMM01Ram,
    MMM01RamBattery,
    MBC3TimerBattery,
    MBC3TimerRamBattery,
    MBC3,
    MBC3Ram,
    MBC3RamBattery,
    MBC5,
    MBC5Ram,
    MBC5RamBattery,
    MBC5Rumble,
    MBC5RumbleRam,
    MBC5RumbleRamBattery,
    MBC6,
    MBC7SensorRumbleRamBattery,
    PocketCamera,
    BandaiTama5,
    HuC3,
    HuC1RamBattery,
    Unknown,
}

impl CartridgeType {
    pub fn from_cartridge(cartridge: &[u8]) -> CartridgeType {
        match cartridge[CARTRIDGE_TYPE_LOCATION] {
            0x00 => CartridgeType::RomOnly,
            0x01 => CartridgeType::MBC1,
            0x02 => CartridgeType::MBC1Ram,
            0x03 => CartridgeType::MBC1RamBattery,
            0x05 => CartridgeType::MBC2,
            0x06 => CartridgeType::MBC2Battery,
            0x08 => CartridgeType::RomRam,
            0x09 => CartridgeType::RomRamBattery,
            0x0B => CartridgeType::MMM01,
            0x0C => CartridgeType::MMM01Ram,
            0x0D => CartridgeType::MMM01RamBattery,
            0x0F => CartridgeType::MBC3TimerBattery,
            0x10 => CartridgeType::MBC3TimerRamBattery,
            0x11 => CartridgeType::MBC3,
            0x12 => CartridgeType::MBC3Ram,
            0x13 => CartridgeType::MBC3RamBattery,
            0x19 => CartridgeType::MBC5,
            0x1A => CartridgeType::MBC5Ram,
            0x1B => CartridgeType::MBC5RamBattery,
            0x1C => CartridgeType::MBC5Rumble,
            0x1D => CartridgeType::MBC5RumbleRam,
            0x1E => CartridgeType::MBC5RumbleRamBattery,
            0x20 => CartridgeType::MBC6,
            0x22 => CartridgeType::MBC7SensorRumbleRamBattery,
            0xFC => CartridgeType::PocketCamera,
            0xFD => CartridgeType::BandaiTama5,
            0xFE => CartridgeType::HuC3,
            0xFF => CartridgeType::HuC1RamBattery,
            _ => CartridgeType::Unknown,
        }
    }
}
