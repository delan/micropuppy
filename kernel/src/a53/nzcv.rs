use core::arch::asm;

use crate::reg::prelude::*;
use crate::reg::system::SystemRegisterSpec;

#[allow(clippy::upper_case_acronyms)]
pub struct NZCV;

impl SystemRegisterSpec for NZCV {
    unsafe fn mrs() -> u64 {
        let bits: u64;
        asm!("mrs {}, NZCV", out(reg) bits);
        bits
    }

    unsafe fn msr(bits: u64) {
        asm!("msr NZCV, {}", in(reg) bits);
    }
}

impl RegisterReadable for NZCV {}

impl RegisterWritable for NZCV {}

#[allow(dead_code)]
impl RegisterReader<NZCV> {
    pub fn n(&self) -> bool {
        self.bit(31)
    }
    pub fn z(&self) -> bool {
        self.bit(30)
    }
    pub fn c(&self) -> bool {
        self.bit(29)
    }
    pub fn v(&self) -> bool {
        self.bit(28)
    }
}

#[allow(dead_code)]
impl RegisterWriter<NZCV> {
    pub fn n(&mut self, n: bool) {
        unsafe { self.bit(31, n) }
    }
    pub fn z(&mut self, z: bool) {
        unsafe { self.bit(30, z) }
    }
    pub fn c(&mut self, c: bool) {
        unsafe { self.bit(29, c) }
    }
    pub fn v(&mut self, v: bool) {
        unsafe { self.bit(28, v) }
    }
}
