#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FlagRegisterValue {
    Carry = 1 << 4,
    HalfCarry = 1 << 5,
    Negative = 1 << 6,
    Zero = 1 << 7,
}

impl Into<u8> for FlagRegisterValue {
    fn into(self) -> u8 {
        self as u8
    }
}
