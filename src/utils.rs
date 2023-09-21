use num;

pub fn u8s_to_u16(x: u8, y: u8) -> u16 {
    ((x as u16) << 8) + (y as u16)
}

pub fn u16_to_u8s(value: u16) -> [u8; 2] {
    [(value >> 8) as u8, value as u8]
}

pub trait BitWise<T: num::Integer> {
    fn is_bit_set(&self, bit: T) -> bool;
    fn set_bit(&self, bit: T) -> T;
    fn unset_bit(&self, bit: T) -> T;
    fn toggle_bit(&self, bit: T, should_set: bool) -> T;
}

impl BitWise<u8> for u8 {
    fn is_bit_set(&self, bit: u8) -> bool {
        self & bit != 0
    }

    fn set_bit(&self, bit: u8) -> u8 {
        self | bit
    }

    fn unset_bit(&self, bit: u8) -> u8 {
        self & !bit
    }

    fn toggle_bit(&self, bit: u8, should_set: bool) -> u8 {
        if should_set {
            self.set_bit(bit)
        } else {
            self.unset_bit(bit)
        }
    }
}

pub trait Carryable<T: num::Integer> {
    fn add_should_half_carry(&self, b: T) -> bool;
    fn add_should_carry(&self, b: T) -> bool;
    fn dec_should_half_carry(&self) -> bool;
    fn inc_should_half_carry(&self) -> bool;
    fn sub_should_half_carry(&self, b: T) -> bool;
    fn sub_should_carry(&self, b: T) -> bool;
}

impl Carryable<u8> for u8 {
    fn add_should_half_carry(&self, b: u8) -> bool {
        ((self & 0xf) + (b & 0xf) & 0x10) == 0x10
    }

    fn add_should_carry(&self, b: u8) -> bool {
        ((*self as u16 + b as u16) & 0x100) != 0
    }

    fn dec_should_half_carry(&self) -> bool {
        self & 0x0f == 0x0f
    }

    fn inc_should_half_carry(&self) -> bool {
        self & 0x0f == 0x00
    }

    fn sub_should_half_carry(&self, b: u8) -> bool {
        ((self & 0xf) as i16).wrapping_sub((b & 0xf) as i16) < 0
    }

    fn sub_should_carry(&self, b: u8) -> bool {
        &b > self
    }
}

impl Carryable<u16> for u16 {
    fn add_should_half_carry(&self, b: u16) -> bool {
        (self & 0xfff) + (b & 0xfff) > 0xfff
    }

    fn add_should_carry(&self, b: u16) -> bool {
        ((*self as u32 + b as u32) & 0x10000) != 0
    }

    fn dec_should_half_carry(&self) -> bool {
        self & 0x0f == 0x0f
    }

    fn inc_should_half_carry(&self) -> bool {
        self & 0x0f == 0x00
    }

    fn sub_should_half_carry(&self, b: u16) -> bool {
        ((self & 0xfff) as i32).wrapping_sub((b & 0xfff) as i32) < 0
    }

    fn sub_should_carry(&self, b: u16) -> bool {
        &b > self
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{BitWise, Carryable};

    #[test]
    fn add_half_carry_check_10plus12() {
        assert!((10 as u8).add_should_half_carry(12));
    }

    #[test]
    fn add_half_carry_check_5plus4() {
        assert!(!(5 as u8).add_should_half_carry(4));
    }

    #[test]
    fn bitwise_is_bit_set_0() {
        assert!(1.is_bit_set(1 << 0));
    }

    #[test]
    fn bitwise_is_bit_set_1() {
        assert!(2.is_bit_set(1 << 1))
    }

    #[test]
    fn bitwise_is_bit_set_3c() {
        assert!(!(0x3c as u8).is_bit_set(1 << 0));
        assert!(!(0x3c as u8).is_bit_set(1 << 1));
        assert!((0x3c as u8).is_bit_set(1 << 2));
        assert!((0x3c as u8).is_bit_set(1 << 3));
        assert!((0x3c as u8).is_bit_set(1 << 4));
        assert!((0x3c as u8).is_bit_set(1 << 5));
        assert!(!(0x3c as u8).is_bit_set(1 << 6));
        assert!(!(0x3c as u8).is_bit_set(1 << 7));
    }

    #[test]
    fn bitwise_is_bit_set_7e() {
        assert!(!(0x7e as u8).is_bit_set(1 << 0));
        assert!((0x7e as u8).is_bit_set(1 << 1));
        assert!((0x7e as u8).is_bit_set(1 << 2));
        assert!((0x7e as u8).is_bit_set(1 << 3));
        assert!((0x7e as u8).is_bit_set(1 << 4));
        assert!((0x7e as u8).is_bit_set(1 << 5));
        assert!((0x7e as u8).is_bit_set(1 << 6));
        assert!(!(0x7e as u8).is_bit_set(1 << 7));
    }

    #[test]
    fn bitwise_is_bit_set_excludes_unset_bits() {
        assert!(!2.is_bit_set(1 << 0));
    }

    #[test]
    fn bitwise_is_bit_set_for_3() {
        assert!(3.is_bit_set(1 << 0));
        assert!(3.is_bit_set(1 << 1));
    }
}
