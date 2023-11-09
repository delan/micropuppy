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

/// An AArch64 system register accessed with the `mrs` and `msr` assembly instructions.
pub struct Register<S: SystemRegisterSpec>(PhantomData<S>);

impl<S: SystemRegisterSpec> Register<S> {
    // HACK: shouldn't exist
    pub fn new() -> Self {
        Self(Default::default())
    }
}

impl<S: SystemRegisterSpec + RegisterReadable> Register<S>
where
    S: RegisterSpec<Bits = u64>,
{
    /// Reads the current value of the register, providing access through an instance of
    /// [`RegisterReader`].
    ///
    /// The register's value is only read once: when `read` is called. This is the value accessed
    /// through bit or field getters on [`RegisterReader`] inside the `reader` closure.
    ///
    /// The return value of the `reader` closure is returned by `read`.
    pub fn read<R>(&self, reader: impl FnOnce(&RegisterReader<S>) -> R) -> R {
        let r = RegisterReader::new(S::read());
        reader(&r)
    }
}

impl<S: SystemRegisterSpec + RegisterWritable> Register<S>
where
    S: RegisterSpec<Bits = u64>,
{
    /// Writes a value built by an instance of [`RegisterWriter`], initialised to zero, to the
    /// register.
    ///
    /// The register's value is only written once: when the `writer` closure returns. The value
    /// written is the value built through bit or field setters on [`RegisterWriter`] inside the
    /// `writer` closure.
    ///
    /// # Safety
    /// Setting bits or fields to zero may result in undefined behaviour, as zero is not guaranteed
    /// to be a supported value. Refer to the register's definition to determine valid values.
    pub unsafe fn write_zero(&self, writer: impl FnOnce(&mut RegisterWriter<S>)) {
        let mut w = RegisterWriter::zero();
        writer(&mut w);
        S::write(w.bits);
    }
}

impl<S: SystemRegisterSpec + RegisterWritable + RegisterInitial> Register<S>
where
    S: RegisterSpec<Bits = u64>,
{
    /// Writes a value built by an instance of [`RegisterWriter`], initialised to the register's
    /// initial value (provided by [`RegisterInitial`]), to the register.
    ///
    /// The register's value is only written once: when the `writer` closure returns. The value
    /// written is the value built through bit or field setters on [`RegisterWriter`] inside the
    /// `writer` closure.
    pub fn write_initial(&self, writer: impl FnOnce(&mut RegisterWriter<S>)) {
        let mut w = RegisterWriter::initial();
        writer(&mut w);
        S::write(w.bits);
    }
}
