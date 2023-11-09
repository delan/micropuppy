use core::marker::PhantomData;

use super::*;

pub trait SystemRegisterSpec {
    // HACK: using constant strings in asm! is hard, it seems
    fn read() -> u64;
    fn write(bits: u64);
}

impl<T: SystemRegisterSpec> RegisterSpec for T {
    type Type = u64;
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
    S: RegisterSpec<Type = u64>,
{
    pub fn read<R>(&self, reader: impl FnOnce(&RegisterRead<S>) -> R) -> R {
        let r = RegisterRead::new(S::read());
        reader(&r)
    }
}

impl<S: SystemRegisterSpec + RegisterWritable> Register<S>
where
    S: RegisterSpec<Type = u64>,
{
    pub unsafe fn write_zero(&self, writer: impl FnOnce(&mut RegisterWrite<S>)) {
        let mut w = RegisterWrite::zero();
        writer(&mut w);
        S::write(w.bits);
    }
}

impl<S: SystemRegisterSpec + RegisterWritable + RegisterDefault> Register<S>
where
    S: RegisterSpec<Type = u64>,
{
    pub fn write_default(&self, writer: impl FnOnce(&mut RegisterWrite<S>)) {
        let mut w = RegisterWrite::default();
        writer(&mut w);
        S::write(w.bits);
    }
}
