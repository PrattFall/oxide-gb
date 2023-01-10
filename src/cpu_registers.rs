use crate::flag_register::FlagRegisterValue;
use crate::utils::{u16_to_u8s, u8s_to_u16, BitWise};
use std::collections::HashMap;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum GeneralRegister {
    A, // Accumulator
    F, // Flag Register
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum CombinedRegister {
    AF,
    BC,
    DE,
    HL,
}

const ALL_REGISTERS: [GeneralRegister; 8] = [
    GeneralRegister::A,
    GeneralRegister::F,
    GeneralRegister::B,
    GeneralRegister::C,
    GeneralRegister::D,
    GeneralRegister::E,
    GeneralRegister::H,
    GeneralRegister::L,
];

// const ALL_COMBINED_REGISTERS: [CombinedRegister; 4] = [
//     CombinedRegister::AF,
//     CombinedRegister::BC,
//     CombinedRegister::DE,
//     CombinedRegister::HL,
// ];

pub struct Registers {
    registers: HashMap<GeneralRegister, u8>,
}

impl Registers {
    pub fn set(&mut self, register: GeneralRegister, value: u8) -> &mut Self {
        self.registers.insert(register, value);
        self
    }

    pub fn get(&self, register: GeneralRegister) -> u8 {
        // 'get' seems overcomplicated, but I want to
        // account for missing registers if something weird happens
        *self.registers.get(&register).unwrap_or(&0)
    }

    pub fn set16(&mut self, register: CombinedRegister, value: u16) -> &mut Self {
        let [left, right] = u16_to_u8s(value);

        match register {
            CombinedRegister::AF => {
                self.set(GeneralRegister::A, left);
                self.set(GeneralRegister::F, right);
            }
            CombinedRegister::BC => {
                self.set(GeneralRegister::B, left);
                self.set(GeneralRegister::C, right);
            }
            CombinedRegister::DE => {
                self.set(GeneralRegister::D, left);
                self.set(GeneralRegister::E, right);
            }
            CombinedRegister::HL => {
                self.set(GeneralRegister::H, left);
                self.set(GeneralRegister::L, right);
            }
        }

        self
    }

    pub fn get16(&self, register: CombinedRegister) -> u16 {
        match register {
            CombinedRegister::AF => {
                u8s_to_u16(self.get(GeneralRegister::A), self.get(GeneralRegister::F))
            }
            CombinedRegister::BC => {
                u8s_to_u16(self.get(GeneralRegister::B), self.get(GeneralRegister::C))
            }
            CombinedRegister::DE => {
                u8s_to_u16(self.get(GeneralRegister::D), self.get(GeneralRegister::E))
            }
            CombinedRegister::HL => {
                u8s_to_u16(self.get(GeneralRegister::H), self.get(GeneralRegister::L))
            }
        }
    }

    pub fn set_flag(&mut self, flag: FlagRegisterValue) -> &mut Self {
        self.set(
            GeneralRegister::F,
            self.get(GeneralRegister::F).set_bit(flag as u8),
        );

        self
    }

    pub fn unset_flag(&mut self, flag: FlagRegisterValue) -> &mut Self {
        self.set(
            GeneralRegister::F,
            self.get(GeneralRegister::F).unset_bit(flag as u8),
        );

        self
    }

    pub fn toggle_flag(&mut self, flag: FlagRegisterValue, should_set: bool) -> &mut Self {
        if should_set {
            self.set_flag(flag);
        } else {
            self.unset_flag(flag);
        }

        self
    }

    pub fn is_flag_set(&self, flag: FlagRegisterValue) -> bool {
        self.get(GeneralRegister::F).is_bit_set(flag as u8)
    }

    pub fn increment16(&mut self, register: CombinedRegister) -> u16 {
        let result = self.get16(register).wrapping_add(1);
        self.set16(register, result);
        result
    }

    pub fn decrement16(&mut self, register: CombinedRegister) -> u16 {
        let result = self.get16(register).wrapping_sub(1);
        self.set16(register, result);
        result
    }
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            registers: HashMap::from(ALL_REGISTERS.map(|r| (r, 0))),
        }
    }
}
