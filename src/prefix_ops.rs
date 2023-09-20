use crate::cpu_registers::{CombinedRegister, GeneralRegister};
use crate::mbc::MBC;

pub trait PrefixOps {
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
