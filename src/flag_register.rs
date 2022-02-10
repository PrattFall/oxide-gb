#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FlagRegisterValue {
    C = 1 << 4,
    H = 1 << 5,
    N = 1 << 6,
    Z = 1 << 7,
}

impl Into<u8> for FlagRegisterValue {
    fn into(self) -> u8 {
        self as u8
    }
}
