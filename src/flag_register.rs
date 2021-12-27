#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FlagRegisterValue {
    C,
    H,
    N,
    Z,
    NZ,
    NC,
}

impl FlagRegisterValue {
    fn to_u8(&self) -> u8 {
        match self {
            FlagRegisterValue::C => (1 << 4),
            FlagRegisterValue::H => (1 << 5),
            FlagRegisterValue::N => (1 << 6),
            FlagRegisterValue::Z => (1 << 7),
            FlagRegisterValue::NZ => FlagRegisterValue::N.to_u8() | FlagRegisterValue::Z.to_u8(),
            FlagRegisterValue::NC => FlagRegisterValue::N.to_u8() | FlagRegisterValue::C.to_u8(),
        }
    }
}

pub trait FlagRegister<T> {
    fn contains_flag(&self, flag: T) -> bool;
    fn set_flag(&self, flag: T) -> Self;
    fn unset_flag(&self, flag: T) -> Self;
}

impl FlagRegister<FlagRegisterValue> for u8 {
    fn contains_flag(&self, flag: FlagRegisterValue) -> bool {
        self & flag.to_u8() == flag.to_u8()
    }

    fn set_flag(&self, flag: FlagRegisterValue) -> Self {
        self | flag.to_u8()
    }

    fn unset_flag(&self, flag: FlagRegisterValue) -> Self {
        self & !flag.to_u8()
    }
}
