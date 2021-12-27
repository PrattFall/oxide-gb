use std::collections::HashMap;
use std::num::Wrapping;

// const CLOCK_MHZ: f64 = 4.194304;

fn add_should_half_carry(a: u8, b: u8) -> bool {
    ((a & 0xf) + (b & 0xf) & 0x10) == 0x10
}

fn sub_should_half_carry(a: u8, b: u8) -> bool {
    (a & 0xf) < (b & 0xf)
}

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

fn u8s_to_u16(x: u8, y: u8) -> u16 {
    ((x as u16) << 8) + (y as u16)
}

fn u16_to_u8s(value: u16) -> [u8; 2] {
    [(value >> 8) as u8, value as u8]
}

pub struct SharpSM83 {
    pub program_counter: u16,
    pub stack_pointer: u16,
    pub registers: HashMap<GeneralRegister, u8>,
    pub debug: bool,
}

impl SharpSM83 {
    pub fn new() -> SharpSM83 {
        let registers = HashMap::from([
            (GeneralRegister::A, 0),
            (GeneralRegister::F, 0),
            (GeneralRegister::B, 0),
            (GeneralRegister::C, 0),
            (GeneralRegister::D, 0),
            (GeneralRegister::E, 0),
            (GeneralRegister::H, 0),
            (GeneralRegister::L, 0),
        ]);

        SharpSM83 {
            program_counter: 0,
            stack_pointer: 0,
            registers,
            debug: false,
        }
    }

    fn set_register(&mut self, register: GeneralRegister, value: u8) {
        self.registers.insert(register, value);
    }

    fn set_register_from_memory(
        &mut self,
        memory: &mut Vec<u8>,
        register: GeneralRegister,
        location: u16,
    ) {
        let value = self.get_memory(memory, location);

        if self.debug {
            println!(
                "{:#06x}: Setting value {:#04x} to register {:?} from location {:#06x}",
                self.program_counter, value, register, location
            );
        }

        self.set_register(register, value);
        self.program_counter += 1;
    }

    fn set_combined_register(&mut self, register: CombinedRegister, value: u16) {
        let [left, right] = u16_to_u8s(value);

        match register {
            CombinedRegister::AF => {
                self.set_register(GeneralRegister::A, left);
                self.set_register(GeneralRegister::F, right);
            }
            CombinedRegister::BC => {
                self.set_register(GeneralRegister::B, left);
                self.set_register(GeneralRegister::C, right);
            }
            CombinedRegister::DE => {
                self.set_register(GeneralRegister::D, left);
                self.set_register(GeneralRegister::E, right);
            }
            CombinedRegister::HL => {
                self.set_register(GeneralRegister::H, left);
                self.set_register(GeneralRegister::L, right);
            }
        }
    }

    fn get_register(&self, register: GeneralRegister) -> u8 {
        self.registers[&register]
    }

    fn get_combined_register(&self, register: CombinedRegister) -> u16 {
        match register {
            CombinedRegister::AF => u8s_to_u16(
                self.get_register(GeneralRegister::A),
                self.get_register(GeneralRegister::F),
            ),
            CombinedRegister::BC => u8s_to_u16(
                self.get_register(GeneralRegister::B),
                self.get_register(GeneralRegister::C),
            ),
            CombinedRegister::DE => u8s_to_u16(
                self.get_register(GeneralRegister::D),
                self.get_register(GeneralRegister::E),
            ),
            CombinedRegister::HL => u8s_to_u16(
                self.get_register(GeneralRegister::H),
                self.get_register(GeneralRegister::L),
            ),
        }
    }

    fn inc(&mut self, register: GeneralRegister) {
        self.unset_flag(FlagRegisterValue::N);

        if add_should_half_carry(self.get_register(register), 1) {
            self.set_flag(FlagRegisterValue::H);
        }

        let register_value = self.get_register(register);

        if register_value == 0xFF {
            self.set_register(register, 0);
        } else {
            self.set_register(register, register_value + 1);
        }

        if self.debug {
            println!(
                "{:#06x}: Register {:?} Increased to {:#04x}",
                self.program_counter,
                register,
                self.get_register(register)
            );
        }

        self.program_counter += 1;
    }

    fn inc16(&mut self, register: CombinedRegister) {
        let register_value = self.get_combined_register(register);

        self.set_combined_register(register, register_value.wrapping_add(1));

        if self.debug {
            println!(
                "{:#06x}: Register {:?} Increased to {:#06x}",
                self.program_counter,
                register,
                self.get_combined_register(register)
            );
        }

        self.program_counter += 1;
    }

    fn ld(&mut self, a: GeneralRegister, b: GeneralRegister) {
        if self.debug {
            println!(
                "{:#06x}: Loading {:#04x} from Register {:?} to Register {:?}",
                self.program_counter,
                self.get_register(b),
                b,
                a
            );
        }

        self.set_register(a, self.get_register(b));

        self.program_counter += 1;
    }

    fn is_flag_set(&mut self, flag: FlagRegisterValue) -> bool {
        self.get_register(GeneralRegister::F).contains_flag(flag)
    }

    fn call(&mut self, cartridge: &Vec<u8>, memory: &mut Vec<u8>) {
        let call_location = self.get_next_u16(cartridge);
        let [left, right] = u16_to_u8s(self.program_counter);

        self.stack_pointer -= 2;
        self.set_memory(memory, self.stack_pointer, left);
        self.set_memory(memory, self.stack_pointer + 1, right);
        self.program_counter = call_location;
    }

    fn cpl(&mut self) {
        println!("{:#06x}: CPL", self.program_counter);

        self.set_register(
            GeneralRegister::A,
            self.get_register(GeneralRegister::A) ^ 0xFF,
        );

        self.set_flag(FlagRegisterValue::N);
        self.set_flag(FlagRegisterValue::H);

        self.program_counter += 1;
    }

    fn jp_inner(&mut self, cartridge: &Vec<u8>) {
        let jump_location = self.get_next_u16(cartridge);

        if self.debug {
            println!(
                "{:#06x}: Jumping to {:#06x}",
                self.program_counter, jump_location
            );
        }

        self.program_counter = jump_location;
    }

    fn jp(&mut self, cartridge: &Vec<u8>, flag: Option<FlagRegisterValue>) {
        match flag {
            Some(f) => {
                if self.is_flag_set(f) {
                    self.jp_inner(cartridge);
                } else {
                    if self.debug {
                        println!(
                            "Flag {:?} not set. Found {:#06x}",
                            f,
                            self.get_register(GeneralRegister::F)
                        );
                    }

                    self.program_counter += 1;
                }
            }
            None => {
                self.jp_inner(cartridge);
            }
        }
    }

    fn jr_inner(&mut self, cartridge: &Vec<u8>) {
        // Jump to program_counter + u8
        let relative_location = u16::from(self.get_next_u8(cartridge));

        if self.debug {
            println!(
                "{:#06x}: Jumping {:#06x} ops to {:#06x}",
                self.program_counter,
                relative_location,
                self.program_counter + relative_location
            );
        }

        self.program_counter += relative_location;
    }

    fn jr(&mut self, cartridge: &Vec<u8>, flag: Option<FlagRegisterValue>) {
        match flag {
            Some(f) => {
                if self.is_flag_set(f) {
                    self.jr_inner(cartridge);
                } else {
                    if self.debug {
                        println!(
                            "Flag {:?} not set. Found {:#06x}",
                            f,
                            self.get_register(GeneralRegister::F)
                        );
                    }

                    self.program_counter += 1;
                }
            }
            None => {
                self.jr_inner(cartridge);
            }
        }
    }

    fn set_flag(&mut self, flag: FlagRegisterValue) {
        self.set_register(
            GeneralRegister::F,
            self.get_register(GeneralRegister::F).set_flag(flag),
        );
    }

    fn unset_flag(&mut self, flag: FlagRegisterValue) {
        self.set_register(
            GeneralRegister::F,
            self.get_register(GeneralRegister::F).unset_flag(flag),
        );
    }

    fn dec(&mut self, register: GeneralRegister) {
        self.set_flag(FlagRegisterValue::N);

        if sub_should_half_carry(self.get_register(register), 1) {
            self.set_flag(FlagRegisterValue::H);
        }

        let register_value = self.get_register(register);

        if register_value == 0x00 {
            self.set_register(register, 0xFF);
        } else {
            self.set_register(register, register_value - 1);
        }

        if self.debug {
            println!(
                "{:#06x}: Register {:?} Decreased to {}",
                self.program_counter,
                register,
                self.get_register(register)
            );
        }

        self.program_counter += 1;
    }

    fn add(&mut self, register: GeneralRegister) {
        if self.debug {
            println!(
                "{:#04x}: Adding Register {:?}'s value ({:#04x}) from Register A ({:#04x})",
                self.program_counter,
                register,
                self.get_register(register),
                self.get_register(GeneralRegister::A)
            );
        }

        let a_val = self.get_register(GeneralRegister::A);
        let r_val = self.get_register(register);

        self.set_flag(FlagRegisterValue::N);

        if Wrapping(r_val) + Wrapping(a_val) > Wrapping(0xFF) {
            self.set_register(GeneralRegister::A, r_val.wrapping_add(a_val));
            self.set_flag(FlagRegisterValue::C);
        } else {
            self.set_register(
                GeneralRegister::A,
                self.get_register(GeneralRegister::A) - self.get_register(register),
            );
        }

        if sub_should_half_carry(a_val, self.get_register(GeneralRegister::A)) {
            self.set_flag(FlagRegisterValue::H);
        }

        if self.get_register(GeneralRegister::A) == 0 {
            self.set_flag(FlagRegisterValue::Z);
        }

        self.program_counter += 1;
    }

    fn sub(&mut self, register: GeneralRegister) {
        if self.debug {
            println!(
                "{:#04x}: Subtracting Register {:?}'s value ({:#04x}) from Register A ({:#04x})",
                self.program_counter,
                register,
                self.get_register(register),
                self.get_register(GeneralRegister::A)
            );
        }

        let a_val = self.get_register(GeneralRegister::A);
        let r_val = self.get_register(register);

        self.set_flag(FlagRegisterValue::N);

        if r_val > a_val {
            self.set_register(GeneralRegister::A, a_val.wrapping_sub(r_val));
            self.set_flag(FlagRegisterValue::C);
        } else {
            self.set_register(
                GeneralRegister::A,
                self.get_register(GeneralRegister::A) - self.get_register(register),
            );
        }

        if sub_should_half_carry(a_val, self.get_register(GeneralRegister::A)) {
            self.set_flag(FlagRegisterValue::H);
        }

        if self.get_register(GeneralRegister::A) == 0 {
            self.set_flag(FlagRegisterValue::Z);
        }

        self.program_counter += 1;
    }

    fn get_next_u8(&mut self, cartridge: &Vec<u8>) -> u8 {
        cartridge[usize::from(self.program_counter + 1)]
    }

    fn get_next_u16(&mut self, cartridge: &Vec<u8>) -> u16 {
        u8s_to_u16(
            cartridge[usize::from(self.program_counter + 1)],
            cartridge[usize::from(self.program_counter + 2)],
        )
    }

    fn ld_next_8(&mut self, cartridge: &Vec<u8>, register: GeneralRegister) {
        let loaded_value = self.get_next_u8(cartridge);

        self.set_register(register, loaded_value);

        if self.debug {
            println!(
                "{:#04x}: Loading {:#06x} to Register {:?}",
                self.program_counter, loaded_value, register
            );
        }

        self.program_counter += 2;
    }

    fn ld_next_16(&mut self, cartridge: &Vec<u8>, register: CombinedRegister) {
        let loaded_value = self.get_next_u16(cartridge);

        self.set_combined_register(register, loaded_value);

        if self.debug {
            println!(
                "{:#04x}: Loading {:#06x} to Register DE",
                self.program_counter, loaded_value
            );
            println!(
                "    D: {:#04x} | E: {:#04x}",
                self.get_register(GeneralRegister::D),
                self.get_register(GeneralRegister::E)
            );
        }

        self.program_counter += 3;
    }

    fn ld_to_stack_pointer(&mut self, cartridge: &Vec<u8>) {
        let loaded_value = self.get_next_u16(cartridge);

        if self.debug {
            println!(
                "{:#06x}: Loading value {:#06x} into Stack Pointer",
                self.program_counter, loaded_value
            );
        }

        self.stack_pointer = loaded_value;

        self.program_counter += 3;
    }

    fn ld_to_hl(&mut self, memory: &mut Vec<u8>, register: GeneralRegister) {
        if self.debug {
            println!(
                "{:#06x}: Loading {:#04x} from Register {:?} to (HL)",
                self.program_counter,
                self.get_register(register),
                register,
            );
        }

        self.set_hl_in_memory(memory, self.get_register(register));

        self.program_counter += 1;
    }

    fn nop(&mut self) {
        if self.debug {
            println!("{:#06x}: NOP", self.program_counter);
        }

        self.program_counter += 1;
    }

    fn set_memory(&mut self, memory: &mut Vec<u8>, location: u16, value: u8) {
        memory[usize::from(location)] = value;
    }

    fn get_memory(&mut self, memory: &mut Vec<u8>, location: u16) -> u8 {
        memory[usize::from(location)]
    }

    fn set_hl_in_memory(&mut self, memory: &mut Vec<u8>, value: u8) {
        self.set_memory(
            memory,
            self.get_combined_register(CombinedRegister::HL),
            value,
        );
    }

    fn get_hl_from_memory(&mut self, memory: &mut Vec<u8>) -> u8 {
        self.get_memory(memory, self.get_combined_register(CombinedRegister::HL))
    }

    pub fn apply_operation(&mut self, op: &u8, cartridge: &Vec<u8>, memory: &mut Vec<u8>) {
        println!(
            "{:#06x}: {:#04x}, A {}, B {}, C {}, D {}, E {}, F {}, H {}, L {}",
            self.program_counter,
            op,
            self.get_register(GeneralRegister::A),
            self.get_register(GeneralRegister::B),
            self.get_register(GeneralRegister::C),
            self.get_register(GeneralRegister::D),
            self.get_register(GeneralRegister::E),
            self.get_register(GeneralRegister::F),
            self.get_register(GeneralRegister::H),
            self.get_register(GeneralRegister::L),
        );

        match op {
            0x00 => {
                self.nop();
            }
            0x01 => {
                self.ld_next_16(cartridge, CombinedRegister::BC);
            }
            0x02 => {
                self.set_memory(
                    memory,
                    self.get_combined_register(CombinedRegister::BC),
                    self.get_register(GeneralRegister::A),
                );

                self.program_counter += 1;
            }
            0x03 => {
                self.inc16(CombinedRegister::BC);
            }
            0x04 => {
                self.inc(GeneralRegister::B);
            }
            0x05 => {
                self.dec(GeneralRegister::B);
            }
            0x06 => {
                self.ld_next_8(cartridge, GeneralRegister::B);
            }
            0x0C => {
                self.inc(GeneralRegister::C);
            }
            0x0D => {
                self.dec(GeneralRegister::C);
            }
            0x0E => {
                self.ld_next_8(cartridge, GeneralRegister::C);
            }
            0x10 => {
                println!("TODO: STOP");
            }
            0x11 => {
                self.ld_next_16(cartridge, CombinedRegister::DE);
            }
            0x13 => {
                self.inc16(CombinedRegister::DE);
            }
            0x14 => {
                self.inc(GeneralRegister::D);
            }
            0x15 => {
                self.dec(GeneralRegister::D);
            }
            0x16 => {
                self.ld_next_8(cartridge, GeneralRegister::D);
            }
            0x18 => {
                self.jr(cartridge, None);
            }
            0x1C => {
                self.inc(GeneralRegister::E);
            }
            0x1D => {
                self.dec(GeneralRegister::E);
            }
            0x1E => {
                self.ld_next_8(cartridge, GeneralRegister::E);
            }
            0x20 => {
                self.jr(cartridge, Some(FlagRegisterValue::NZ));
            }
            0x21 => {
                self.ld_next_16(cartridge, CombinedRegister::HL);
            }
            0x23 => {
                self.inc16(CombinedRegister::HL);
            }
            0x24 => {
                self.inc(GeneralRegister::H);
            }
            0x25 => {
                self.dec(GeneralRegister::H);
            }
            0x28 => {
                self.jr(cartridge, Some(FlagRegisterValue::Z));
            }
            0x2C => {
                self.inc(GeneralRegister::L);
            }
            0x2D => {
                self.dec(GeneralRegister::L);
            }
            0x2E => {
                self.ld_next_8(cartridge, GeneralRegister::L);
            }
            0x2F => {
                self.cpl();
            }
            0x30 => {
                self.jr(cartridge, Some(FlagRegisterValue::NC));
            }
            0x31 => {
                self.ld_to_stack_pointer(cartridge);
            }
            0x38 => {
                self.jr(cartridge, Some(FlagRegisterValue::C));
            }
            0x3C => {
                self.inc(GeneralRegister::A);
            }
            0x3D => {
                self.dec(GeneralRegister::A);
            }
            0x3E => {
                self.ld_next_8(cartridge, GeneralRegister::A);
            }
            0x40 => {
                self.ld(GeneralRegister::B, GeneralRegister::B);
            }
            0x41 => {
                self.ld(GeneralRegister::B, GeneralRegister::C);
            }
            0x42 => {
                self.ld(GeneralRegister::B, GeneralRegister::D);
            }
            0x43 => {
                self.ld(GeneralRegister::B, GeneralRegister::E);
            }
            0x44 => {
                self.ld(GeneralRegister::B, GeneralRegister::H);
            }
            0x45 => {
                self.ld(GeneralRegister::B, GeneralRegister::L);
            }
            0x46 => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::D,
                    self.get_combined_register(CombinedRegister::HL),
                );
            }
            0x47 => {
                self.ld(GeneralRegister::B, GeneralRegister::A);
            }
            0x48 => {
                self.ld(GeneralRegister::C, GeneralRegister::B);
            }
            0x49 => {
                self.ld(GeneralRegister::C, GeneralRegister::C);
            }
            0x4A => {
                self.ld(GeneralRegister::C, GeneralRegister::D);
            }
            0x4B => {
                self.ld(GeneralRegister::C, GeneralRegister::E);
            }
            0x4C => {
                self.ld(GeneralRegister::C, GeneralRegister::H);
            }
            0x4D => {
                self.ld(GeneralRegister::C, GeneralRegister::L);
            }
            0x4E => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::C,
                    self.get_combined_register(CombinedRegister::HL),
                );
            }
            0x4F => {
                self.ld(GeneralRegister::C, GeneralRegister::A);
            }
            0x50 => {
                self.ld(GeneralRegister::D, GeneralRegister::B);
            }
            0x51 => {
                self.ld(GeneralRegister::D, GeneralRegister::C);
            }
            0x52 => {
                self.ld(GeneralRegister::D, GeneralRegister::D);
            }
            0x53 => {
                self.ld(GeneralRegister::D, GeneralRegister::E);
            }
            0x54 => {
                self.ld(GeneralRegister::D, GeneralRegister::H);
            }
            0x55 => {
                self.ld(GeneralRegister::D, GeneralRegister::L);
            }
            0x56 => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::D,
                    self.get_combined_register(CombinedRegister::HL),
                );
            }
            0x57 => {
                self.ld(GeneralRegister::D, GeneralRegister::A);
            }
            0x58 => {
                self.ld(GeneralRegister::E, GeneralRegister::B);
            }
            0x59 => {
                self.ld(GeneralRegister::E, GeneralRegister::C);
            }
            0x5A => {
                self.ld(GeneralRegister::E, GeneralRegister::D);
            }
            0x5B => {
                self.ld(GeneralRegister::E, GeneralRegister::E);
            }
            0x5C => {
                self.ld(GeneralRegister::E, GeneralRegister::H);
            }
            0x5D => {
                self.ld(GeneralRegister::E, GeneralRegister::L);
            }
            0x5E => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::E,
                    self.get_combined_register(CombinedRegister::HL),
                );
            }
            0x5F => {
                self.ld(GeneralRegister::E, GeneralRegister::A);
            }
            0x60 => {
                self.ld(GeneralRegister::H, GeneralRegister::B);
            }
            0x61 => {
                self.ld(GeneralRegister::H, GeneralRegister::C);
            }
            0x62 => {
                self.ld(GeneralRegister::H, GeneralRegister::D);
            }
            0x63 => {
                self.ld(GeneralRegister::H, GeneralRegister::E);
            }
            0x64 => {
                self.ld(GeneralRegister::H, GeneralRegister::H);
            }
            0x65 => {
                self.ld(GeneralRegister::H, GeneralRegister::L);
            }
            0x66 => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::H,
                    self.get_combined_register(CombinedRegister::HL),
                );
            }
            0x67 => {
                self.ld(GeneralRegister::H, GeneralRegister::A);
            }
            0x68 => {
                self.ld(GeneralRegister::L, GeneralRegister::B);
            }
            0x69 => {
                self.ld(GeneralRegister::L, GeneralRegister::C);
            }
            0x6A => {
                self.ld(GeneralRegister::L, GeneralRegister::D);
            }
            0x6B => {
                self.ld(GeneralRegister::L, GeneralRegister::E);
            }
            0x6C => {
                self.ld(GeneralRegister::L, GeneralRegister::H);
            }
            0x6D => {
                self.ld(GeneralRegister::L, GeneralRegister::L);
            }
            0x6E => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::L,
                    self.get_combined_register(CombinedRegister::HL),
                );
            }
            0x6F => {
                self.ld(GeneralRegister::L, GeneralRegister::A);
            }
            0x70 => {
                self.ld_to_hl(memory, GeneralRegister::B);
            }
            0x71 => {
                self.ld_to_hl(memory, GeneralRegister::C);
            }
            0x72 => {
                self.ld_to_hl(memory, GeneralRegister::D);
            }
            0x73 => {
                self.ld_to_hl(memory, GeneralRegister::E);
            }
            0x74 => {
                self.ld_to_hl(memory, GeneralRegister::H);
            }
            0x75 => {
                self.ld_to_hl(memory, GeneralRegister::L);
            }
            0x76 => {
                println!("TODO: HALT");
            }
            0x77 => {
                self.ld_to_hl(memory, GeneralRegister::A);
            }
            0x78 => {
                self.ld(GeneralRegister::A, GeneralRegister::B);
            }
            0x79 => {
                self.ld(GeneralRegister::A, GeneralRegister::C);
            }
            0x7A => {
                self.ld(GeneralRegister::A, GeneralRegister::D);
            }
            0x7B => {
                self.ld(GeneralRegister::A, GeneralRegister::E);
            }
            0x7C => {
                self.ld(GeneralRegister::A, GeneralRegister::H);
            }
            0x7D => {
                self.ld(GeneralRegister::A, GeneralRegister::L);
            }
            0x7E => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::A,
                    self.get_combined_register(CombinedRegister::HL),
                );
            }
            0x7F => {
                self.ld(GeneralRegister::A, GeneralRegister::A);
            }
            0x80 => {
                self.add(GeneralRegister::B);
            }
            0x81 => {
                self.add(GeneralRegister::C);
            }
            0x82 => {
                self.add(GeneralRegister::D);
            }
            0x83 => {
                self.add(GeneralRegister::E);
            }
            0x84 => {
                self.add(GeneralRegister::H);
            }
            0x85 => {
                self.add(GeneralRegister::L);
            }
            0x87 => {
                self.add(GeneralRegister::A);
            }
            0x90 => {
                self.sub(GeneralRegister::B);
            }
            0x91 => {
                self.sub(GeneralRegister::C);
            }
            0x92 => {
                self.sub(GeneralRegister::D);
            }
            0x93 => {
                self.sub(GeneralRegister::E);
            }
            0x94 => {
                self.sub(GeneralRegister::H);
            }
            0x95 => {
                self.sub(GeneralRegister::L);
            }
            0x97 => {
                self.sub(GeneralRegister::A);
            }
            0xC2 => {
                self.jp(cartridge, Some(FlagRegisterValue::NZ));
            }
            0xC3 => {
                self.jp(cartridge, None);
            }
            0xCA => {
                self.jp(cartridge, Some(FlagRegisterValue::Z));
            }
            0xCB => {
                println!("TODO: Prefixes ;)");
            }
            0xCD => {
                self.call(cartridge, memory);
            }
            0xD2 => {
                self.jp(cartridge, Some(FlagRegisterValue::NC));
            }
            0xDA => {
                self.jp(cartridge, Some(FlagRegisterValue::C));
            }
            _ => {
                println!("{:#06x}: Not Implemented", self.program_counter);
                self.program_counter += 1;
            }
        }
    }
}
