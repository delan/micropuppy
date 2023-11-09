use core::arch::asm;

use crate::reg::prelude::*;
use crate::reg::system::SystemRegisterSpec;

pub struct DAIF;

impl SystemRegisterSpec for DAIF {
    fn read() -> u64 {
        let bits;
        unsafe { asm!("mrs DAIF, {}", out(reg) bits) };
        bits
    }

    fn write(bits: u64) {
        unsafe { asm!("msr DAIF, {}", in(reg) bits) };
    }
}

impl RegisterReadable for DAIF {}

impl RegisterWritable for DAIF {}

impl RegisterInitial for DAIF {
    const INITIAL_VALUE: u64 = 0x3c0;
}

impl RegisterReader<DAIF> {
    pub fn d(&self) -> bool {
        self.bit::<9>()
    }

    pub fn a(&self) -> bool {
        self.bit::<8>()
    }

    pub fn i(&self) -> bool {
        self.bit::<7>()
    }

    pub fn f(&self) -> bool {
        self.bit::<6>()
    }
}

impl RegisterWriter<DAIF> {
    pub fn d(&mut self, d: bool) {
        unsafe { self.bit::<9>(d) }
    }

    pub fn a(&mut self, a: bool) {
        unsafe { self.bit::<8>(a) }
    }

    pub fn i(&mut self, i: bool) {
        unsafe { self.bit::<7>(i) }
    }

    pub fn f(&mut self, f: bool) {
        unsafe { self.bit::<6>(f) }
    }
}
