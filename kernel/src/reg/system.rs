use core::marker::PhantomData;

use super::*;

pub trait SystemRegisterSpec {
    // HACK: using constant strings in asm! is hard, it seems
    fn read() -> u64;
    fn write(bits: u64);
}

impl<T: SystemRegisterSpec> RegisterSpec for T {
    type Bits = u64;
}

pub struct Register<S: SystemRegisterSpec> {
    phantom: PhantomData<S>,
}

impl<S: SystemRegisterSpec> Register<S> {
    // HACK: shouldn't exist
    pub fn new() -> Self {
        Self {
            phantom: Default::default(),
        }
    }
}

impl<S: SystemRegisterSpec + RegisterReadable> Register<S>
where
    S: RegisterSpec<Bits = u64>,
{
    pub fn read<R>(&self, reader: impl FnOnce(&RegisterReader<S>) -> R) -> R {
        let r = RegisterReader::new(S::read());
        reader(&r)
    }
}

impl<S: SystemRegisterSpec + RegisterWritable> Register<S>
where
    S: RegisterSpec<Bits = u64>,
{
    pub unsafe fn write_zero(&self, writer: impl FnOnce(&mut RegisterWriter<S>)) {
        let mut w = RegisterWriter::zero();
        writer(&mut w);
        S::write(w.bits);
    }
}

impl<S: SystemRegisterSpec + RegisterWritable + RegisterDefault> Register<S>
where
    S: RegisterSpec<Bits = u64>,
{
    pub fn write_default(&self, writer: impl FnOnce(&mut RegisterWriter<S>)) {
        let mut w = RegisterWriter::default();
        writer(&mut w);
        S::write(w.bits);
    }
}
