use crate::utils::{u16_to_u8s, u8s_to_u16};
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

pub struct Registers {
    registers: HashMap<GeneralRegister, u8>,
}

impl Registers {
    pub fn set(&mut self, register: GeneralRegister, value: u8) {
        self.registers.insert(register, value);
    }

    pub fn get(&self, register: GeneralRegister) -> u8 {
        // 'get' seems overcomplicated, but I want to
        // account for missing registers if something weird happens
        *self.registers.get(&register).unwrap_or(&0)
    }

    pub fn set_combined(&mut self, register: CombinedRegister, value: u16) {
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
    }

    pub fn get_combined(&self, register: CombinedRegister) -> u16 {
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
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            registers: HashMap::from([
                (GeneralRegister::A, 0),
                (GeneralRegister::F, 0),
                (GeneralRegister::B, 0),
                (GeneralRegister::C, 0),
                (GeneralRegister::D, 0),
                (GeneralRegister::E, 0),
                (GeneralRegister::H, 0),
                (GeneralRegister::L, 0),
            ]),
        }
    }
}
