use crate::cpu_registers::{CombinedRegister, GeneralRegister, Registers};
use crate::flag_register::FlagRegisterValue;
use crate::mbc::MBC;
use crate::utils::{u16_to_u8s, u8s_to_u16, BitWise, Carryable};

// const CLOCK_MHZ: f64 = 4.194304;
// FLAGS = (Z)ero, (N)egative, (H)alf Carry, (C)arry

#[derive(Default)]
pub struct Cpu {
    pub program_counter: u16,
    pub stack_pointer: u16,
    pub registers: Registers,
    pub interrupts_enabled: bool,
    pub current_op: u8,
    count: u64,
}

// #[cfg_attr(rustfmt, rustfmt_skip)]
// const OP_CYCLES_BYTES: [u8; 256] = [
//     1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1,
//     2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,
//     2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,
//     2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//     1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//     1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 0, 3, 3, 2, 1,
//     1, 1, 3, 0, 3, 1, 2, 1, 1, 1, 3, 0, 3, 0, 2, 1,
//     2, 1, 1, 0, 0, 1, 2, 1, 2, 1, 3, 0, 0, 0, 2, 1,
//     2, 1, 1, 1, 0, 1, 2, 1, 2, 1, 3, 1, 0, 0, 2, 1
// ];

trait Ops {
    fn nop(&mut self);

    fn ccf(&mut self);
    fn stop(&mut self);
    fn di(&mut self);
    fn ei(&mut self);

    fn ret(&mut self, mbc: &MBC);
    fn ret_if(&mut self, mbc: &MBC, flag: FlagRegisterValue);
    fn ret_not(&mut self, mbc: &MBC, flag: FlagRegisterValue);
    fn reti(&mut self, mbc: &MBC);

    fn ld_r_r(&mut self, to: GeneralRegister, from: GeneralRegister);
    fn ld_r_d8(&mut self, mbc: &MBC, register: GeneralRegister);
    fn ld_rr_d16(&mut self, mbc: &MBC, register: CombinedRegister);

    fn ld_mem_rr_r(&mut self, mbc: &mut MBC, to_address: CombinedRegister, from: GeneralRegister);
    fn ldi_mem_rr_r(&mut self, mbc: &mut MBC, to_address: CombinedRegister, from: GeneralRegister);
    fn ldd_mem_rr_r(&mut self, mbc: &mut MBC, to_address: CombinedRegister, from: GeneralRegister);

    fn ld_r_mem_rr(&mut self, mbc: &MBC, to: GeneralRegister, from_address: CombinedRegister);
    fn ldi_r_mem_rr(&mut self, mbc: &MBC, to: GeneralRegister, from_address: CombinedRegister);
    fn ldd_r_mem_rr(&mut self, mbc: &MBC, to: GeneralRegister, from_address: CombinedRegister);

    fn ld_mem_rr_d8(&mut self, mbc: &mut MBC, address: CombinedRegister);

    fn ld_mem_a8_a(&mut self, mbc: &mut MBC);
    fn ld_a_mem_a8(&mut self, mbc: &MBC);

    fn ld_mem_a16_a(&mut self, mbc: &mut MBC);
    fn ld_a_mem_a16(&mut self, mbc: &MBC);

    fn ld_mem_r_a(&mut self, mbc: &mut MBC, register: GeneralRegister);
    fn ld_a_mem_r(&mut self, mbc: &MBC, register: GeneralRegister);

    fn ld_sp_d16(&mut self, mbc: &MBC);
    fn ld_mem_a16_sp(&mut self, mbc: &mut MBC);
    fn ld_sp_rr(&mut self, register: CombinedRegister);

    fn add_r(&mut self, register: GeneralRegister);
    fn add_d8(&mut self, mbc: &MBC);
    fn add_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister);

    fn add_rr(&mut self, register: CombinedRegister);
    fn add_sp(&mut self);
    // fn add_sp_s8(&mut self, mbc: &MBC);

    fn adc_r(&mut self, register: GeneralRegister);
    fn adc_d8(&mut self, mbc: &MBC);
    fn adc_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister);

    fn sub_r(&mut self, register: GeneralRegister);
    fn sub_d8(&mut self, mbc: &MBC);
    fn sub_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister);

    fn sbc_r(&mut self, register: GeneralRegister);
    fn sbc_d8(&mut self, mbc: &MBC);
    fn sbc_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister);

    fn and_r(&mut self, register: GeneralRegister);
    fn and_d8(&mut self, mbc: &MBC);
    fn and_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister);

    fn xor_r(&mut self, register: GeneralRegister);
    fn xor_d8(&mut self, mbc: &MBC);
    fn xor_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister);

    fn or_r(&mut self, register: GeneralRegister);
    fn or_d8(&mut self, mbc: &MBC);
    fn or_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister);

    fn cp_r(&mut self, register: GeneralRegister);
    fn cp_d8(&mut self, mbc: &MBC);
    fn cp_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister);

    fn inc_r(&mut self, register: GeneralRegister);
    fn inc_mem_rr(&mut self, mbc: &mut MBC, register: CombinedRegister);
    fn inc_rr(&mut self, register: CombinedRegister);
    fn inc_sp(&mut self);

    fn dec_r(&mut self, register: GeneralRegister);
    fn dec_mem_rr(&mut self, mbc: &mut MBC, register: CombinedRegister);
    fn dec_rr(&mut self, register: CombinedRegister);
    fn dec_sp(&mut self);

    fn push_rr(&mut self, mbc: &mut MBC, register: CombinedRegister);
    fn pop_rr(&mut self, mbc: &MBC, register: CombinedRegister);

    fn cpl(&mut self);
    fn rlca(&mut self);
    fn rla(&mut self);
    fn rrca(&mut self);
    fn rra(&mut self);
}

trait PrefixOps {
    fn rlc_r(&mut self, register: GeneralRegister);
    fn rlc_mem_rr(&mut self, mbc: &mut MBC, register: CombinedRegister);

    fn rl_r(&mut self, register: GeneralRegister);
    fn rl_mem_rr(&mut self, mbc: &mut MBC, register: CombinedRegister);

    fn rrc_r(&mut self, register: GeneralRegister);
    fn rrc_mem_rr(&mut self, mbc: &mut MBC, register: CombinedRegister);

    fn rr_r(&mut self, register: GeneralRegister);
    fn rr_mem_rr(&mut self, mbc: &mut MBC, register: CombinedRegister);

    fn srl_r(&mut self, register: GeneralRegister);

    fn bit_r(&mut self, bit: u8, register: GeneralRegister);
    fn bit_mem_rr(&mut self, mbc: &MBC, bit: u8, register: CombinedRegister);
}

impl Ops for Cpu {
    /// An explicit "nothing" instruction to the CPU
    fn nop(&mut self) {
        self.program_counter += 1;
    }

    fn ccf(&mut self) {
        self.registers
            .toggle_flag(
                FlagRegisterValue::C,
                !self.registers.is_flag_set(FlagRegisterValue::C),
            )
            .unset_flag(FlagRegisterValue::N)
            .unset_flag(FlagRegisterValue::H);

        self.program_counter += 1;
    }

    fn stop(&mut self) {
        println!("Stopped");
        self.program_counter += 2;
    }

    fn di(&mut self) {
        self.interrupts_enabled = false;
        self.program_counter += 1;
    }

    fn ei(&mut self) {
        self.interrupts_enabled = true;
        self.program_counter += 1;
    }

    fn ret(&mut self, mbc: &MBC) {
        self.program_counter = self.pop_stack(mbc);
    }

    fn ret_if(&mut self, mbc: &MBC, flag: FlagRegisterValue) {
        if self.registers.is_flag_set(flag) {
            self.ret(mbc);
        } else {
            self.program_counter += 1;
        }
    }

    fn ret_not(&mut self, mbc: &MBC, flag: FlagRegisterValue) {
        if !self.registers.is_flag_set(flag) {
            self.ret(mbc);
        } else {
            self.program_counter += 1;
        }
    }

    fn reti(&mut self, mbc: &MBC) {
        self.ret(mbc);
        self.ei();
    }

    fn ld_r_r(&mut self, to: GeneralRegister, from: GeneralRegister) {
        self.registers.set(to, self.registers.get(from));

        self.program_counter += 1;
    }

    fn ld_r_d8(&mut self, mbc: &MBC, register: GeneralRegister) {
        self.registers
            .set(register, mbc.get_next_u8(self.program_counter.into()));

        self.program_counter += 2;
    }

    fn ld_rr_d16(&mut self, mbc: &MBC, register: CombinedRegister) {
        let value = mbc.get_next_u16(self.program_counter.into());

        self.registers.set16(register, value);

        self.program_counter += 3;
    }

    fn ld_mem_rr_r(&mut self, mbc: &mut MBC, to_address: CombinedRegister, from: GeneralRegister) {
        mbc.write(
            self.registers.get16(to_address).into(),
            self.registers.get(from),
        );

        self.program_counter += 1;
    }

    fn ldi_mem_rr_r(&mut self, mbc: &mut MBC, to_address: CombinedRegister, from: GeneralRegister) {
        self.ld_mem_rr_r(mbc, to_address, from);
        self.registers.increment16(to_address);
    }

    fn ldd_mem_rr_r(&mut self, mbc: &mut MBC, to_address: CombinedRegister, from: GeneralRegister) {
        self.ld_mem_rr_r(mbc, to_address, from);
        self.registers.decrement16(to_address);
    }

    fn ld_r_mem_rr(&mut self, mbc: &MBC, to: GeneralRegister, from_address: CombinedRegister) {
        self.registers
            .set(to, mbc.read(self.registers.get16(from_address).into()));

        self.program_counter += 1;
    }

    fn ldi_r_mem_rr(&mut self, mbc: &MBC, to: GeneralRegister, from_address: CombinedRegister) {
        self.ld_r_mem_rr(mbc, to, from_address);
        self.registers.increment16(from_address);
    }

    fn ldd_r_mem_rr(&mut self, mbc: &MBC, to: GeneralRegister, from_address: CombinedRegister) {
        self.ld_r_mem_rr(mbc, to, from_address);
        self.registers.decrement16(from_address);
    }

    fn ld_mem_rr_d8(&mut self, mbc: &mut MBC, address: CombinedRegister) {
        mbc.write(
            self.registers.get16(address).into(),
            mbc.get_next_u8(self.program_counter.into()),
        );

        self.program_counter += 2;
    }

    fn ld_mem_a8_a(&mut self, mbc: &mut MBC) {
        mbc.write(
            (0xff00 + mbc.get_next_u8(self.program_counter.into()) as u16).into(),
            self.registers.get(GeneralRegister::A),
        );

        self.program_counter += 2;
    }

    fn ld_a_mem_a8(&mut self, mbc: &MBC) {
        self.registers.set(
            GeneralRegister::A,
            mbc.read((0xff00 + mbc.get_next_u8(self.program_counter.into()) as u16).into()),
        );

        self.program_counter += 2;
    }

    fn ld_mem_r_a(&mut self, mbc: &mut MBC, register: GeneralRegister) {
        mbc.write(
            self.registers.get(register).into(),
            self.registers.get(GeneralRegister::A),
        );
        self.program_counter += 1;
    }

    fn ld_a_mem_r(&mut self, mbc: &MBC, register: GeneralRegister) {
        self.registers.set(
            GeneralRegister::A,
            mbc.read((0xff00 + mbc.read(self.registers.get(register).into()) as u16).into()),
        );

        self.program_counter += 1;
    }

    fn ld_mem_a16_a(&mut self, mbc: &mut MBC) {
        mbc.write(
            mbc.get_next_u16(self.program_counter.into()).into(),
            self.registers.get(GeneralRegister::A),
        );

        self.program_counter += 3;
    }

    fn ld_a_mem_a16(&mut self, mbc: &MBC) {
        self.registers.set(
            GeneralRegister::A,
            mbc.read(mbc.get_next_u16(self.program_counter.into()).into()),
        );

        self.program_counter += 3;
    }

    fn ld_sp_d16(&mut self, mbc: &MBC) {
        self.stack_pointer = mbc.get_next_u16(self.program_counter.into());

        self.program_counter += 3;
    }

    fn ld_sp_rr(&mut self, register: CombinedRegister) {
        self.stack_pointer = self.registers.get16(register);

        self.program_counter += 1;
    }

    fn ld_mem_a16_sp(&mut self, mbc: &mut MBC) {
        let location = mbc.get_next_u16(self.program_counter.into());
        let [value1, value2] = u16_to_u8s(self.stack_pointer);

        mbc.write(location.into(), value1);
        mbc.write((location + 1).into(), value2);

        self.program_counter += 3;
    }

    fn add_r(&mut self, register: GeneralRegister) {
        self.add_inner(self.registers.get(register));

        self.program_counter += 1;
    }

    fn add_d8(&mut self, mbc: &MBC) {
        self.add_inner(mbc.get_next_u8(self.program_counter.into()));

        self.program_counter += 2;
    }

    fn add_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister) {
        self.add_inner(mbc.read(self.registers.get16(register).into()));

        self.program_counter += 1;
    }

    fn add_rr(&mut self, register: CombinedRegister) {
        self.add_16_inner(self.registers.get16(register));

        self.program_counter += 1;
    }

    fn add_sp(&mut self) {
        self.add_16_inner(self.stack_pointer);

        self.program_counter += 1;
    }

    fn adc_r(&mut self, register: GeneralRegister) {
        self.adc_inner(self.registers.get(register));

        self.program_counter += 1;
    }

    fn adc_d8(&mut self, mbc: &MBC) {
        self.adc_inner(mbc.get_next_u8(self.program_counter.into()));

        self.program_counter += 2;
    }

    fn adc_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister) {
        self.adc_inner(mbc.read(self.registers.get16(register).into()));

        self.program_counter += 1;
    }

    fn sub_r(&mut self, register: GeneralRegister) {
        self.sub_inner(self.registers.get(register));

        self.program_counter += 1;
    }

    fn sub_d8(&mut self, mbc: &MBC) {
        self.sub_inner(mbc.get_next_u8(self.program_counter.into()));

        self.program_counter += 2;
    }

    fn sub_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister) {
        self.sub_inner(mbc.read(self.registers.get16(register).into()));

        self.program_counter += 1;
    }

    fn sbc_r(&mut self, register: GeneralRegister) {
        self.sbc_inner(self.registers.get(register));

        self.program_counter += 1;
    }

    fn sbc_d8(&mut self, mbc: &MBC) {
        self.sbc_inner(mbc.get_next_u8(self.program_counter.into()));

        self.program_counter += 2;
    }

    fn sbc_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister) {
        self.sbc_inner(mbc.read(self.registers.get16(register).into()));

        self.program_counter += 1;
    }

    fn and_r(&mut self, register: GeneralRegister) {
        self.and_inner(self.registers.get(register));

        self.program_counter += 1;
    }

    fn and_d8(&mut self, mbc: &MBC) {
        self.and_inner(mbc.get_next_u8(self.program_counter.into()));

        self.program_counter += 2;
    }

    fn and_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister) {
        self.and_inner(mbc.read(self.registers.get16(register).into()));

        self.program_counter += 1;
    }

    fn xor_r(&mut self, register: GeneralRegister) {
        self.xor_inner(self.registers.get(register));

        self.program_counter += 1;
    }

    fn xor_d8(&mut self, mbc: &MBC) {
        self.xor_inner(mbc.get_next_u8(self.program_counter.into()));

        self.program_counter += 2;
    }

    fn xor_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister) {
        self.xor_inner(mbc.read(self.registers.get16(register).into()));

        self.program_counter += 1;
    }

    fn or_r(&mut self, register: GeneralRegister) {
        self.or_inner(self.registers.get(register));

        self.program_counter += 1;
    }

    fn or_d8(&mut self, mbc: &MBC) {
        self.or_inner(mbc.get_next_u8(self.program_counter.into()));

        self.program_counter += 2;
    }

    fn or_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister) {
        self.or_inner(mbc.read(self.registers.get16(register).into()));

        self.program_counter += 1;
    }

    fn cp_r(&mut self, register: GeneralRegister) {
        self.cp_inner(self.registers.get(register));

        self.program_counter += 1;
    }

    fn cp_d8(&mut self, mbc: &MBC) {
        self.cp_inner(mbc.get_next_u8(self.program_counter.into()));

        self.program_counter += 2;
    }

    fn cp_mem_rr(&mut self, mbc: &MBC, register: CombinedRegister) {
        self.cp_inner(mbc.read(self.registers.get16(register).into()));

        self.program_counter += 1;
    }

    fn inc_r(&mut self, register: GeneralRegister) {
        let result = self.registers.get(register).wrapping_add(1);

        self.registers
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .unset_flag(FlagRegisterValue::N)
            .toggle_flag(FlagRegisterValue::H, result.inc_should_half_carry())
            .set(register, result);

        self.program_counter += 1;
    }

    fn inc_mem_rr(&mut self, mbc: &mut MBC, register: CombinedRegister) {
        let result = mbc
            .read(self.registers.get16(register).into())
            .wrapping_add(1);

        self.registers
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .unset_flag(FlagRegisterValue::N)
            .toggle_flag(FlagRegisterValue::H, result.inc_should_half_carry());

        mbc.write(self.registers.get16(register).into(), result);

        self.program_counter += 1;
    }

    fn inc_rr(&mut self, register: CombinedRegister) {
        self.registers
            .set16(register, self.registers.get16(register).wrapping_add(1));

        self.program_counter += 1;
    }

    fn inc_sp(&mut self) {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.program_counter += 1;
    }

    fn dec_r(&mut self, register: GeneralRegister) {
        let result = self.registers.get(register).wrapping_sub(1);

        self.registers
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .set_flag(FlagRegisterValue::N)
            .toggle_flag(FlagRegisterValue::H, result.dec_should_half_carry())
            .set(register, result);

        self.program_counter += 1;
    }

    fn dec_mem_rr(&mut self, mbc: &mut MBC, register: CombinedRegister) {
        let result = mbc
            .read(self.registers.get16(register).into())
            .wrapping_sub(1);

        self.registers
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .unset_flag(FlagRegisterValue::N)
            .toggle_flag(FlagRegisterValue::H, result.dec_should_half_carry());

        mbc.write(self.registers.get16(register).into(), result);

        self.program_counter += 1;
    }

    fn dec_rr(&mut self, register: CombinedRegister) {
        self.registers
            .set16(register, self.registers.get16(register).wrapping_sub(1));

        self.program_counter += 1;
    }

    fn dec_sp(&mut self) {
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        self.program_counter += 1;
    }

    fn push_rr(&mut self, mbc: &mut MBC, register: CombinedRegister) {
        self.push_stack(mbc, self.registers.get16(register));
    }

    fn pop_rr(&mut self, mbc: &MBC, register: CombinedRegister) {
        let out = self.pop_stack(mbc);
        self.registers.set16(register, out);
    }

    fn cpl(&mut self) {
        let result = self.registers.get(GeneralRegister::A) ^ 0xFF;

        self.registers
            .set_flag(FlagRegisterValue::N)
            .set_flag(FlagRegisterValue::H)
            .set(GeneralRegister::A, result);

        self.program_counter += 1;
    }

    fn rlca(&mut self) {
        let result = self.rlc_inner(self.registers.get(GeneralRegister::A));

        self.registers
            .set(GeneralRegister::A, result)
            .unset_flag(FlagRegisterValue::Z);

        self.program_counter += 1;
    }

    fn rla(&mut self) {
        let result = self.rl_inner(self.registers.get(GeneralRegister::A));

        self.registers
            .set(GeneralRegister::A, result)
            .unset_flag(FlagRegisterValue::Z);

        self.program_counter += 1;
    }

    fn rrca(&mut self) {
        let result = self.rrc_inner(self.registers.get(GeneralRegister::A));

        self.registers
            .set(GeneralRegister::A, result)
            .unset_flag(FlagRegisterValue::Z);

        self.program_counter += 1;
    }

    fn rra(&mut self) {
        let result = self.rr_inner(self.registers.get(GeneralRegister::A));

        self.registers
            .set(GeneralRegister::A, result)
            .unset_flag(FlagRegisterValue::Z);

        self.program_counter += 1;
    }
}

impl PrefixOps for Cpu {
    fn rlc_r(&mut self, register: GeneralRegister) {
        let result = self.rlc_inner(self.registers.get(register));
        self.registers.set(register, result);
        self.program_counter += 2;
    }

    fn rlc_mem_rr(&mut self, mbc: &mut MBC, register: CombinedRegister) {
        let location: usize = self.registers.get16(register).into();
        let result = self.rlc_inner(mbc.read(location));
        mbc.write(location, result);
        self.program_counter += 2;
    }

    fn rl_r(&mut self, register: GeneralRegister) {
        let result = self.rl_inner(self.registers.get(register));
        self.registers.set(register, result);
        self.program_counter += 2;
    }

    fn rl_mem_rr(&mut self, mbc: &mut MBC, register: CombinedRegister) {
        let location: usize = self.registers.get16(register).into();
        let result = self.rl_inner(mbc.read(location));
        mbc.write(location, result);
        self.program_counter += 2;
    }

    fn rrc_r(&mut self, register: GeneralRegister) {
        let result = self.rrc_inner(self.registers.get(register));
        self.registers.set(register, result);
        self.program_counter += 2;
    }

    fn rrc_mem_rr(&mut self, mbc: &mut MBC, register: CombinedRegister) {
        let location: usize = self.registers.get16(register).into();
        let result = self.rrc_inner(mbc.read(location));
        mbc.write(location, result);
        self.program_counter += 2;
    }

    fn rr_r(&mut self, register: GeneralRegister) {
        let result = self.rr_inner(self.registers.get(register));
        self.registers.set(register, result);
        self.program_counter += 2;
    }

    fn rr_mem_rr(&mut self, mbc: &mut MBC, register: CombinedRegister) {
        let location: usize = self.registers.get16(register).into();
        let result = self.rr_inner(mbc.read(location));
        mbc.write(location, result);
        self.program_counter += 2;
    }

    fn srl_r(&mut self, register: GeneralRegister) {
        let value = self.registers.get(register);
        let result = value.rotate_right(1).unset_bit(1 << 7);

        self.registers
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .unset_flag(FlagRegisterValue::N)
            .unset_flag(FlagRegisterValue::H)
            .toggle_flag(FlagRegisterValue::C, result.is_bit_set(1 << 0))
            .set(register, result);

        self.program_counter += 2;
    }

    fn bit_r(&mut self, bit: u8, register: GeneralRegister) {
        let value = self.registers.get(register);
        self.bit_inner(bit, value);

        self.program_counter += 2;
    }

    fn bit_mem_rr(&mut self, mbc: &MBC, bit: u8, register: CombinedRegister) {
        let value = mbc.read(self.registers.get16(register).into());

        self.bit_inner(bit, value);

        self.program_counter += 2;
    }
}

impl Cpu {
    fn pop_stack(&mut self, mbc: &MBC) -> u16 {
        let low_byte = mbc.read(self.stack_pointer.into());
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        println!("Low Byte: {:#06x}", low_byte);
        let high_byte = mbc.read(self.stack_pointer.into());
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        println!("High Byte: {:#06x}", high_byte);

        u8s_to_u16(high_byte, low_byte)
    }

    fn push_stack(&mut self, mbc: &mut MBC, value: u16) {
        let [high, low] = u16_to_u8s(value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        mbc.write(self.stack_pointer.into(), high);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        mbc.write(self.stack_pointer.into(), low);
    }

    fn carry_val(&self) -> u8 {
        self.registers.is_flag_set(FlagRegisterValue::C) as u8
    }

    /// Represents an unused CPU instruction. Not to be confused with NOP
    fn nothing(&mut self) {
        panic!("Invalid CPU Instruction");
    }

    fn not_implemented(&mut self, message: &str) {
        // Hunting for Not Implementeds now
        panic!("TODO: {}", message);
    }

    fn add_inner(&mut self, value: u8) {
        let a_value = self.registers.get(GeneralRegister::A);
        let result = a_value.wrapping_add(value);

        self.registers
            .set(GeneralRegister::A, result)
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .unset_flag(FlagRegisterValue::N)
            .toggle_flag(FlagRegisterValue::H, a_value.add_should_half_carry(value))
            .toggle_flag(FlagRegisterValue::C, a_value.add_should_carry(value));
    }

    fn adc_inner(&mut self, value: u8) {
        self.add_inner(value.wrapping_add(self.carry_val()));
    }

    fn add_16_inner(&mut self, value: u16) {
        let hl_val = self.registers.get16(CombinedRegister::HL);
        let result = hl_val.wrapping_add(value);

        self.registers
            .set16(CombinedRegister::HL, result)
            .unset_flag(FlagRegisterValue::N)
            .toggle_flag(FlagRegisterValue::H, hl_val.add_should_half_carry(value))
            .toggle_flag(FlagRegisterValue::C, hl_val.add_should_carry(value));
    }

    fn sub_inner(&mut self, value: u8) {
        let a_val = self.registers.get(GeneralRegister::A);
        let result = a_val.wrapping_sub(value);

        self.registers
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .set_flag(FlagRegisterValue::N)
            .toggle_flag(FlagRegisterValue::H, a_val.sub_should_half_carry(value))
            .toggle_flag(FlagRegisterValue::C, a_val.sub_should_carry(value))
            .set(GeneralRegister::A, result);
    }

    fn sbc_inner(&mut self, value: u8) {
        self.sub_inner(value.wrapping_add(self.carry_val()));
    }

    fn and_inner(&mut self, value: u8) {
        let a_val = self.registers.get(GeneralRegister::A);
        let result = a_val & value;

        self.registers
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .unset_flag(FlagRegisterValue::N)
            .set_flag(FlagRegisterValue::H)
            .unset_flag(FlagRegisterValue::C);
    }

    fn xor_inner(&mut self, value: u8) {
        let a_val = self.registers.get(GeneralRegister::A);
        let result = a_val ^ value;

        self.registers
            .set(GeneralRegister::A, result)
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .unset_flag(FlagRegisterValue::N)
            .unset_flag(FlagRegisterValue::H)
            .unset_flag(FlagRegisterValue::C);
    }

    fn or_inner(&mut self, value: u8) {
        let a_val = self.registers.get(GeneralRegister::A);
        let result = a_val | value;

        self.registers
            .set(GeneralRegister::A, result)
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .unset_flag(FlagRegisterValue::N)
            .unset_flag(FlagRegisterValue::H)
            .unset_flag(FlagRegisterValue::C);
    }

    fn cp_inner(&mut self, value: u8) {
        let a_val = self.registers.get(GeneralRegister::A);

        self.registers
            .toggle_flag(FlagRegisterValue::Z, a_val.wrapping_sub(value) == 0)
            .set_flag(FlagRegisterValue::N)
            .toggle_flag(FlagRegisterValue::H, a_val.sub_should_half_carry(value))
            .toggle_flag(FlagRegisterValue::C, a_val.sub_should_carry(value));
    }

    fn bit_inner(&mut self, bit: u8, value: u8) {
        self.registers
            .toggle_flag(FlagRegisterValue::Z, !value.is_bit_set(1 << bit))
            .unset_flag(FlagRegisterValue::N)
            .set_flag(FlagRegisterValue::H);
    }

    fn call(&mut self, mbc: &mut MBC) {
        let call_location = mbc.get_next_u16(self.program_counter.into());
        let [left, right] = u16_to_u8s(self.program_counter);

        self.stack_pointer = self.stack_pointer.wrapping_sub(2);

        mbc.write(self.stack_pointer.into(), left);
        mbc.write((self.stack_pointer.wrapping_add(1)).into(), right);

        self.program_counter = call_location;
    }

    fn jp_inner(&mut self, mbc: &MBC) {
        let jump_location = mbc.get_next_u16(self.program_counter.into());

        self.program_counter = jump_location;
    }

    fn jp(&mut self, mbc: &MBC, flag: Option<FlagRegisterValue>) {
        match flag {
            Some(f) => {
                if self.registers.is_flag_set(f) {
                    self.jp_inner(mbc);
                }
            }
            _ => {
                self.jp_inner(mbc);
            }
        }

        self.program_counter += 3;
    }

    fn jp_not(&mut self, mbc: &MBC, flag: Option<FlagRegisterValue>) {
        match flag {
            Some(f) => {
                if !self.registers.is_flag_set(f) {
                    self.jp_inner(mbc);
                }
            }
            _ => {
                self.jp_inner(mbc);
            }
        }

        self.program_counter += 3;
    }

    fn jr_base(&mut self, mbc: &MBC) {
        let relative_location = mbc.get_next_u8(self.program_counter.into()) as i8;

        let abs_location = self.program_counter.wrapping_add(relative_location as u16) as u16;

        // It increments to pull the relative location so we're adding 2 here
        self.program_counter = abs_location + 2;
    }

    fn jrf(&mut self, mbc: &MBC, flag: FlagRegisterValue) {
        if self.registers.is_flag_set(flag) {
            self.jr_base(mbc)
        } else {
            self.program_counter += 2;
        }
    }

    fn jrf_not(&mut self, mbc: &MBC, flag: FlagRegisterValue) {
        if self.registers.is_flag_set(flag) {
            self.program_counter += 2;
        } else {
            self.jr_base(mbc);
        }
    }

    fn rl_inner(&mut self, value: u8) -> u8 {
        let carry = self.carry_val();
        let will_carry = value.is_bit_set(1 << 7);
        let result = value.rotate_left(1) | carry;

        self.registers
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .unset_flag(FlagRegisterValue::N)
            .unset_flag(FlagRegisterValue::H)
            .toggle_flag(FlagRegisterValue::C, will_carry);

        result
    }

    fn rlc_inner(&mut self, value: u8) -> u8 {
        let will_carry = value.is_bit_set(1 << 7);
        let truncate_bit = value.is_bit_set(1 << 7) as u8;
        let result = value.rotate_left(1) | truncate_bit;

        self.registers
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .unset_flag(FlagRegisterValue::N)
            .unset_flag(FlagRegisterValue::H)
            .toggle_flag(FlagRegisterValue::C, will_carry);

        result
    }

    fn rr_inner(&mut self, value: u8) -> u8 {
        let carry = self.carry_val();
        let will_carry = value.is_bit_set(0);
        let result = value.rotate_right(1) | carry.rotate_left(7);

        self.registers
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .unset_flag(FlagRegisterValue::N)
            .unset_flag(FlagRegisterValue::H)
            .toggle_flag(FlagRegisterValue::C, will_carry);

        result
    }

    fn rrc_inner(&mut self, value: u8) -> u8 {
        let will_carry = value.is_bit_set(0);
        let truncate_bit = value.is_bit_set(0) as u8;
        let result = value.rotate_right(1) | truncate_bit.rotate_left(7);

        self.registers
            .toggle_flag(FlagRegisterValue::Z, result == 0)
            .unset_flag(FlagRegisterValue::N)
            .unset_flag(FlagRegisterValue::H)
            .toggle_flag(FlagRegisterValue::C, will_carry);

        result
    }

    fn res_r(&mut self, bit: u8, register: GeneralRegister) {
        let value = self.registers.get(register);
        value.unset_bit(1 << bit);
    }

    fn res_mem_rr(&mut self, mbc: &mut MBC, bit: u8) {
        let location: usize = self.registers.get16(CombinedRegister::HL).into();
        let value = mbc.read(location);
        let result = value.unset_bit(1 << bit);

        mbc.write(location, result);
    }

    fn prefix(&mut self, mbc: &mut MBC) {
        let op = mbc.get_next_u8(self.program_counter.into());

        match op {
            0x00 => self.rlc_r(GeneralRegister::B),
            0x01 => self.rlc_r(GeneralRegister::C),
            0x02 => self.rlc_r(GeneralRegister::D),
            0x03 => self.rlc_r(GeneralRegister::E),
            0x04 => self.rlc_r(GeneralRegister::H),
            0x05 => self.rlc_r(GeneralRegister::L),
            0x06 => self.rlc_mem_rr(mbc, CombinedRegister::HL),
            0x07 => self.rlc_r(GeneralRegister::A),
            0x08 => self.rrc_r(GeneralRegister::B),
            0x09 => self.rrc_r(GeneralRegister::C),
            0x0A => self.rrc_r(GeneralRegister::D),
            0x0B => self.rrc_r(GeneralRegister::E),
            0x0C => self.rrc_r(GeneralRegister::H),
            0x0D => self.rrc_r(GeneralRegister::L),
            0x0E => self.rrc_mem_rr(mbc, CombinedRegister::HL),
            0x0F => self.rrc_r(GeneralRegister::A),

            0x10 => self.rl_r(GeneralRegister::B),
            0x11 => self.rl_r(GeneralRegister::C),
            0x12 => self.rl_r(GeneralRegister::D),
            0x13 => self.rl_r(GeneralRegister::E),
            0x14 => self.rl_r(GeneralRegister::H),
            0x15 => self.rl_r(GeneralRegister::L),
            0x16 => self.rl_mem_rr(mbc, CombinedRegister::HL),
            0x17 => self.rl_r(GeneralRegister::A),
            0x18 => self.rr_r(GeneralRegister::B),
            0x19 => self.rr_r(GeneralRegister::C),
            0x1A => self.rr_r(GeneralRegister::D),
            0x1B => self.rr_r(GeneralRegister::E),
            0x1C => self.rr_r(GeneralRegister::H),
            0x1D => self.rr_r(GeneralRegister::L),
            0x1E => self.rr_mem_rr(mbc, CombinedRegister::HL),
            0x1F => self.rr_r(GeneralRegister::A),

            0x38 => self.srl_r(GeneralRegister::B),
            0x39 => self.srl_r(GeneralRegister::C),
            0x3A => self.srl_r(GeneralRegister::D),
            0x3B => self.srl_r(GeneralRegister::E),
            0x3C => self.srl_r(GeneralRegister::H),
            0x3D => self.srl_r(GeneralRegister::L),

            0x3F => self.srl_r(GeneralRegister::A),

            0x40 => self.bit_r(0, GeneralRegister::B),
            0x41 => self.bit_r(0, GeneralRegister::C),
            0x42 => self.bit_r(0, GeneralRegister::D),
            0x43 => self.bit_r(0, GeneralRegister::E),
            0x44 => self.bit_r(0, GeneralRegister::H),
            0x45 => self.bit_r(0, GeneralRegister::L),
            0x46 => self.bit_mem_rr(mbc, 0, CombinedRegister::HL),
            0x47 => self.bit_r(0, GeneralRegister::A),
            0x48 => self.bit_r(1, GeneralRegister::B),
            0x49 => self.bit_r(1, GeneralRegister::C),
            0x4A => self.bit_r(1, GeneralRegister::D),
            0x4B => self.bit_r(1, GeneralRegister::E),
            0x4C => self.bit_r(1, GeneralRegister::H),
            0x4D => self.bit_r(1, GeneralRegister::L),
            0x4E => self.bit_mem_rr(mbc, 1, CombinedRegister::HL),
            0x4F => self.bit_r(1, GeneralRegister::A),

            0x50 => self.bit_r(2, GeneralRegister::B),
            0x51 => self.bit_r(2, GeneralRegister::C),
            0x52 => self.bit_r(2, GeneralRegister::D),
            0x53 => self.bit_r(2, GeneralRegister::E),
            0x54 => self.bit_r(2, GeneralRegister::H),
            0x55 => self.bit_r(2, GeneralRegister::L),
            0x56 => self.bit_mem_rr(mbc, 2, CombinedRegister::HL),
            0x57 => self.bit_r(2, GeneralRegister::A),
            0x58 => self.bit_r(3, GeneralRegister::B),
            0x59 => self.bit_r(3, GeneralRegister::C),
            0x5A => self.bit_r(3, GeneralRegister::D),
            0x5B => self.bit_r(3, GeneralRegister::E),
            0x5C => self.bit_r(3, GeneralRegister::H),
            0x5D => self.bit_r(3, GeneralRegister::L),
            0x5E => self.bit_mem_rr(mbc, 3, CombinedRegister::HL),
            0x5F => self.bit_r(3, GeneralRegister::A),

            0x60 => self.bit_r(4, GeneralRegister::B),
            0x61 => self.bit_r(4, GeneralRegister::C),
            0x62 => self.bit_r(4, GeneralRegister::D),
            0x63 => self.bit_r(4, GeneralRegister::E),
            0x64 => self.bit_r(4, GeneralRegister::H),
            0x65 => self.bit_r(4, GeneralRegister::L),
            0x66 => self.bit_mem_rr(mbc, 4, CombinedRegister::HL),
            0x67 => self.bit_r(4, GeneralRegister::A),
            0x68 => self.bit_r(5, GeneralRegister::B),
            0x69 => self.bit_r(5, GeneralRegister::C),
            0x6A => self.bit_r(5, GeneralRegister::D),
            0x6B => self.bit_r(5, GeneralRegister::E),
            0x6C => self.bit_r(5, GeneralRegister::H),
            0x6D => self.bit_r(5, GeneralRegister::L),
            0x6E => self.bit_mem_rr(mbc, 5, CombinedRegister::HL),
            0x6F => self.bit_r(5, GeneralRegister::A),

            0x70 => self.bit_r(6, GeneralRegister::B),
            0x71 => self.bit_r(6, GeneralRegister::C),
            0x72 => self.bit_r(6, GeneralRegister::D),
            0x73 => self.bit_r(6, GeneralRegister::E),
            0x74 => self.bit_r(6, GeneralRegister::H),
            0x75 => self.bit_r(6, GeneralRegister::L),
            0x76 => self.bit_mem_rr(mbc, 6, CombinedRegister::HL),
            0x77 => self.bit_r(6, GeneralRegister::A),
            0x78 => self.bit_r(7, GeneralRegister::B),
            0x79 => self.bit_r(7, GeneralRegister::C),
            0x7A => self.bit_r(7, GeneralRegister::D),
            0x7B => self.bit_r(7, GeneralRegister::E),
            0x7C => self.bit_r(7, GeneralRegister::H),
            0x7D => self.bit_r(7, GeneralRegister::L),
            0x7E => self.bit_mem_rr(mbc, 7, CombinedRegister::HL),
            0x7F => self.bit_r(7, GeneralRegister::A),

            0x80 => self.res_r(0, GeneralRegister::B),
            0x81 => self.res_r(0, GeneralRegister::C),
            0x82 => self.res_r(0, GeneralRegister::D),
            0x83 => self.res_r(0, GeneralRegister::E),
            0x84 => self.res_r(0, GeneralRegister::H),
            0x85 => self.res_r(0, GeneralRegister::L),
            0x86 => self.res_mem_rr(mbc, 0),
            0x87 => self.res_r(0, GeneralRegister::A),
            0x88 => self.res_r(1, GeneralRegister::B),
            0x89 => self.res_r(1, GeneralRegister::C),
            0x8A => self.res_r(1, GeneralRegister::D),
            0x8B => self.res_r(1, GeneralRegister::E),
            0x8C => self.res_r(1, GeneralRegister::H),
            0x8D => self.res_r(1, GeneralRegister::L),
            0x8E => self.res_mem_rr(mbc, 1),
            0x8F => self.res_r(1, GeneralRegister::A),

            0x90 => self.res_r(2, GeneralRegister::B),
            0x91 => self.res_r(2, GeneralRegister::C),
            0x92 => self.res_r(2, GeneralRegister::D),
            0x93 => self.res_r(2, GeneralRegister::E),
            0x94 => self.res_r(2, GeneralRegister::H),
            0x95 => self.res_r(2, GeneralRegister::L),
            0x96 => self.res_mem_rr(mbc, 2),
            0x97 => self.res_r(2, GeneralRegister::A),
            0x98 => self.res_r(3, GeneralRegister::B),
            0x99 => self.res_r(3, GeneralRegister::C),
            0x9A => self.res_r(3, GeneralRegister::D),
            0x9B => self.res_r(3, GeneralRegister::E),
            0x9C => self.res_r(3, GeneralRegister::H),
            0x9D => self.res_r(3, GeneralRegister::L),
            0x9E => self.res_mem_rr(mbc, 3),
            0x9F => self.res_r(3, GeneralRegister::A),

            0xA0 => self.res_r(4, GeneralRegister::B),
            0xA1 => self.res_r(4, GeneralRegister::C),
            0xA2 => self.res_r(4, GeneralRegister::D),
            0xA3 => self.res_r(4, GeneralRegister::E),
            0xA4 => self.res_r(4, GeneralRegister::H),
            0xA5 => self.res_r(4, GeneralRegister::L),
            0xA6 => self.res_mem_rr(mbc, 4),
            0xA7 => self.res_r(4, GeneralRegister::A),
            0xA8 => self.res_r(5, GeneralRegister::B),
            0xA9 => self.res_r(5, GeneralRegister::C),
            0xAA => self.res_r(5, GeneralRegister::D),
            0xAB => self.res_r(5, GeneralRegister::E),
            0xAC => self.res_r(5, GeneralRegister::H),
            0xAD => self.res_r(5, GeneralRegister::L),
            0xAE => self.res_mem_rr(mbc, 5),
            0xAF => self.res_r(5, GeneralRegister::A),

            0xB0 => self.res_r(6, GeneralRegister::B),
            0xB1 => self.res_r(6, GeneralRegister::C),
            0xB2 => self.res_r(6, GeneralRegister::D),
            0xB3 => self.res_r(6, GeneralRegister::E),
            0xB4 => self.res_r(6, GeneralRegister::H),
            0xB5 => self.res_r(6, GeneralRegister::L),
            0xB6 => self.res_mem_rr(mbc, 6),
            0xB7 => self.res_r(6, GeneralRegister::A),
            0xB8 => self.res_r(7, GeneralRegister::B),
            0xB9 => self.res_r(7, GeneralRegister::C),
            0xBA => self.res_r(7, GeneralRegister::D),
            0xBB => self.res_r(7, GeneralRegister::E),
            0xBC => self.res_r(7, GeneralRegister::H),
            0xBD => self.res_r(7, GeneralRegister::L),
            0xBE => self.res_mem_rr(mbc, 7),
            0xBF => self.res_r(7, GeneralRegister::A),

            _ => self.not_implemented(&format!("Prefix not implemented: {:#04x}", op)),
        }

        self.program_counter += 2;
    }

    pub fn apply_operation(&mut self, mbc: &mut MBC) {
        self.current_op = mbc.read(self.program_counter.into());
        self.count += 1;

        println!(
            "[{}] ({:#06x}) {:#04x} AF={:#06x} BC={:#06x} DE={:#06x} HL={:#06x} SP={:#06x}",
            self.count,
            self.program_counter,
            self.current_op,
            self.registers.get16(CombinedRegister::AF),
            self.registers.get16(CombinedRegister::BC),
            self.registers.get16(CombinedRegister::DE),
            self.registers.get16(CombinedRegister::HL),
            self.stack_pointer,
        );

        match self.current_op {
            0x00 => self.nop(),
            0x01 => self.ld_rr_d16(mbc, CombinedRegister::BC),
            0x02 => self.ld_mem_rr_r(mbc, CombinedRegister::BC, GeneralRegister::A),
            0x03 => self.inc_rr(CombinedRegister::BC),
            0x04 => self.inc_r(GeneralRegister::B),
            0x05 => self.dec_r(GeneralRegister::B),
            0x06 => self.ld_r_d8(mbc, GeneralRegister::B),
            0x07 => self.rlca(),
            0x08 => self.ld_mem_a16_sp(mbc),
            0x09 => self.add_rr(CombinedRegister::BC),
            0x0A => self.ld_r_mem_rr(mbc, GeneralRegister::A, CombinedRegister::BC),
            0x0B => self.dec_rr(CombinedRegister::BC),
            0x0C => self.inc_r(GeneralRegister::C),
            0x0D => self.dec_r(GeneralRegister::C),
            0x0E => self.ld_r_d8(mbc, GeneralRegister::C),
            0x0F => self.rrca(),

            0x10 => self.stop(),
            0x11 => self.ld_rr_d16(mbc, CombinedRegister::DE),
            0x12 => self.ld_mem_rr_r(mbc, CombinedRegister::DE, GeneralRegister::A),
            0x13 => self.inc_rr(CombinedRegister::DE),
            0x14 => self.inc_r(GeneralRegister::D),
            0x15 => self.dec_r(GeneralRegister::D),
            0x16 => self.ld_r_d8(mbc, GeneralRegister::D),
            0x17 => self.rla(),
            0x18 => self.jr_base(mbc),
            0x19 => self.add_rr(CombinedRegister::DE),
            0x1A => self.ld_r_mem_rr(mbc, GeneralRegister::A, CombinedRegister::DE),
            0x1B => self.dec_rr(CombinedRegister::DE),
            0x1C => self.inc_r(GeneralRegister::E),
            0x1D => self.dec_r(GeneralRegister::E),
            0x1E => self.ld_r_d8(mbc, GeneralRegister::E),
            0x1F => self.rra(),

            0x20 => self.jrf_not(mbc, FlagRegisterValue::Z),
            0x21 => self.ld_rr_d16(mbc, CombinedRegister::HL),
            0x22 => self.ldi_mem_rr_r(mbc, CombinedRegister::HL, GeneralRegister::A),
            0x23 => self.inc_rr(CombinedRegister::HL),
            0x24 => self.inc_r(GeneralRegister::H),
            0x25 => self.dec_r(GeneralRegister::H),
            0x26 => self.ld_r_d8(mbc, GeneralRegister::H),
            0x27 => self.not_implemented("DAA"),
            0x28 => self.jrf(mbc, FlagRegisterValue::Z),
            0x29 => self.add_rr(CombinedRegister::HL),
            0x2A => self.ldi_r_mem_rr(mbc, GeneralRegister::A, CombinedRegister::HL),
            0x2B => self.dec_rr(CombinedRegister::HL),
            0x2C => self.inc_r(GeneralRegister::L),
            0x2D => self.dec_r(GeneralRegister::L),
            0x2E => self.ld_r_d8(mbc, GeneralRegister::L),
            0x2F => self.cpl(),

            0x30 => self.jrf_not(mbc, FlagRegisterValue::C),
            0x31 => self.ld_sp_d16(mbc),
            0x32 => self.ldd_mem_rr_r(mbc, CombinedRegister::HL, GeneralRegister::A),
            0x33 => self.inc_sp(),
            0x34 => self.inc_mem_rr(mbc, CombinedRegister::HL),
            0x35 => self.inc_mem_rr(mbc, CombinedRegister::HL),
            0x36 => self.ld_mem_rr_d8(mbc, CombinedRegister::HL),
            0x37 => self.not_implemented("SCF"),
            0x38 => self.jrf(mbc, FlagRegisterValue::C),
            0x39 => self.add_sp(),
            0x3A => self.ldd_r_mem_rr(mbc, GeneralRegister::A, CombinedRegister::HL),
            0x3B => self.dec_sp(),
            0x3C => self.inc_r(GeneralRegister::A),
            0x3D => self.dec_r(GeneralRegister::A),
            0x3E => self.ld_r_d8(mbc, GeneralRegister::A),
            0x3F => self.ccf(),

            0x40 => self.ld_r_r(GeneralRegister::B, GeneralRegister::B),
            0x41 => self.ld_r_r(GeneralRegister::B, GeneralRegister::C),
            0x42 => self.ld_r_r(GeneralRegister::B, GeneralRegister::D),
            0x43 => self.ld_r_r(GeneralRegister::B, GeneralRegister::E),
            0x44 => self.ld_r_r(GeneralRegister::B, GeneralRegister::H),
            0x45 => self.ld_r_r(GeneralRegister::B, GeneralRegister::L),
            0x46 => self.ld_r_mem_rr(mbc, GeneralRegister::B, CombinedRegister::HL),
            0x47 => self.ld_r_r(GeneralRegister::B, GeneralRegister::A),
            0x48 => self.ld_r_r(GeneralRegister::C, GeneralRegister::B),
            0x49 => self.ld_r_r(GeneralRegister::C, GeneralRegister::C),
            0x4A => self.ld_r_r(GeneralRegister::C, GeneralRegister::D),
            0x4B => self.ld_r_r(GeneralRegister::C, GeneralRegister::E),
            0x4C => self.ld_r_r(GeneralRegister::C, GeneralRegister::H),
            0x4D => self.ld_r_r(GeneralRegister::C, GeneralRegister::L),
            0x4E => self.ld_r_mem_rr(mbc, GeneralRegister::C, CombinedRegister::HL),
            0x4F => self.ld_r_r(GeneralRegister::C, GeneralRegister::A),

            0x50 => self.ld_r_r(GeneralRegister::D, GeneralRegister::B),
            0x51 => self.ld_r_r(GeneralRegister::D, GeneralRegister::C),
            0x52 => self.ld_r_r(GeneralRegister::D, GeneralRegister::D),
            0x53 => self.ld_r_r(GeneralRegister::D, GeneralRegister::E),
            0x54 => self.ld_r_r(GeneralRegister::D, GeneralRegister::H),
            0x55 => self.ld_r_r(GeneralRegister::D, GeneralRegister::L),
            0x56 => self.ld_r_mem_rr(mbc, GeneralRegister::D, CombinedRegister::HL),
            0x57 => self.ld_r_r(GeneralRegister::D, GeneralRegister::A),
            0x58 => self.ld_r_r(GeneralRegister::E, GeneralRegister::B),
            0x59 => self.ld_r_r(GeneralRegister::E, GeneralRegister::C),
            0x5A => self.ld_r_r(GeneralRegister::E, GeneralRegister::D),
            0x5B => self.ld_r_r(GeneralRegister::E, GeneralRegister::E),
            0x5C => self.ld_r_r(GeneralRegister::E, GeneralRegister::H),
            0x5D => self.ld_r_r(GeneralRegister::E, GeneralRegister::L),
            0x5E => self.ld_r_mem_rr(mbc, GeneralRegister::E, CombinedRegister::HL),
            0x5F => self.ld_r_r(GeneralRegister::E, GeneralRegister::A),

            0x60 => self.ld_r_r(GeneralRegister::H, GeneralRegister::B),
            0x61 => self.ld_r_r(GeneralRegister::H, GeneralRegister::C),
            0x62 => self.ld_r_r(GeneralRegister::H, GeneralRegister::D),
            0x63 => self.ld_r_r(GeneralRegister::H, GeneralRegister::E),
            0x64 => self.ld_r_r(GeneralRegister::H, GeneralRegister::H),
            0x65 => self.ld_r_r(GeneralRegister::H, GeneralRegister::L),
            0x66 => self.ld_r_mem_rr(mbc, GeneralRegister::H, CombinedRegister::HL),
            0x67 => self.ld_r_r(GeneralRegister::H, GeneralRegister::A),
            0x68 => self.ld_r_r(GeneralRegister::L, GeneralRegister::B),
            0x69 => self.ld_r_r(GeneralRegister::L, GeneralRegister::C),
            0x6A => self.ld_r_r(GeneralRegister::L, GeneralRegister::D),
            0x6B => self.ld_r_r(GeneralRegister::L, GeneralRegister::E),
            0x6C => self.ld_r_r(GeneralRegister::L, GeneralRegister::H),
            0x6D => self.ld_r_r(GeneralRegister::L, GeneralRegister::L),
            0x6E => self.ld_r_mem_rr(mbc, GeneralRegister::L, CombinedRegister::HL),
            0x6F => self.ld_r_r(GeneralRegister::L, GeneralRegister::A),

            0x70 => self.ld_mem_rr_r(mbc, CombinedRegister::HL, GeneralRegister::B),
            0x71 => self.ld_mem_rr_r(mbc, CombinedRegister::HL, GeneralRegister::C),
            0x72 => self.ld_mem_rr_r(mbc, CombinedRegister::HL, GeneralRegister::D),
            0x73 => self.ld_mem_rr_r(mbc, CombinedRegister::HL, GeneralRegister::E),
            0x74 => self.ld_mem_rr_r(mbc, CombinedRegister::HL, GeneralRegister::H),
            0x75 => self.ld_mem_rr_r(mbc, CombinedRegister::HL, GeneralRegister::L),
            0x76 => self.not_implemented("HALT"),
            0x77 => self.ld_mem_rr_r(mbc, CombinedRegister::HL, GeneralRegister::A),
            0x78 => self.ld_r_r(GeneralRegister::A, GeneralRegister::B),
            0x79 => self.ld_r_r(GeneralRegister::A, GeneralRegister::C),
            0x7A => self.ld_r_r(GeneralRegister::A, GeneralRegister::D),
            0x7B => self.ld_r_r(GeneralRegister::A, GeneralRegister::E),
            0x7C => self.ld_r_r(GeneralRegister::A, GeneralRegister::H),
            0x7D => self.ld_r_r(GeneralRegister::A, GeneralRegister::L),
            0x7E => self.ld_r_mem_rr(mbc, GeneralRegister::A, CombinedRegister::HL),
            0x7F => self.ld_r_r(GeneralRegister::A, GeneralRegister::A),

            0x80 => self.add_r(GeneralRegister::B),
            0x81 => self.add_r(GeneralRegister::C),
            0x82 => self.add_r(GeneralRegister::D),
            0x83 => self.add_r(GeneralRegister::E),
            0x84 => self.add_r(GeneralRegister::H),
            0x85 => self.add_r(GeneralRegister::L),
            0x86 => self.add_mem_rr(mbc, CombinedRegister::HL),
            0x87 => self.add_r(GeneralRegister::A),
            0x88 => self.adc_r(GeneralRegister::B),
            0x89 => self.adc_r(GeneralRegister::C),
            0x8A => self.adc_r(GeneralRegister::D),
            0x8B => self.adc_r(GeneralRegister::E),
            0x8C => self.adc_r(GeneralRegister::H),
            0x8D => self.adc_r(GeneralRegister::L),
            0x8E => self.adc_mem_rr(mbc, CombinedRegister::HL),
            0x8F => self.adc_r(GeneralRegister::A),

            0x90 => self.sub_r(GeneralRegister::B),
            0x91 => self.sub_r(GeneralRegister::C),
            0x92 => self.sub_r(GeneralRegister::D),
            0x93 => self.sub_r(GeneralRegister::E),
            0x94 => self.sub_r(GeneralRegister::H),
            0x95 => self.sub_r(GeneralRegister::L),
            0x96 => self.sub_mem_rr(mbc, CombinedRegister::HL),
            0x97 => self.sub_r(GeneralRegister::A),
            0x98 => self.sbc_r(GeneralRegister::B),
            0x99 => self.sbc_r(GeneralRegister::C),
            0x9A => self.sbc_r(GeneralRegister::D),
            0x9B => self.sbc_r(GeneralRegister::E),
            0x9C => self.sbc_r(GeneralRegister::H),
            0x9D => self.sbc_r(GeneralRegister::L),
            0x9E => self.sbc_mem_rr(mbc, CombinedRegister::HL),
            0x9F => self.sbc_r(GeneralRegister::A),

            0xA0 => self.and_r(GeneralRegister::B),
            0xA1 => self.and_r(GeneralRegister::C),
            0xA2 => self.and_r(GeneralRegister::D),
            0xA3 => self.and_r(GeneralRegister::E),
            0xA4 => self.and_r(GeneralRegister::H),
            0xA5 => self.and_r(GeneralRegister::L),
            0xA6 => self.and_mem_rr(mbc, CombinedRegister::HL),
            0xA7 => self.and_r(GeneralRegister::A),
            0xA8 => self.xor_r(GeneralRegister::B),
            0xA9 => self.xor_r(GeneralRegister::C),
            0xAA => self.xor_r(GeneralRegister::D),
            0xAB => self.xor_r(GeneralRegister::E),
            0xAC => self.xor_r(GeneralRegister::H),
            0xAD => self.xor_r(GeneralRegister::L),
            0xAE => self.xor_mem_rr(mbc, CombinedRegister::HL),
            0xAF => self.xor_r(GeneralRegister::A),

            0xB0 => self.or_r(GeneralRegister::B),
            0xB1 => self.or_r(GeneralRegister::C),
            0xB2 => self.or_r(GeneralRegister::D),
            0xB3 => self.or_r(GeneralRegister::E),
            0xB4 => self.or_r(GeneralRegister::H),
            0xB5 => self.or_r(GeneralRegister::L),
            0xB6 => self.or_mem_rr(mbc, CombinedRegister::HL),
            0xB7 => self.or_r(GeneralRegister::A),
            0xB8 => self.cp_r(GeneralRegister::B),
            0xB9 => self.cp_r(GeneralRegister::C),
            0xBA => self.cp_r(GeneralRegister::D),
            0xBB => self.cp_r(GeneralRegister::E),
            0xBC => self.cp_r(GeneralRegister::H),
            0xBD => self.cp_r(GeneralRegister::L),
            0xBE => self.cp_mem_rr(mbc, CombinedRegister::HL),
            0xBF => self.cp_r(GeneralRegister::A),

            0xC0 => self.ret_not(mbc, FlagRegisterValue::Z),
            0xC1 => self.pop_rr(mbc, CombinedRegister::BC),
            0xC2 => self.jp_not(mbc, Some(FlagRegisterValue::Z)),
            0xC3 => self.jp(mbc, None),
            0xC4 => self.not_implemented("CALL NZ, u16"),
            0xC5 => self.push_rr(mbc, CombinedRegister::BC),
            0xC6 => self.add_d8(mbc),
            0xC7 => self.not_implemented("RST 00h"),
            0xC8 => self.ret_if(mbc, FlagRegisterValue::Z),
            0xC9 => self.ret(mbc),
            0xCA => self.jp(mbc, Some(FlagRegisterValue::Z)),
            0xCB => self.prefix(mbc),
            0xCC => self.not_implemented("CALL Z, u16"),
            0xCD => self.call(mbc),
            0xCE => self.adc_d8(mbc),
            0xCF => self.not_implemented("RST 08h"),

            0xD0 => self.ret_not(mbc, FlagRegisterValue::C),
            0xD1 => self.pop_rr(mbc, CombinedRegister::DE),
            0xD2 => self.jp_not(mbc, Some(FlagRegisterValue::C)),
            0xD3 => self.nothing(),
            0xD4 => self.not_implemented("CALL NC, u16"),
            0xD5 => self.push_rr(mbc, CombinedRegister::DE),
            0xD6 => self.sub_d8(mbc),
            0xD7 => self.not_implemented("RST 10h"),
            0xD8 => self.ret_if(mbc, FlagRegisterValue::C),
            0xD9 => self.reti(mbc),
            0xDA => self.jp(mbc, Some(FlagRegisterValue::C)),
            0xDB => self.nothing(),
            0xDC => self.not_implemented("CALL C, u16"),
            0xDD => self.nothing(),
            0xDE => self.sbc_d8(mbc),
            0xDF => self.not_implemented("RST 18h"),

            0xE0 => self.ld_mem_a8_a(mbc),
            0xE1 => self.pop_rr(mbc, CombinedRegister::HL),
            0xE2 => self.ld_mem_r_a(mbc, GeneralRegister::C),
            0xE3 => self.nothing(),
            0xE4 => self.nothing(),
            0xE5 => self.push_rr(mbc, CombinedRegister::HL),
            0xE6 => self.and_d8(mbc),
            0xE7 => self.not_implemented("RST 20h"),
            0xE8 => self.not_implemented("ADD SP, i8"),
            0xE9 => self.not_implemented("JP HL"),
            0xEA => self.ld_mem_a16_a(mbc),
            0xEB => self.nothing(),
            0xEC => self.nothing(),
            0xED => self.nothing(),
            0xEE => self.xor_d8(mbc),
            0xEF => self.not_implemented("RST 28h"),

            0xF0 => self.ld_a_mem_a8(mbc),
            0xF1 => self.pop_rr(mbc, CombinedRegister::AF),
            0xF2 => self.ld_a_mem_r(mbc, GeneralRegister::C),
            0xF3 => self.di(),
            0xF4 => self.nothing(),
            0xF5 => self.push_rr(mbc, CombinedRegister::AF),
            0xF6 => self.or_d8(mbc),
            0xF7 => self.not_implemented("RST 30h"),
            0xF8 => self.not_implemented("LD HL, SP+i"),
            0xF9 => self.ld_sp_rr(CombinedRegister::HL),
            0xFA => self.ld_a_mem_a16(mbc),
            0xFB => self.ei(),
            0xFC => self.nothing(),
            0xFD => self.nothing(),
            0xFE => self.cp_d8(mbc),
            0xFF => self.not_implemented("RST 38h"),
        }
    }
}
