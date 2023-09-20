use crate::cpu_registers::{CombinedRegister, GeneralRegister};
use crate::flag_register::FlagRegisterValue;
use crate::mbc::MBC;

pub trait Ops {
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
