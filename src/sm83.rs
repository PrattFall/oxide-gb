use crate::cpu_registers::{CombinedRegister, GeneralRegister, Registers};
use crate::flag_register::{FlagRegister, FlagRegisterValue};
use crate::memory_bank_controller::MemoryBankController;
use crate::utils::{add_should_half_carry, sub_should_half_carry, u16_to_u8s, u8s_to_u16};
use std::num::Wrapping;

// const CLOCK_MHZ: f64 = 4.194304;

#[derive(Default)]
pub struct SharpSM83 {
    pub program_counter: u16,
    pub stack_pointer: u16,
    pub registers: Registers,
    pub debug: bool,
}

impl SharpSM83 {
    fn not_implemented(&mut self, message: &str) {
        if self.debug {
            println!("TODO: {}", message);
        }

        self.program_counter += 1;
    }

    fn set_register_from_memory<T: MemoryBankController + ?Sized>(
        &mut self,
        memory: &mut T,
        register: GeneralRegister,
        location: u16,
    ) {
        let value = memory.read_memory(usize::from(location));

        if self.debug {
            println!(
                "{:#06x}: Setting value {:#04x} to register {:?} from location {:#06x}",
                self.program_counter, value, register, location
            );
        }

        self.registers.set(register, value);
        self.program_counter += 1;
    }

    fn inc(&mut self, register: GeneralRegister) {
        self.unset_flag(FlagRegisterValue::N);

        if add_should_half_carry(self.registers.get(register), 1) {
            self.set_flag(FlagRegisterValue::H);
        }

        let register_value = self.registers.get(register);

        if register_value == 0xFF {
            self.registers.set(register, 0);
        } else {
            self.registers.set(register, register_value + 1);
        }

        if self.debug {
            println!(
                "{:#06x}: Register {:?} Increased to {:#04x}",
                self.program_counter,
                register,
                self.registers.get(register)
            );
        }

        self.program_counter += 1;
    }

    fn inc16(&mut self, register: CombinedRegister) {
        let register_value = self.registers.get_combined(register);

        self.registers
            .set_combined(register, register_value.wrapping_add(1));

        if self.debug {
            println!(
                "{:#06x}: Register {:?} Increased to {:#06x}",
                self.program_counter,
                register,
                self.registers.get_combined(register)
            );
        }

        self.program_counter += 1;
    }

    fn ld(&mut self, a: GeneralRegister, b: GeneralRegister) {
        if self.debug {
            println!(
                "{:#06x}: Loading {:#04x} from Register {:?} to Register {:?}",
                self.program_counter,
                self.registers.get(b),
                b,
                a
            );
        }

        self.registers.set(a, self.registers.get(b));

        self.program_counter += 1;
    }

    fn is_flag_set(&mut self, flag: FlagRegisterValue) -> bool {
        self.registers.get(GeneralRegister::F).contains_flag(flag)
    }

    fn call<T: MemoryBankController + ?Sized>(&mut self, memory: &mut T) {
        let call_location = self.get_next_u16(memory);
        let [left, right] = u16_to_u8s(self.program_counter);

        self.stack_pointer = self.stack_pointer.wrapping_sub(2);
        memory.write_memory(usize::from(self.stack_pointer), left);
        memory.write_memory(usize::from(self.stack_pointer + 1), right);
        self.program_counter = call_location;
    }

    fn cpl(&mut self) {
        println!("{:#06x}: CPL", self.program_counter);

        self.registers.set(
            GeneralRegister::A,
            self.registers.get(GeneralRegister::A) ^ 0xFF,
        );

        self.set_flag(FlagRegisterValue::N);
        self.set_flag(FlagRegisterValue::H);

        self.program_counter += 1;
    }

    fn jp_inner<T: MemoryBankController + ?Sized>(&mut self, memory: &mut T) {
        let jump_location = self.get_next_u16(memory);

        if self.debug {
            println!(
                "{:#06x}: Jumping to {:#06x}",
                self.program_counter, jump_location
            );
        }

        self.program_counter = jump_location;
    }

    fn jp<T: MemoryBankController + ?Sized>(
        &mut self,
        memory: &mut T,
        flag: Option<FlagRegisterValue>,
    ) {
        match flag {
            Some(f) if self.is_flag_set(f) => {
                self.jp_inner(memory);
            }
            Some(f) => {
                if self.debug {
                    println!(
                        "Flag {:?} not set. Found {:#06x}",
                        f,
                        self.registers.get(GeneralRegister::F)
                    );
                }

                self.program_counter += 1;
            }
            None => {
                self.jp_inner(memory);
            }
        }
    }

    fn jr_inner<T: MemoryBankController + ?Sized>(&mut self, memory: &mut T) {
        // Jump to program_counter + u8
        let relative_location = u16::from(self.get_next_u8(memory));

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

    fn jr<T: MemoryBankController + ?Sized>(
        &mut self,
        memory: &mut T,
        flag: Option<FlagRegisterValue>,
    ) {
        match flag {
            Some(f) if self.is_flag_set(f) => {
                self.jr_inner(memory);
            }
            Some(f) => {
                if self.debug {
                    println!(
                        "Flag {:?} not set. Found {:#06x}",
                        f,
                        self.registers.get(GeneralRegister::F)
                    );
                }

                self.program_counter += 1;
            }
            None => {
                self.jr_inner(memory);
            }
        }
    }

    fn set_flag(&mut self, flag: FlagRegisterValue) {
        self.registers.set(
            GeneralRegister::F,
            self.registers.get(GeneralRegister::F).set_flag(flag),
        );
    }

    fn unset_flag(&mut self, flag: FlagRegisterValue) {
        self.registers.set(
            GeneralRegister::F,
            self.registers.get(GeneralRegister::F).unset_flag(flag),
        );
    }

    fn dec(&mut self, register: GeneralRegister) {
        self.set_flag(FlagRegisterValue::N);

        if sub_should_half_carry(self.registers.get(register), 1) {
            self.set_flag(FlagRegisterValue::H);
        }

        let register_value = self.registers.get(register);

        self.registers.set(register, register_value.wrapping_sub(1));

        if self.debug {
            println!(
                "{:#06x}: Register {:?} Decreased to {}",
                self.program_counter,
                register,
                self.registers.get(register)
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
                self.registers.get(register),
                self.registers.get(GeneralRegister::A)
            );
        }

        let a_val = self.registers.get(GeneralRegister::A);
        let r_val = self.registers.get(register);

        self.set_flag(FlagRegisterValue::N);

        if Wrapping(r_val) + Wrapping(a_val) > Wrapping(0xFF) {
            self.registers
                .set(GeneralRegister::A, r_val.wrapping_add(a_val));
            self.set_flag(FlagRegisterValue::C);
        } else {
            self.registers
                .set(GeneralRegister::A, a_val.wrapping_sub(r_val));
        }

        if sub_should_half_carry(a_val, self.registers.get(GeneralRegister::A)) {
            self.set_flag(FlagRegisterValue::H);
        }

        if self.registers.get(GeneralRegister::A) == 0 {
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
                self.registers.get(register),
                self.registers.get(GeneralRegister::A)
            );
        }

        let a_val = self.registers.get(GeneralRegister::A);
        let r_val = self.registers.get(register);

        self.set_flag(FlagRegisterValue::N);

        if r_val > a_val {
            self.registers
                .set(GeneralRegister::A, a_val.wrapping_sub(r_val));
            self.set_flag(FlagRegisterValue::C);
        } else {
            self.registers
                .set(GeneralRegister::A, a_val.wrapping_sub(r_val));
        }

        if sub_should_half_carry(a_val, self.registers.get(GeneralRegister::A)) {
            self.set_flag(FlagRegisterValue::H);
        }

        if self.registers.get(GeneralRegister::A) == 0 {
            self.set_flag(FlagRegisterValue::Z);
        }

        self.program_counter += 1;
    }

    fn get_next_u8<T: MemoryBankController + ?Sized>(&mut self, memory: &mut T) -> u8 {
        memory.read_memory(usize::from(self.program_counter + 1))
    }

    fn get_next_u16<T: MemoryBankController + ?Sized>(&mut self, memory: &mut T) -> u16 {
        u8s_to_u16(
            memory.read_memory(usize::from(self.program_counter + 1)),
            memory.read_memory(usize::from(self.program_counter + 2)),
        )
    }

    fn ld_next_8<T: MemoryBankController + ?Sized>(
        &mut self,
        memory: &mut T,
        register: GeneralRegister,
    ) {
        let loaded_value = self.get_next_u8(memory);

        self.registers.set(register, loaded_value);

        if self.debug {
            println!(
                "{:#04x}: Loading {:#06x} to Register {:?}",
                self.program_counter, loaded_value, register
            );
        }

        self.program_counter += 2;
    }

    fn ld_next_16<T: MemoryBankController + ?Sized>(
        &mut self,
        memory: &mut T,
        register: CombinedRegister,
    ) {
        let loaded_value = self.get_next_u16(memory);

        self.registers.set_combined(register, loaded_value);

        if self.debug {
            println!(
                "{:#04x}: Loading {:#06x} to Register {:?}",
                self.program_counter, loaded_value, register
            );
        }

        self.program_counter += 3;
    }

    fn ld_to_stack_pointer<T: MemoryBankController + ?Sized>(&mut self, memory: &mut T) {
        let loaded_value = self.get_next_u16(memory);

        if self.debug {
            println!(
                "{:#06x}: Loading value {:#06x} into Stack Pointer",
                self.program_counter, loaded_value
            );
        }

        self.stack_pointer = loaded_value;

        self.program_counter += 3;
    }

    fn ld_to_hl<T: MemoryBankController + ?Sized>(
        &mut self,
        memory: &mut T,
        register: GeneralRegister,
    ) {
        if self.debug {
            println!(
                "{:#06x}: Loading {:#04x} from Register {:?} to (HL)",
                self.program_counter,
                self.registers.get(register),
                register,
            );
        }

        self.set_hl_in_memory(memory, self.registers.get(register));

        self.program_counter += 1;
    }

    fn nop(&mut self) {
        if self.debug {
            println!("{:#06x}: NOP", self.program_counter);
        }

        self.program_counter += 1;
    }

    fn set_hl_in_memory<T: MemoryBankController + ?Sized>(&mut self, memory: &mut T, value: u8) {
        memory.write_memory(
            usize::from(self.registers.get_combined(CombinedRegister::HL)),
            value,
        );
    }

    fn get_hl_from_memory<T: MemoryBankController + ?Sized>(&mut self, memory: &mut T) -> u8 {
        memory.read_memory(usize::from(
            self.registers.get_combined(CombinedRegister::HL),
        ))
    }

    fn display_current_registers(&self, op: u8) {
        println!(
            "{:#06x}: {:#04x}, A {:#04x}, B {:#04x}, C {:#04x}, D {:#04x}, E {:#04x}, F {:#04x}, H {:#04x}, L {:#04x}",
            self.program_counter,
            op,
            self.registers.get(GeneralRegister::A),
            self.registers.get(GeneralRegister::B),
            self.registers.get(GeneralRegister::C),
            self.registers.get(GeneralRegister::D),
            self.registers.get(GeneralRegister::E),
            self.registers.get(GeneralRegister::F),
            self.registers.get(GeneralRegister::H),
            self.registers.get(GeneralRegister::L),
        );
    }

    pub fn apply_operation<T: MemoryBankController + ?Sized>(&mut self, memory: &mut T) {
        let op = memory.read_memory(usize::from(self.program_counter));

        if self.debug {
            self.display_current_registers(op);
        }

        match op {
            0x00 => self.nop(),
            0x01 => self.ld_next_16(memory, CombinedRegister::BC),
            0x02 => {
                memory.write_memory(
                    usize::from(self.registers.get_combined(CombinedRegister::BC)),
                    self.registers.get(GeneralRegister::A),
                );

                self.program_counter += 1;
            }
            0x03 => self.inc16(CombinedRegister::BC),
            0x04 => self.inc(GeneralRegister::B),
            0x05 => self.dec(GeneralRegister::B),
            0x06 => self.ld_next_8(memory, GeneralRegister::B),
            0x07 => self.not_implemented("RLCA"),
            0x08 => self.not_implemented("LD (u16), Stack Pointer"),
            0x09 => self.not_implemented("ADD HL, BC"),
            0x0A => self.not_implemented("TODO: ADD HL, BC"),
            0x0B => self.not_implemented("LD A, (BC)"),
            0x0C => self.inc(GeneralRegister::C),
            0x0D => self.dec(GeneralRegister::C),
            0x0E => self.ld_next_8(memory, GeneralRegister::C),
            0x0F => self.not_implemented("RRCA"),
            0x10 => self.not_implemented("STOP"),
            0x11 => self.ld_next_16(memory, CombinedRegister::DE),
            0x12 => self.not_implemented("LD (DE), A"),
            0x13 => self.inc16(CombinedRegister::DE),
            0x14 => self.inc(GeneralRegister::D),
            0x15 => self.dec(GeneralRegister::D),
            0x16 => self.ld_next_8(memory, GeneralRegister::D),
            0x17 => self.not_implemented("RLA"),
            0x18 => self.jr(memory, None),
            0x19 => self.not_implemented("ADD HL, DE"),
            0x1A => self.not_implemented("LD A, (DE)"),
            0x1B => self.not_implemented("DEC DE"),
            0x1C => self.inc(GeneralRegister::E),
            0x1D => self.dec(GeneralRegister::E),
            0x1E => self.ld_next_8(memory, GeneralRegister::E),
            0x1F => self.not_implemented("RRA"),
            0x20 => self.jr(memory, Some(FlagRegisterValue::NZ)),
            0x21 => self.ld_next_16(memory, CombinedRegister::HL),
            0x22 => self.not_implemented("LD (HL+), A"),
            0x23 => self.inc16(CombinedRegister::HL),
            0x24 => self.inc(GeneralRegister::H),
            0x25 => self.dec(GeneralRegister::H),
            0x26 => self.not_implemented("DAA"),
            0x27 => self.not_implemented("JR Z, u8"),
            0x28 => self.jr(memory, Some(FlagRegisterValue::Z)),
            0x29 => self.not_implemented("ADD HL, HL"),
            0x2A => self.not_implemented("LD A, (HL+)"),
            0x2B => self.not_implemented("DEC HL"),
            0x2C => self.inc(GeneralRegister::L),
            0x2D => self.dec(GeneralRegister::L),
            0x2E => self.ld_next_8(memory, GeneralRegister::L),
            0x2F => self.cpl(),
            0x30 => self.jr(memory, Some(FlagRegisterValue::NC)),
            0x31 => self.ld_to_stack_pointer(memory),
            0x32 => self.not_implemented("LD (HL-), A"),
            0x33 => self.not_implemented("INC SP"),
            0x34 => self.not_implemented("INC (HL) 1"),
            0x35 => self.not_implemented("DEC (HL) 1"),
            0x36 => self.not_implemented("LD (HL), u8"),
            0x37 => self.not_implemented("SCF"),
            0x38 => self.jr(memory, Some(FlagRegisterValue::C)),
            0x39 => self.not_implemented("ADD HL, SP"),
            0x3A => self.not_implemented("LD A, (HL-)"),
            0x3B => self.not_implemented("DEC SP"),
            0x3C => self.inc(GeneralRegister::A),
            0x3D => self.dec(GeneralRegister::A),
            0x3E => self.ld_next_8(memory, GeneralRegister::A),
            0x3F => self.not_implemented("CCF"),
            0x40 => self.ld(GeneralRegister::B, GeneralRegister::B),
            0x41 => self.ld(GeneralRegister::B, GeneralRegister::C),
            0x42 => self.ld(GeneralRegister::B, GeneralRegister::D),
            0x43 => self.ld(GeneralRegister::B, GeneralRegister::E),
            0x44 => self.ld(GeneralRegister::B, GeneralRegister::H),
            0x45 => self.ld(GeneralRegister::B, GeneralRegister::L),
            0x46 => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::D,
                    self.registers.get_combined(CombinedRegister::HL),
                );
            }
            0x47 => self.ld(GeneralRegister::B, GeneralRegister::A),
            0x48 => self.ld(GeneralRegister::C, GeneralRegister::B),
            0x49 => self.ld(GeneralRegister::C, GeneralRegister::C),
            0x4A => self.ld(GeneralRegister::C, GeneralRegister::D),
            0x4B => self.ld(GeneralRegister::C, GeneralRegister::E),
            0x4C => self.ld(GeneralRegister::C, GeneralRegister::H),
            0x4D => self.ld(GeneralRegister::C, GeneralRegister::L),
            0x4E => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::C,
                    self.registers.get_combined(CombinedRegister::HL),
                );
            }
            0x4F => self.ld(GeneralRegister::C, GeneralRegister::A),
            0x50 => self.ld(GeneralRegister::D, GeneralRegister::B),
            0x51 => self.ld(GeneralRegister::D, GeneralRegister::C),
            0x52 => self.ld(GeneralRegister::D, GeneralRegister::D),
            0x53 => self.ld(GeneralRegister::D, GeneralRegister::E),
            0x54 => self.ld(GeneralRegister::D, GeneralRegister::H),
            0x55 => self.ld(GeneralRegister::D, GeneralRegister::L),
            0x56 => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::D,
                    self.registers.get_combined(CombinedRegister::HL),
                );
            }
            0x57 => self.ld(GeneralRegister::D, GeneralRegister::A),
            0x58 => self.ld(GeneralRegister::E, GeneralRegister::B),
            0x59 => self.ld(GeneralRegister::E, GeneralRegister::C),
            0x5A => self.ld(GeneralRegister::E, GeneralRegister::D),
            0x5B => self.ld(GeneralRegister::E, GeneralRegister::E),
            0x5C => self.ld(GeneralRegister::E, GeneralRegister::H),
            0x5D => self.ld(GeneralRegister::E, GeneralRegister::L),
            0x5E => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::E,
                    self.registers.get_combined(CombinedRegister::HL),
                );
            }
            0x5F => self.ld(GeneralRegister::E, GeneralRegister::A),
            0x60 => self.ld(GeneralRegister::H, GeneralRegister::B),
            0x61 => self.ld(GeneralRegister::H, GeneralRegister::C),
            0x62 => self.ld(GeneralRegister::H, GeneralRegister::D),
            0x63 => self.ld(GeneralRegister::H, GeneralRegister::E),
            0x64 => self.ld(GeneralRegister::H, GeneralRegister::H),
            0x65 => self.ld(GeneralRegister::H, GeneralRegister::L),
            0x66 => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::H,
                    self.registers.get_combined(CombinedRegister::HL),
                );
            }
            0x67 => self.ld(GeneralRegister::H, GeneralRegister::A),
            0x68 => self.ld(GeneralRegister::L, GeneralRegister::B),
            0x69 => self.ld(GeneralRegister::L, GeneralRegister::C),
            0x6A => self.ld(GeneralRegister::L, GeneralRegister::D),
            0x6B => self.ld(GeneralRegister::L, GeneralRegister::E),
            0x6C => self.ld(GeneralRegister::L, GeneralRegister::H),
            0x6D => self.ld(GeneralRegister::L, GeneralRegister::L),
            0x6E => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::L,
                    self.registers.get_combined(CombinedRegister::HL),
                );
            }
            0x6F => self.ld(GeneralRegister::L, GeneralRegister::A),
            0x70 => self.ld_to_hl(memory, GeneralRegister::B),
            0x71 => self.ld_to_hl(memory, GeneralRegister::C),
            0x72 => self.ld_to_hl(memory, GeneralRegister::D),
            0x73 => self.ld_to_hl(memory, GeneralRegister::E),
            0x74 => self.ld_to_hl(memory, GeneralRegister::H),
            0x75 => self.ld_to_hl(memory, GeneralRegister::L),
            0x76 => self.not_implemented("HALT"),
            0x77 => self.ld_to_hl(memory, GeneralRegister::A),
            0x78 => self.ld(GeneralRegister::A, GeneralRegister::B),
            0x79 => self.ld(GeneralRegister::A, GeneralRegister::C),
            0x7A => self.ld(GeneralRegister::A, GeneralRegister::D),
            0x7B => self.ld(GeneralRegister::A, GeneralRegister::E),
            0x7C => self.ld(GeneralRegister::A, GeneralRegister::H),
            0x7D => self.ld(GeneralRegister::A, GeneralRegister::L),
            0x7E => {
                self.set_register_from_memory(
                    memory,
                    GeneralRegister::A,
                    self.registers.get_combined(CombinedRegister::HL),
                );
            }
            0x7F => self.ld(GeneralRegister::A, GeneralRegister::A),
            0x80 => self.add(GeneralRegister::B),
            0x81 => self.add(GeneralRegister::C),
            0x82 => self.add(GeneralRegister::D),
            0x83 => self.add(GeneralRegister::E),
            0x84 => self.add(GeneralRegister::H),
            0x85 => self.add(GeneralRegister::L),
            0x86 => self.not_implemented("ADD A, (HL)"),
            0x87 => self.add(GeneralRegister::A),
            0x88 => self.not_implemented("ADC A, B"),
            0x89 => self.not_implemented("ADC A, C"),
            0x8A => self.not_implemented("ADC A, D"),
            0x8B => self.not_implemented("ADC A, E"),
            0x8C => self.not_implemented("ADC A, H"),
            0x8D => self.not_implemented("ADC A, L"),
            0x8E => self.not_implemented("ADC A, (HL)"),
            0x8F => self.not_implemented("ADC A, A"),
            0x90 => self.sub(GeneralRegister::B),
            0x91 => self.sub(GeneralRegister::C),
            0x92 => self.sub(GeneralRegister::D),
            0x93 => self.sub(GeneralRegister::E),
            0x94 => self.sub(GeneralRegister::H),
            0x95 => self.sub(GeneralRegister::L),
            0x96 => self.not_implemented("SUB A, (HL)"),
            0x97 => self.sub(GeneralRegister::A),
            0x98 => self.not_implemented("SBC A, B"),
            0x99 => self.not_implemented("SBC A, C"),
            0x9A => self.not_implemented("SBC A, D"),
            0x9B => self.not_implemented("SBC A, E"),
            0x9C => self.not_implemented("SBC A, H"),
            0x9D => self.not_implemented("SBC A, L"),
            0x9E => self.not_implemented("SBC A, (HL)"),
            0x9F => self.not_implemented("SBC A, A"),
            0xA0 => self.not_implemented("AND A, B"),
            0xA1 => self.not_implemented("AND A, C"),
            0xA2 => self.not_implemented("AND A, D"),
            0xA3 => self.not_implemented("AND A, E"),
            0xA4 => self.not_implemented("AND A, H"),
            0xA5 => self.not_implemented("AND A, L"),
            0xA6 => self.not_implemented("AND A, (HL)"),
            0xA7 => self.not_implemented("AND A, A"),
            0xA8 => self.not_implemented("XOR A, B"),
            0xA9 => self.not_implemented("XOR A, C"),
            0xAA => self.not_implemented("XOR A, D"),
            0xAB => self.not_implemented("XOR A, E"),
            0xAC => self.not_implemented("XOR A, H"),
            0xAD => self.not_implemented("XOR A, L"),
            0xAE => self.not_implemented("XOR A, (HL)"),
            0xAF => self.not_implemented("XOR A, A"),
            0xB0 => self.not_implemented("OR A, B"),
            0xB1 => self.not_implemented("OR A, C"),
            0xB2 => self.not_implemented("OR A, D"),
            0xB3 => self.not_implemented("OR A, E"),
            0xB4 => self.not_implemented("OR A, H"),
            0xB5 => self.not_implemented("OR A, L"),
            0xB6 => self.not_implemented("OR A, (HL)"),
            0xB7 => self.not_implemented("OR A, A"),
            0xB8 => self.not_implemented("CP A, B"),
            0xB9 => self.not_implemented("CP A, C"),
            0xBA => self.not_implemented("CP A, D"),
            0xBB => self.not_implemented("CP A, E"),
            0xBC => self.not_implemented("CP A, H"),
            0xBD => self.not_implemented("CP A, L"),
            0xBE => self.not_implemented("CP A, (HL)"),
            0xBF => self.not_implemented("CP A, A"),
            0xC0 => self.not_implemented("RET NZ"),
            0xC1 => self.not_implemented("POP BC"),
            0xC2 => self.jp(memory, Some(FlagRegisterValue::NZ)),
            0xC3 => self.jp(memory, None),
            0xC4 => self.not_implemented("CALL NZ, u16"),
            0xC5 => self.not_implemented("PUSH BC"),
            0xC6 => self.not_implemented("ADD A, u8"),
            0xC7 => self.not_implemented("RST 00h"),
            0xC8 => self.not_implemented("RET Z"),
            0xC9 => self.not_implemented("RET"),
            0xCA => self.jp(memory, Some(FlagRegisterValue::Z)),
            0xCB => self.not_implemented("Prefixes ,)"),
            0xCC => self.not_implemented("CALL Z, u16"),
            0xCD => self.call(memory),
            0xCE => self.not_implemented("ADC A, u8"),
            0xCF => self.not_implemented("RST 08h"),
            0xD0 => self.not_implemented("RET NC"),
            0xD1 => self.not_implemented("POP DE"),
            0xD2 => self.jp(memory, Some(FlagRegisterValue::NC)),
            0xD3 => (),
            0xD4 => self.not_implemented("CALL NC, u16"),
            0xD5 => self.not_implemented("PUSH DE"),
            0xD6 => self.not_implemented("SUB A, u8"),
            0xD7 => self.not_implemented("RST 10h"),
            0xD8 => self.not_implemented("RET C"),
            0xD9 => self.not_implemented("RETI"),
            0xDA => self.jp(memory, Some(FlagRegisterValue::C)),
            0xDB => (),
            0xDC => self.not_implemented("CALL C, u16"),
            0xDD => (),
            0xDE => self.not_implemented("SBC A, u8"),
            0xDF => self.not_implemented("RST 18h"),
            0xE0 => self.not_implemented("ld (FF00+u8), A"),
            0xE1 => self.not_implemented("POP HL"),
            0xE2 => self.not_implemented("LD (FF00+C), A"),
            0xE3 => (),
            0xE4 => (),
            0xE5 => self.not_implemented("PUSH HL"),
            0xE6 => self.not_implemented("AND A, u8"),
            0xE7 => self.not_implemented("RST 20h"),
            0xE8 => self.not_implemented("ADD SP, i8"),
            0xE9 => self.not_implemented("JP HL"),
            0xEA => self.not_implemented("LD (u16), A"),
            0xEB => (),
            0xEC => (),
            0xED => (),
            0xEE => self.not_implemented("XOR A, u8"),
            0xEF => self.not_implemented("RST 28h"),
            0xF0 => self.not_implemented("LD A, (FF00+u8)"),
            0xF1 => self.not_implemented("POP AF"),
            0xF2 => self.not_implemented("LD A, (FF00+C)"),
            0xF3 => self.not_implemented("DI"),
            0xF4 => (),
            0xF5 => self.not_implemented("PUSH AF"),
            0xF6 => self.not_implemented("OR A, u8"),
            0xF7 => self.not_implemented("RST 30h"),
            0xF8 => self.not_implemented("LD HL, SP+i"),
            0xF9 => self.not_implemented("LD SP, HL"),
            0xFA => self.not_implemented("LD A, (u16)"),
            0xFB => self.not_implemented("EI"),
            0xFC => (),
            0xFD => (),
            0xFE => self.not_implemented("CP A, u8"),
            0xFF => self.not_implemented("RST 38h"),
            _ => {
                println!("{:#06x}: Not Implemented", self.program_counter);
                self.program_counter += 1;
            }
        }
    }
}
