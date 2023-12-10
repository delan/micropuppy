use core::arch::asm;

use crate::reg::prelude::*;
use crate::reg::system::SystemRegisterSpec;

#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
pub struct SPSR_EL1;

impl SystemRegisterSpec for SPSR_EL1 {
    unsafe fn mrs() -> u64 {
        let bits: u64;
        asm!("mrs {}, SPSR_EL1", out(reg) bits);
        bits
    }

    unsafe fn msr(bits: u64) {
        asm!("msr SPSR_EL1, {}", in(reg) bits);
    }
}

impl RegisterReadable for SPSR_EL1 {}

impl RegisterWritable for SPSR_EL1 {}

impl RegisterReader<SPSR_EL1> {
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

impl RegisterWriter<SPSR_EL1> {
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
