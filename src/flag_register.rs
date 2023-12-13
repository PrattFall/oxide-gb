use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct FlagRegisterValue: u8 {
        const CARRY      = 1 << 4;
        const HALF_CARRY = 1 << 5;
        const NEGATIVE   = 1 << 6;
        const ZERO       = 1 << 7;
    }
}

impl Into<u8> for FlagRegisterValue {
    fn into(self) -> u8 {
        self.bits() as u8
    }
}
