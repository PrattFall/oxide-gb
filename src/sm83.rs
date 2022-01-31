use crate::cpu_registers::{CombinedRegister, GeneralRegister, Registers};
use crate::flag_register::FlagRegisterValue;
use crate::memory_bank_controller::MemoryBankController;
use crate::utils::{
    add_16_should_half_carry, add_should_half_carry, bit_set_at, reset_bit, sub_should_half_carry,
    u16_to_u8s, Mode, MODE,
};

// const CLOCK_MHZ: f64 = 4.194304;

#[derive(Default)]
pub struct SharpSM83 {
    pub program_counter: u16,
    pub stack_pointer: u16,
    pub registers: Registers,
}

pub enum MemoryOffset {
    Plus,
    Minus,
}

impl SharpSM83 {
    fn nop(&mut self) {
        if MODE == Mode::Debug {
            println!("{:#06x}: NOP", self.program_counter);
        }

        self.program_counter += 1;
    }

    fn not_implemented(&mut self, message: &str) {
        if MODE == Mode::Debug {
            println!("{:#06x}: ******** TODO: {}", self.program_counter, message);
        }

        self.program_counter += 1;
    }

    fn cp_val(&mut self, value: u8) {
        let a_val = self.registers.get(GeneralRegister::A);

        if a_val.wrapping_sub(value) == 0 {
            self.registers.set_flag(FlagRegisterValue::Z);
        }
    }

    fn cp_memory<T: MemoryBankController + ?Sized>(&mut self, mbc: &mut T, location: usize) {
        let value = mbc.read_memory(location);

        if MODE == Mode::Debug {
            let a_val = self.registers.get(GeneralRegister::A);
            println!(
                "{:#06x}: Comparing Register A ({:#04x}) with value at location {:#06x} ({:#04x})",
                self.program_counter, a_val, location, value
            );
        }

        self.cp_val(value);

        self.program_counter += 1;
    }

    fn cp_u8<T: MemoryBankController + ?Sized>(&mut self, mbc: &mut T) {
        let value = mbc.get_next_u8(self.program_counter.into());

        if MODE == Mode::Debug {
            let location = usize::from(self.program_counter + 1);
            let a_val = self.registers.get(GeneralRegister::A);
            println!(
                "{:#06x}: Comparing Register A ({:#04x}) with value at location {:#06x} ({:#04x})",
                self.program_counter, a_val, location, value
            );
        }

        self.cp_val(value);

        self.program_counter += 2;
    }

    fn cp_register(&mut self, register: GeneralRegister) {
        let value = self.registers.get(register);

        if MODE == Mode::Debug {
            let a_val = self.registers.get(GeneralRegister::A);
            println!(
                "{:#06x}: Comparing Register A ({:#04x}) with Register {:?} ({:#04x})",
                self.program_counter, a_val, register, value
            );
        }

        self.cp_val(value);

        self.program_counter += 1;
    }

    fn inc(&mut self, register: GeneralRegister) {
        let register_value = self.registers.get(register);

        self.registers.unset_flag(FlagRegisterValue::N);

        self.registers
            .set_half_carry(add_should_half_carry(self.registers.get(register), 1));

        self.registers.set(register, register_value.wrapping_add(1));

        if MODE == Mode::Debug {
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
        let result = register_value.wrapping_add(1);

        self.registers.set_combined(register, result);

        if MODE == Mode::Debug {
            println!(
                "{:#06x}: Register {:?} Increased to {:#06x}",
                self.program_counter, register, result
            );
        }

        self.program_counter += 1;
    }

    fn ld(&mut self, a: GeneralRegister, b: GeneralRegister) {
        if MODE == Mode::Debug {
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

    fn call<T: MemoryBankController + ?Sized>(&mut self, mbc: &mut T) {
        let call_location = mbc.get_next_u16(self.program_counter.into());
        let [left, right] = u16_to_u8s(self.program_counter);

        self.stack_pointer = self.stack_pointer.wrapping_sub(2);
        mbc.write_memory(self.stack_pointer.into(), left);
        mbc.write_memory((self.stack_pointer + 1).into(), right);
        self.program_counter = call_location;
    }

    fn cpl(&mut self) {
        println!("{:#06x}: CPL", self.program_counter);

        self.registers.set(
            GeneralRegister::A,
            self.registers.get(GeneralRegister::A) ^ 0xFF,
        );

        self.registers.set_flag(FlagRegisterValue::N);
        self.registers.set_flag(FlagRegisterValue::H);

        self.program_counter += 1;
    }

    fn flag_not_found(&mut self, f: FlagRegisterValue) {
        if MODE == Mode::Debug {
            println!(
                "    Flag {:?} not set. Found {:b}",
                f,
                self.registers.get(GeneralRegister::F)
            );
        }

        self.program_counter += 1;
    }

    fn jp<T: MemoryBankController + ?Sized>(
        &mut self,
        mbc: &mut T,
        flag: Option<FlagRegisterValue>,
    ) {
        if MODE == Mode::Debug {
            let location = mbc.get_next_u16(self.program_counter.into());
            println!(
                "{:#06x}: Attempting to jump to {:#06x}",
                self.program_counter, location
            );
        }

        match flag {
            Some(f) if !self.registers.is_flag_set(f) => {
                self.flag_not_found(f);
            }
            _ => {
                let jump_location = mbc.get_next_u16(self.program_counter.into());

                self.program_counter = jump_location;
            }
        }
    }

    fn jr<T: MemoryBankController + ?Sized>(
        &mut self,
        mbc: &mut T,
        flag: Option<FlagRegisterValue>,
    ) {
        if MODE == Mode::Debug {
            let relative_location = u16::from(mbc.get_next_u8(self.program_counter.into()));
            println!(
                "{:#06x}: Attempting to Jump {:#06x} ops to {:#06x}",
                self.program_counter,
                relative_location,
                self.program_counter + relative_location
            );
        }

        match flag {
            Some(f) if !self.registers.is_flag_set(f) => {
                self.flag_not_found(f);
            }
            _ => {
                // Jump to program_counter + u8
                let relative_location = u16::from(mbc.get_next_u8(self.program_counter.into()));

                self.program_counter += relative_location;
            }
        }
    }

    fn dec(&mut self, register: GeneralRegister) {
        let register_value = self.registers.get(register);
        let result = register_value.wrapping_sub(1);

        if MODE == Mode::Debug {
            println!(
                "{:#06x}: Register {:?} Decreased to {}",
                self.program_counter,
                register,
                self.registers.get(register)
            );
        }

        self.registers.set_flag(FlagRegisterValue::N);

        self.registers
            .set_half_carry(sub_should_half_carry(register_value, 1));

        self.registers.set(register, result);

        self.program_counter += 1;
    }

    fn add(&mut self, value: u8) {
        let a_value = self.registers.get(GeneralRegister::A);
        let result = a_value.wrapping_add(value);

        self.registers.set(GeneralRegister::A, result);

        self.registers.unset_flag(FlagRegisterValue::N);

        if 0xff - a_value < value {
            self.registers.set_flag(FlagRegisterValue::C);
        }

        self.registers
            .set_half_carry(add_should_half_carry(a_value, value));

        if result == 0 {
            self.registers.set_flag(FlagRegisterValue::Z);
        }
    }

    fn add_register(&mut self, register: GeneralRegister) {
        let r_val = self.registers.get(register);

        if MODE == Mode::Debug {
            let a_val = self.registers.get(GeneralRegister::A);
            println!(
                "{:#06x}: Adding Register {:?}'s value ({:#04x}) to Register A ({:#04x})",
                self.program_counter, register, r_val, a_val
            );
        }

        self.add(r_val);

        self.program_counter += 1;
    }

    fn add_memory<T: MemoryBankController + ?Sized>(&mut self, mbc: &T) {
        let value = mbc.read_memory(self.registers.get_combined(CombinedRegister::HL).into());

        if MODE == Mode::Debug {
            let a_val = self.registers.get(GeneralRegister::A);
            println!(
                "{:#06x}: Adding memory location (HL)'s value ({:#04x}) to Register A ({:#04x})",
                self.program_counter, value, a_val
            );
        }

        self.add(value);

        self.program_counter += 1;
    }

    fn add_16(&mut self, value: u16) {
        let hl_val = self.registers.get_combined(CombinedRegister::HL);
        let result = hl_val.wrapping_add(value);

        self.registers.set_combined(CombinedRegister::HL, result);

        self.registers.unset_flag(FlagRegisterValue::N);

        self.registers
            .set_half_carry(add_16_should_half_carry(hl_val, value));

        if 0xffff - hl_val < value {
            self.registers.set_flag(FlagRegisterValue::C);
        }
    }

    fn add_combined_register(&mut self, register: CombinedRegister) {
        let register_value = self.registers.get_combined(register);

        if MODE == Mode::Debug {
            let hl_value = self.registers.get_combined(CombinedRegister::HL);
            println!(
                "{:#06x}: Adding Register {:?}'s value ({:#06x}) to Register HL ({:#06x})",
                self.program_counter, register, register_value, hl_value
            );
        }

        self.add_16(register_value);

        self.program_counter += 1;
    }

    fn add_sp(&mut self) {
        if MODE == Mode::Debug {
            let hl_val = self.registers.get_combined(CombinedRegister::HL);
            println!(
                "{:#06x}: Adding Stack Pointer value ({:#06x}) to Register HL ({:#06x})",
                self.program_counter, self.stack_pointer, hl_val
            );
        }

        self.add_16(self.stack_pointer);

        self.program_counter += 1;
    }

    fn sub(&mut self, register: GeneralRegister) {
        let a_val = self.registers.get(GeneralRegister::A);
        let r_val = self.registers.get(register);
        let result = a_val.wrapping_sub(r_val);

        self.registers.set(GeneralRegister::A, result);

        if MODE == Mode::Debug {
            println!(
                "{:#06x}: Subtracting Register {:?}'s value ({:#04x}) from Register A ({:#04x})",
                self.program_counter,
                register,
                self.registers.get(register),
                self.registers.get(GeneralRegister::A)
            );
        }

        self.registers.set_flag(FlagRegisterValue::N);

        if r_val > a_val {
            self.registers.set_flag(FlagRegisterValue::C);
        }

        self.registers
            .set_half_carry(sub_should_half_carry(a_val, r_val));

        if result == 0 {
            self.registers.set_flag(FlagRegisterValue::Z);
        }

        self.program_counter += 1;
    }

    fn ld_next_8<T: MemoryBankController + ?Sized>(
        &mut self,
        mbc: &mut T,
        register: GeneralRegister,
    ) {
        let loaded_value = mbc.get_next_u8(self.program_counter.into());

        if MODE == Mode::Debug {
            println!(
                "{:#06x}: Loading {:#06x} to Register {:?}",
                self.program_counter, loaded_value, register
            );
        }

        self.registers.set(register, loaded_value);

        self.program_counter += 2;
    }

    fn ld_next_16<T: MemoryBankController + ?Sized>(
        &mut self,
        mbc: &mut T,
        register: CombinedRegister,
    ) {
        let loaded_value = mbc.get_next_u16(self.program_counter.into());

        if MODE == Mode::Debug {
            println!(
                "{:#06x}: Loading {:#06x} to Register {:?}",
                self.program_counter, loaded_value, register
            );
        }

        self.registers.set_combined(register, loaded_value);

        self.program_counter += 3;
    }

    fn ld_to_sp<T: MemoryBankController + ?Sized>(&mut self, mbc: &mut T) {
        let loaded_value = mbc.get_next_u16(self.program_counter.into());

        if MODE == Mode::Debug {
            println!(
                "{:#06x}: Loading value {:#06x} into Stack Pointer",
                self.program_counter, loaded_value
            );
        }

        self.stack_pointer = loaded_value;

        self.program_counter += 3;
    }

    fn display_current_registers(&self, op: u8) {
        println!(
            "{:#06x}: ({:#04x}) {}",
            self.program_counter,
            op,
            self.registers.display()
        );
    }

    fn read_memory_with_offset(
        &self,
        location: CombinedRegister,
        offset: Option<MemoryOffset>,
    ) -> u16 {
        let memory_loc = self.registers.get_combined(location);

        match offset {
            None => memory_loc,
            Some(MemoryOffset::Plus) => memory_loc.wrapping_add(1),
            Some(MemoryOffset::Minus) => memory_loc.wrapping_sub(1),
        }
    }

    fn ld_rr_r<T: MemoryBankController + ?Sized>(
        &mut self,
        mbc: &mut T,
        location: CombinedRegister,
        register: GeneralRegister,
        offset: Option<MemoryOffset>,
    ) {
        if MODE == Mode::Debug {
            println!(
                "{:#06x}: Loading {:#04x} from Register {:?} to ({:?})",
                self.program_counter,
                self.registers.get(register),
                register,
                location,
            );
        }

        let memory_location = self.read_memory_with_offset(location, offset);

        mbc.write_memory(memory_location.into(), self.registers.get(register));

        self.program_counter += 1;
    }

    fn ld_r_rr<T: MemoryBankController + ?Sized>(
        &mut self,
        mbc: &mut T,
        register: GeneralRegister,
        location: CombinedRegister,
        offset: Option<MemoryOffset>,
    ) {
        if MODE == Mode::Debug {
            println!(
                "{:#06x}: Loading {:#04x} from Location ({:?}) to {:?}",
                self.program_counter,
                self.registers.get(register),
                location,
                register,
            );
        }

        let memory_location = self.read_memory_with_offset(location, offset);

        self.registers
            .set(register, mbc.read_memory(memory_location.into()));

        self.program_counter += 1;
    }

    fn rotate_left(value: u8) -> u8 {
        (value << 1) | value.wrapping_shr(7)
    }

    // specialized from rlc because the flags are set differently
    fn rlca(&mut self) {
        let value = self.registers.get(GeneralRegister::A);
        let result = SharpSM83::rotate_left(value);
        self.registers.set(GeneralRegister::A, result);

        if bit_set_at(7, value) {
            self.registers.set_flag(FlagRegisterValue::C);
        } else {
            self.registers.unset_flag(FlagRegisterValue::C);
        }

        self.registers.unset_flag(FlagRegisterValue::Z);
        self.registers.unset_flag(FlagRegisterValue::H);
        self.registers.unset_flag(FlagRegisterValue::N);

        if MODE == Mode::Debug {
            println!(
                "{:#06x}: RLCA ({:b}) -> ({:b})",
                self.program_counter, value, result
            );
        }

        self.program_counter += 1;
    }

    fn rlc(&mut self, value: u8) -> u8 {
        let result = SharpSM83::rotate_left(value);

        if bit_set_at(7, value) {
            self.registers.set_flag(FlagRegisterValue::C);
        } else {
            self.registers.unset_flag(FlagRegisterValue::C);
        }

        if result == 0 {
            self.registers.set_flag(FlagRegisterValue::Z);
        } else {
            self.registers.unset_flag(FlagRegisterValue::Z);
        }

        self.registers.unset_flag(FlagRegisterValue::H);
        self.registers.unset_flag(FlagRegisterValue::N);

        result
    }

    fn rlc_register(&mut self, register: GeneralRegister) {
        let value = self.registers.get(register);
        let result = self.rlc(value);

        self.registers.set(register, result);

        if MODE == Mode::Debug {
            println!(
                "{:#06x}: Rotating Register {:?} ({:b}) left ({:b})",
                self.program_counter, register, value, result,
            );
        }
    }

    fn rlc_memory<T: MemoryBankController + ?Sized>(&mut self, mbc: &mut T) {
        let location: usize = self.registers.get_combined(CombinedRegister::HL).into();
        let value = mbc.read_memory(location);
        let result = self.rlc(value);

        mbc.write_memory(location, result);

        if MODE == Mode::Debug {
            println!(
                "{:#06x}: Rotating value at Memory Location (HL) ({:b}) left ({:b})",
                self.program_counter, value, result,
            );
        }
    }

    fn bit(&mut self, bit: u8, value: u8) -> bool {
        let result = bit_set_at(bit, value);

        if result {
            self.registers.unset_flag(FlagRegisterValue::Z);
        } else {
            self.registers.set_flag(FlagRegisterValue::Z);
        }

        self.registers.unset_flag(FlagRegisterValue::N);
        self.registers.set_flag(FlagRegisterValue::H);

        result
    }

    fn bit_register(&mut self, bit: u8, register: GeneralRegister) {
        let value = self.registers.get(register);

        let result = self.bit(bit, value);

        if MODE == Mode::Debug {
            println!(
                "{:#06x}: BIT on Register {:?} ({})",
                self.program_counter, register, result,
            );
        }
    }

    fn bit_memory<T: MemoryBankController + ?Sized>(&mut self, bit: u8, mbc: &T) {
        let value = mbc.read_memory(self.registers.get_combined(CombinedRegister::HL).into());

        let result = self.bit(bit, value);

        if MODE == Mode::Debug {
            println!(
                "{:#06x}: BIT on Memory Location (HL) ({})",
                self.program_counter, result,
            );
        }
    }

    fn res_register(&mut self, bit: u8, register: GeneralRegister) {
        let value = self.registers.get(register);
        let result = reset_bit(bit, value);

        if MODE == Mode::Debug {
            println!(
                "{:#06x}: RES on Register {:?} bit ({}) before ({:b}) after ({:b})",
                self.program_counter, register, bit, value, result,
            );
        }
    }

    fn res_memory<T: MemoryBankController + ?Sized>(&mut self, bit: u8, mbc: &mut T) {
        let location: usize = self.registers.get_combined(CombinedRegister::HL).into();
        let value = mbc.read_memory(location);
        let result = reset_bit(bit, value);

        mbc.write_memory(location, result);

        if MODE == Mode::Debug {
            println!(
                "{:#06x}: RES on Memory Location (HL) bit ({}) before ({:b}) after ({:b})",
                self.program_counter, bit, value, result,
            );
        }
    }

    fn prefix<T: MemoryBankController + ?Sized>(&mut self, mbc: &mut T) {
        let op = mbc.get_next_u8(self.program_counter.into());

        match op {
            0x00 => self.rlc_register(GeneralRegister::B),
            0x01 => self.rlc_register(GeneralRegister::C),
            0x02 => self.rlc_register(GeneralRegister::D),
            0x03 => self.rlc_register(GeneralRegister::E),
            0x04 => self.rlc_register(GeneralRegister::H),
            0x05 => self.rlc_register(GeneralRegister::L),
            0x06 => self.rlc_memory(mbc),
            0x07 => self.rlc_register(GeneralRegister::A),

            0x40 => self.bit_register(0, GeneralRegister::B),
            0x41 => self.bit_register(0, GeneralRegister::C),
            0x42 => self.bit_register(0, GeneralRegister::D),
            0x43 => self.bit_register(0, GeneralRegister::E),
            0x44 => self.bit_register(0, GeneralRegister::H),
            0x45 => self.bit_register(0, GeneralRegister::L),
            0x46 => self.bit_memory(0, mbc),
            0x47 => self.bit_register(0, GeneralRegister::A),
            0x48 => self.bit_register(1, GeneralRegister::B),
            0x49 => self.bit_register(1, GeneralRegister::C),
            0x4A => self.bit_register(1, GeneralRegister::D),
            0x4B => self.bit_register(1, GeneralRegister::E),
            0x4C => self.bit_register(1, GeneralRegister::H),
            0x4D => self.bit_register(1, GeneralRegister::L),
            0x4E => self.bit_memory(1, mbc),
            0x4F => self.bit_register(1, GeneralRegister::A),

            0x50 => self.bit_register(2, GeneralRegister::B),
            0x51 => self.bit_register(2, GeneralRegister::C),
            0x52 => self.bit_register(2, GeneralRegister::D),
            0x53 => self.bit_register(2, GeneralRegister::E),
            0x54 => self.bit_register(2, GeneralRegister::H),
            0x55 => self.bit_register(2, GeneralRegister::L),
            0x56 => self.bit_memory(2, mbc),
            0x57 => self.bit_register(2, GeneralRegister::A),
            0x58 => self.bit_register(3, GeneralRegister::B),
            0x59 => self.bit_register(3, GeneralRegister::C),
            0x5A => self.bit_register(3, GeneralRegister::D),
            0x5B => self.bit_register(3, GeneralRegister::E),
            0x5C => self.bit_register(3, GeneralRegister::H),
            0x5D => self.bit_register(3, GeneralRegister::L),
            0x5E => self.bit_memory(3, mbc),
            0x5F => self.bit_register(3, GeneralRegister::A),

            0x60 => self.bit_register(4, GeneralRegister::B),
            0x61 => self.bit_register(4, GeneralRegister::C),
            0x62 => self.bit_register(4, GeneralRegister::D),
            0x63 => self.bit_register(4, GeneralRegister::E),
            0x64 => self.bit_register(4, GeneralRegister::H),
            0x65 => self.bit_register(4, GeneralRegister::L),
            0x66 => self.bit_memory(4, mbc),
            0x67 => self.bit_register(4, GeneralRegister::A),
            0x68 => self.bit_register(5, GeneralRegister::B),
            0x69 => self.bit_register(5, GeneralRegister::C),
            0x6A => self.bit_register(5, GeneralRegister::D),
            0x6B => self.bit_register(5, GeneralRegister::E),
            0x6C => self.bit_register(5, GeneralRegister::H),
            0x6D => self.bit_register(5, GeneralRegister::L),
            0x6E => self.bit_memory(5, mbc),
            0x6F => self.bit_register(5, GeneralRegister::A),

            0x70 => self.bit_register(6, GeneralRegister::B),
            0x71 => self.bit_register(6, GeneralRegister::C),
            0x72 => self.bit_register(6, GeneralRegister::D),
            0x73 => self.bit_register(6, GeneralRegister::E),
            0x74 => self.bit_register(6, GeneralRegister::H),
            0x75 => self.bit_register(6, GeneralRegister::L),
            0x76 => self.bit_memory(6, mbc),
            0x77 => self.bit_register(6, GeneralRegister::A),
            0x78 => self.bit_register(7, GeneralRegister::B),
            0x79 => self.bit_register(7, GeneralRegister::C),
            0x7A => self.bit_register(7, GeneralRegister::D),
            0x7B => self.bit_register(7, GeneralRegister::E),
            0x7C => self.bit_register(7, GeneralRegister::H),
            0x7D => self.bit_register(7, GeneralRegister::L),
            0x7E => self.bit_memory(7, mbc),
            0x7F => self.bit_register(7, GeneralRegister::A),

            0x80 => self.res_register(0, GeneralRegister::B),
            0x81 => self.res_register(0, GeneralRegister::C),
            0x82 => self.res_register(0, GeneralRegister::D),
            0x83 => self.res_register(0, GeneralRegister::E),
            0x84 => self.res_register(0, GeneralRegister::H),
            0x85 => self.res_register(0, GeneralRegister::L),
            0x86 => self.res_memory(0, mbc),
            0x87 => self.res_register(0, GeneralRegister::A),
            0x88 => self.res_register(1, GeneralRegister::B),
            0x89 => self.res_register(1, GeneralRegister::C),
            0x8A => self.res_register(1, GeneralRegister::D),
            0x8B => self.res_register(1, GeneralRegister::E),
            0x8C => self.res_register(1, GeneralRegister::H),
            0x8D => self.res_register(1, GeneralRegister::L),
            0x8E => self.res_memory(1, mbc),
            0x8F => self.res_register(1, GeneralRegister::A),

            0x90 => self.res_register(2, GeneralRegister::B),
            0x91 => self.res_register(2, GeneralRegister::C),
            0x92 => self.res_register(2, GeneralRegister::D),
            0x93 => self.res_register(2, GeneralRegister::E),
            0x94 => self.res_register(2, GeneralRegister::H),
            0x95 => self.res_register(2, GeneralRegister::L),
            0x96 => self.res_memory(2, mbc),
            0x97 => self.res_register(2, GeneralRegister::A),
            0x98 => self.res_register(3, GeneralRegister::B),
            0x99 => self.res_register(3, GeneralRegister::C),
            0x9A => self.res_register(3, GeneralRegister::D),
            0x9B => self.res_register(3, GeneralRegister::E),
            0x9C => self.res_register(3, GeneralRegister::H),
            0x9D => self.res_register(3, GeneralRegister::L),
            0x9E => self.res_memory(3, mbc),
            0x9F => self.res_register(3, GeneralRegister::A),

            0xA0 => self.res_register(4, GeneralRegister::B),
            0xA1 => self.res_register(4, GeneralRegister::C),
            0xA2 => self.res_register(4, GeneralRegister::D),
            0xA3 => self.res_register(4, GeneralRegister::E),
            0xA4 => self.res_register(4, GeneralRegister::H),
            0xA5 => self.res_register(4, GeneralRegister::L),
            0xA6 => self.res_memory(4, mbc),
            0xA7 => self.res_register(4, GeneralRegister::A),
            0xA8 => self.res_register(5, GeneralRegister::B),
            0xA9 => self.res_register(5, GeneralRegister::C),
            0xAA => self.res_register(5, GeneralRegister::D),
            0xAB => self.res_register(5, GeneralRegister::E),
            0xAC => self.res_register(5, GeneralRegister::H),
            0xAD => self.res_register(5, GeneralRegister::L),
            0xAE => self.res_memory(5, mbc),
            0xAF => self.res_register(5, GeneralRegister::A),

            0xB0 => self.res_register(6, GeneralRegister::B),
            0xB1 => self.res_register(6, GeneralRegister::C),
            0xB2 => self.res_register(6, GeneralRegister::D),
            0xB3 => self.res_register(6, GeneralRegister::E),
            0xB4 => self.res_register(6, GeneralRegister::H),
            0xB5 => self.res_register(6, GeneralRegister::L),
            0xB6 => self.res_memory(6, mbc),
            0xB7 => self.res_register(6, GeneralRegister::A),
            0xB8 => self.res_register(7, GeneralRegister::B),
            0xB9 => self.res_register(7, GeneralRegister::C),
            0xBA => self.res_register(7, GeneralRegister::D),
            0xBB => self.res_register(7, GeneralRegister::E),
            0xBC => self.res_register(7, GeneralRegister::H),
            0xBD => self.res_register(7, GeneralRegister::L),
            0xBE => self.res_memory(7, mbc),
            0xBF => self.res_register(7, GeneralRegister::A),

            _ => self.not_implemented(&format!("Prefix not implemented: {:#04x}", op)),
        }

        self.program_counter += 2;
    }

    pub fn apply_operation<T: MemoryBankController + ?Sized>(&mut self, mbc: &mut T) {
        let op = mbc.read_memory(self.program_counter.into());

        match op {
            0x00 => self.nop(),
            0x01 => self.ld_next_16(mbc, CombinedRegister::BC),
            0x02 => self.ld_rr_r(mbc, CombinedRegister::BC, GeneralRegister::A, None),
            0x03 => self.inc16(CombinedRegister::BC),
            0x04 => self.inc(GeneralRegister::B),
            0x05 => self.dec(GeneralRegister::B),
            0x06 => self.ld_next_8(mbc, GeneralRegister::B),
            0x07 => self.rlca(),
            0x08 => self.not_implemented("LD (u16), Stack Pointer"),
            0x09 => self.add_combined_register(CombinedRegister::BC),
            0x0A => self.ld_r_rr(mbc, GeneralRegister::A, CombinedRegister::BC, None),
            0x0B => self.not_implemented("DEC BC"),
            0x0C => self.inc(GeneralRegister::C),
            0x0D => self.dec(GeneralRegister::C),
            0x0E => self.ld_next_8(mbc, GeneralRegister::C),
            0x0F => self.not_implemented("RRCA"),

            0x10 => self.not_implemented("STOP"),
            0x11 => self.ld_next_16(mbc, CombinedRegister::DE),
            0x12 => self.ld_rr_r(mbc, CombinedRegister::DE, GeneralRegister::A, None),
            0x13 => self.inc16(CombinedRegister::DE),
            0x14 => self.inc(GeneralRegister::D),
            0x15 => self.dec(GeneralRegister::D),
            0x16 => self.ld_next_8(mbc, GeneralRegister::D),
            0x17 => self.not_implemented("RLA"),
            0x18 => self.jr(mbc, None),
            0x19 => self.add_combined_register(CombinedRegister::DE),
            0x1A => self.ld_r_rr(mbc, GeneralRegister::A, CombinedRegister::DE, None),
            0x1B => self.not_implemented("DEC DE"),
            0x1C => self.inc(GeneralRegister::E),
            0x1D => self.dec(GeneralRegister::E),
            0x1E => self.ld_next_8(mbc, GeneralRegister::E),
            0x1F => self.not_implemented("RRA"),

            0x20 => self.jr(mbc, Some(FlagRegisterValue::NZ)),
            0x21 => self.ld_next_16(mbc, CombinedRegister::HL),
            0x22 => self.ld_rr_r(
                mbc,
                CombinedRegister::HL,
                GeneralRegister::A,
                Some(MemoryOffset::Plus),
            ),
            0x23 => self.inc16(CombinedRegister::HL),
            0x24 => self.inc(GeneralRegister::H),
            0x25 => self.dec(GeneralRegister::H),
            0x26 => self.not_implemented("LD H, d8"),
            0x27 => self.not_implemented("DAA"),
            0x28 => self.jr(mbc, Some(FlagRegisterValue::Z)),
            0x29 => self.add_combined_register(CombinedRegister::HL),
            0x2A => self.ld_r_rr(
                mbc,
                GeneralRegister::A,
                CombinedRegister::HL,
                Some(MemoryOffset::Plus),
            ),
            0x2B => self.not_implemented("DEC HL"),
            0x2C => self.inc(GeneralRegister::L),
            0x2D => self.dec(GeneralRegister::L),
            0x2E => self.ld_next_8(mbc, GeneralRegister::L),
            0x2F => self.cpl(),

            0x30 => self.jr(mbc, Some(FlagRegisterValue::NC)),
            0x31 => self.ld_to_sp(mbc),
            0x32 => self.ld_rr_r(
                mbc,
                CombinedRegister::HL,
                GeneralRegister::A,
                Some(MemoryOffset::Minus),
            ),
            0x33 => self.not_implemented("INC SP"),
            0x34 => self.not_implemented("INC (HL) 1"),
            0x35 => self.not_implemented("DEC (HL) 1"),
            0x36 => self.not_implemented("LD (HL), u8"),
            0x37 => self.not_implemented("SCF"),
            0x38 => self.jr(mbc, Some(FlagRegisterValue::C)),
            0x39 => self.add_sp(),
            0x3A => self.ld_r_rr(
                mbc,
                GeneralRegister::A,
                CombinedRegister::HL,
                Some(MemoryOffset::Minus),
            ),
            0x3B => self.not_implemented("DEC SP"),
            0x3C => self.inc(GeneralRegister::A),
            0x3D => self.dec(GeneralRegister::A),
            0x3E => self.ld_next_8(mbc, GeneralRegister::A),
            0x3F => self.not_implemented("CCF"),

            0x40 => self.ld(GeneralRegister::B, GeneralRegister::B),
            0x41 => self.ld(GeneralRegister::B, GeneralRegister::C),
            0x42 => self.ld(GeneralRegister::B, GeneralRegister::D),
            0x43 => self.ld(GeneralRegister::B, GeneralRegister::E),
            0x44 => self.ld(GeneralRegister::B, GeneralRegister::H),
            0x45 => self.ld(GeneralRegister::B, GeneralRegister::L),
            0x46 => self.ld_r_rr(mbc, GeneralRegister::B, CombinedRegister::HL, None),
            0x47 => self.ld(GeneralRegister::B, GeneralRegister::A),
            0x48 => self.ld(GeneralRegister::C, GeneralRegister::B),
            0x49 => self.ld(GeneralRegister::C, GeneralRegister::C),
            0x4A => self.ld(GeneralRegister::C, GeneralRegister::D),
            0x4B => self.ld(GeneralRegister::C, GeneralRegister::E),
            0x4C => self.ld(GeneralRegister::C, GeneralRegister::H),
            0x4D => self.ld(GeneralRegister::C, GeneralRegister::L),
            0x4E => self.ld_r_rr(mbc, GeneralRegister::C, CombinedRegister::HL, None),
            0x4F => self.ld(GeneralRegister::C, GeneralRegister::A),

            0x50 => self.ld(GeneralRegister::D, GeneralRegister::B),
            0x51 => self.ld(GeneralRegister::D, GeneralRegister::C),
            0x52 => self.ld(GeneralRegister::D, GeneralRegister::D),
            0x53 => self.ld(GeneralRegister::D, GeneralRegister::E),
            0x54 => self.ld(GeneralRegister::D, GeneralRegister::H),
            0x55 => self.ld(GeneralRegister::D, GeneralRegister::L),
            0x56 => self.ld_r_rr(mbc, GeneralRegister::D, CombinedRegister::HL, None),
            0x57 => self.ld(GeneralRegister::D, GeneralRegister::A),
            0x58 => self.ld(GeneralRegister::E, GeneralRegister::B),
            0x59 => self.ld(GeneralRegister::E, GeneralRegister::C),
            0x5A => self.ld(GeneralRegister::E, GeneralRegister::D),
            0x5B => self.ld(GeneralRegister::E, GeneralRegister::E),
            0x5C => self.ld(GeneralRegister::E, GeneralRegister::H),
            0x5D => self.ld(GeneralRegister::E, GeneralRegister::L),
            0x5E => self.ld_r_rr(mbc, GeneralRegister::E, CombinedRegister::HL, None),
            0x5F => self.ld(GeneralRegister::E, GeneralRegister::A),

            0x60 => self.ld(GeneralRegister::H, GeneralRegister::B),
            0x61 => self.ld(GeneralRegister::H, GeneralRegister::C),
            0x62 => self.ld(GeneralRegister::H, GeneralRegister::D),
            0x63 => self.ld(GeneralRegister::H, GeneralRegister::E),
            0x64 => self.ld(GeneralRegister::H, GeneralRegister::H),
            0x65 => self.ld(GeneralRegister::H, GeneralRegister::L),
            0x66 => self.ld_r_rr(mbc, GeneralRegister::H, CombinedRegister::HL, None),
            0x67 => self.ld(GeneralRegister::H, GeneralRegister::A),
            0x68 => self.ld(GeneralRegister::L, GeneralRegister::B),
            0x69 => self.ld(GeneralRegister::L, GeneralRegister::C),
            0x6A => self.ld(GeneralRegister::L, GeneralRegister::D),
            0x6B => self.ld(GeneralRegister::L, GeneralRegister::E),
            0x6C => self.ld(GeneralRegister::L, GeneralRegister::H),
            0x6D => self.ld(GeneralRegister::L, GeneralRegister::L),
            0x6E => self.ld_r_rr(mbc, GeneralRegister::L, CombinedRegister::HL, None),
            0x6F => self.ld(GeneralRegister::L, GeneralRegister::A),

            0x70 => self.ld_rr_r(mbc, CombinedRegister::HL, GeneralRegister::B, None),
            0x71 => self.ld_rr_r(mbc, CombinedRegister::HL, GeneralRegister::C, None),
            0x72 => self.ld_rr_r(mbc, CombinedRegister::HL, GeneralRegister::D, None),
            0x73 => self.ld_rr_r(mbc, CombinedRegister::HL, GeneralRegister::E, None),
            0x74 => self.ld_rr_r(mbc, CombinedRegister::HL, GeneralRegister::H, None),
            0x75 => self.ld_rr_r(mbc, CombinedRegister::HL, GeneralRegister::L, None),
            0x76 => self.not_implemented("HALT"),
            0x77 => self.ld_rr_r(mbc, CombinedRegister::HL, GeneralRegister::A, None),
            0x78 => self.ld(GeneralRegister::A, GeneralRegister::B),
            0x79 => self.ld(GeneralRegister::A, GeneralRegister::C),
            0x7A => self.ld(GeneralRegister::A, GeneralRegister::D),
            0x7B => self.ld(GeneralRegister::A, GeneralRegister::E),
            0x7C => self.ld(GeneralRegister::A, GeneralRegister::H),
            0x7D => self.ld(GeneralRegister::A, GeneralRegister::L),
            0x7E => self.ld_r_rr(mbc, GeneralRegister::A, CombinedRegister::HL, None),
            0x7F => self.ld(GeneralRegister::A, GeneralRegister::A),

            0x80 => self.add_register(GeneralRegister::B),
            0x81 => self.add_register(GeneralRegister::C),
            0x82 => self.add_register(GeneralRegister::D),
            0x83 => self.add_register(GeneralRegister::E),
            0x84 => self.add_register(GeneralRegister::H),
            0x85 => self.add_register(GeneralRegister::L),
            0x86 => self.add_memory(mbc),
            0x87 => self.add_register(GeneralRegister::A),
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
            0xB8 => self.cp_register(GeneralRegister::B),
            0xB9 => self.cp_register(GeneralRegister::C),
            0xBA => self.cp_register(GeneralRegister::D),
            0xBB => self.cp_register(GeneralRegister::E),
            0xBC => self.cp_register(GeneralRegister::H),
            0xBD => self.cp_register(GeneralRegister::L),
            0xBE => self.cp_memory(
                mbc,
                self.registers.get_combined(CombinedRegister::HL).into(),
            ),
            0xBF => self.cp_register(GeneralRegister::A),

            0xC0 => self.not_implemented("RET NZ"),
            0xC1 => self.not_implemented("POP BC"),
            0xC2 => self.jp(mbc, Some(FlagRegisterValue::NZ)),
            0xC3 => self.jp(mbc, None),
            0xC4 => self.not_implemented("CALL NZ, u16"),
            0xC5 => self.not_implemented("PUSH BC"),
            0xC6 => self.not_implemented("ADD A, u8"),
            0xC7 => self.not_implemented("RST 00h"),
            0xC8 => self.not_implemented("RET Z"),
            0xC9 => self.not_implemented("RET"),
            0xCA => self.jp(mbc, Some(FlagRegisterValue::Z)),
            0xCB => self.prefix(mbc),
            0xCC => self.not_implemented("CALL Z, u16"),
            0xCD => self.call(mbc),
            0xCE => self.not_implemented("ADC A, u8"),
            0xCF => self.not_implemented("RST 08h"),

            0xD0 => self.not_implemented("RET NC"),
            0xD1 => self.not_implemented("POP DE"),
            0xD2 => self.jp(mbc, Some(FlagRegisterValue::NC)),
            0xD3 => (),
            0xD4 => self.not_implemented("CALL NC, u16"),
            0xD5 => self.not_implemented("PUSH DE"),
            0xD6 => self.not_implemented("SUB A, u8"),
            0xD7 => self.not_implemented("RST 10h"),
            0xD8 => self.not_implemented("RET C"),
            0xD9 => self.not_implemented("RETI"),
            0xDA => self.jp(mbc, Some(FlagRegisterValue::C)),
            0xDB => (),
            0xDC => self.not_implemented("CALL C, u16"),
            0xDD => (),
            0xDE => self.not_implemented("SBC A, u8"),
            0xDF => self.not_implemented("RST 18h"),

            0xE0 => self.not_implemented("LD (FF00+u8), A"),
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
            0xFE => self.cp_u8(mbc),
            0xFF => self.not_implemented("RST 38h"),
        }

        if MODE == Mode::Debug {
            self.display_current_registers(op);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SharpSM83;
    use crate::cpu_registers::GeneralRegister;

    #[test]
    fn inc_should_work() {
        let mut cpu = SharpSM83::default();
        cpu.inc(GeneralRegister::A);
        let a_val = cpu.registers.get(GeneralRegister::A);

        assert_eq!(a_val, 0x01);
    }

    #[test]
    fn inc_should_wrap() {
        let mut cpu = SharpSM83::default();
        cpu.registers.set(GeneralRegister::A, 0xff);
        cpu.inc(GeneralRegister::A);
        let a_val = cpu.registers.get(GeneralRegister::A);

        assert_eq!(a_val, 0x00);
    }

    #[test]
    fn dec_should_work() {
        let mut cpu = SharpSM83::default();
        cpu.registers.set(GeneralRegister::A, 0x02);
        cpu.dec(GeneralRegister::A);
        let a_val = cpu.registers.get(GeneralRegister::A);

        assert_eq!(a_val, 0x01);
    }

    #[test]
    fn dec_should_wrap() {
        let mut cpu = SharpSM83::default();
        cpu.dec(GeneralRegister::A);
        let a_val = cpu.registers.get(GeneralRegister::A);

        assert_eq!(a_val, 0xff);
    }

    #[test]
    fn add_should_work() {
        let mut cpu = SharpSM83::default();
        cpu.registers.set(GeneralRegister::B, 0x01);
        cpu.add_register(GeneralRegister::B);
        let a_val = cpu.registers.get(GeneralRegister::A);

        assert_eq!(a_val, 0x01);
    }

    #[test]
    fn add_should_wrap() {
        let mut cpu = SharpSM83::default();
        cpu.registers.set(GeneralRegister::A, 0xff);
        cpu.registers.set(GeneralRegister::B, 0x01);
        cpu.add_register(GeneralRegister::B);
        let a_val = cpu.registers.get(GeneralRegister::A);

        assert_eq!(a_val, 0x00);
    }

    #[test]
    fn sub_should_work() {
        let mut cpu = SharpSM83::default();
        cpu.registers.set(GeneralRegister::A, 0x02);
        cpu.registers.set(GeneralRegister::B, 0x01);
        cpu.sub(GeneralRegister::B);
        let a_val = cpu.registers.get(GeneralRegister::A);

        assert_eq!(a_val, 0x01);
    }

    #[test]
    fn sub_should_wrap() {
        let mut cpu = SharpSM83::default();
        cpu.registers.set(GeneralRegister::B, 0x01);
        cpu.sub(GeneralRegister::B);
        let a_val = cpu.registers.get(GeneralRegister::A);

        assert_eq!(a_val, 0xff);
    }

    #[test]
    fn ld_should_work() {
        let mut cpu = SharpSM83::default();
        cpu.registers.set(GeneralRegister::A, 0x01);
        cpu.ld(GeneralRegister::B, GeneralRegister::A);

        assert_eq!(cpu.registers.get(GeneralRegister::B), 0x01);
    }
}
