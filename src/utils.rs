#[derive(PartialEq, Eq)]
pub enum Mode {
    Debug,
    Production,
}

pub const MODE: Mode = Mode::Debug;

pub fn add_should_half_carry(a: u8, b: u8) -> bool {
    ((a & 0xf) + (b & 0xf) & 0x10) == 0x10
}

pub fn add_16_should_half_carry(a: u16, b: u16) -> bool {
    (a & 0xfff) + (b & 0xfff) > 0xfff
}

pub fn sub_should_half_carry(a: u8, b: u8) -> bool {
    (a & 0xf) < (b & 0xf)
}

pub fn u8s_to_u16(x: u8, y: u8) -> u16 {
    ((x as u16) << 8) + (y as u16)
}

pub fn u16_to_u8s(value: u16) -> [u8; 2] {
    [(value >> 8) as u8, value as u8]
}

#[cfg(test)]
mod tests {
    use crate::utils::add_should_half_carry;

    #[test]
    fn add_half_carry_check_10plus12() {
        assert!(add_should_half_carry(10, 12));
    }

    #[test]
    fn add_half_carry_check_5plus4() {
        assert!(!add_should_half_carry(5, 4));
    }
}
