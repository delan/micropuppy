//! Memory-mapped registers.
use vcell::VolatileCell;

use super::*;

/// A memory-mapped register which owns its value.
///
/// [`Register`] is `#[repr(transparent)]` so that it can be placed in a `#[repr(C)]` struct
/// matching a peripheral's memory layout. Casting a value to a pointer to the register block allows
/// simple and safe access to a peripheral's entire register set.
#[repr(transparent)]
pub struct Register<S: RegisterSpec>(VolatileCell<S::Bits>);

pub type PaddingBytes<const BYTES: usize> = [Register<u8>; BYTES];

impl<S: RegisterSpec + RegisterReadable> Register<S> {
    /// Reads the current value of the register, providing access through an instance of
    /// [`RegisterReader`].
    ///
    /// The register's value is only read once: when `read` is called. This is the value accessed
    /// through bit or field getters on [`RegisterReader`] inside the `reader` closure.
    ///
    /// The return value of the `reader` closure is returned by `read`.
    pub fn read<R>(&self, reader: impl FnOnce(&RegisterReader<S>) -> R) -> R {
        let r = RegisterReader::new(self.0.get());
        reader(&r)
    }
}

impl<S: RegisterSpec + RegisterWritable> Register<S> {
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
        self.0.set(w.bits);
    }
}

impl<S: RegisterSpec + RegisterInitial> Register<S> {
    /// Writes a value built by an instance of [`RegisterWriter`], initialised to the register's
    /// initial value (provided by [`RegisterInitial`]), to the register.
    ///
    /// The register's value is only written once: when the `writer` closure returns. The value
    /// written is the value built through bit or field setters on [`RegisterWriter`] inside the
    /// `writer` closure.
    pub fn write_initial(&self, writer: impl FnOnce(&mut RegisterWriter<S>)) {
        let mut w = RegisterWriter::initial();
        writer(&mut w);
        self.0.set(w.bits);
    }
}

#[macro_export]
macro_rules! memory_mapped_register {
    { $name:ident($bits:ty) } => {
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl RegisterSpec for $name {
            type Bits = $bits;
        }
    };
    { $name:ident($bits:ty), r } => {
        reg!($name, $bits);

        impl RegisterReadable for $name {}
    };
    { $name:ident($bits:ty), w } => {
        reg!($name, $bits);

        impl RegisterWritable for $name {}
    };
    { $name:ident($bits:ty), wi=$initial:literal } => {
        reg!($name($bits));

        impl RegisterWritable for $name {}
        impl RegisterInitial for $name {
            const INITIAL_VALUE: Self::Bits = $initial;
        }
    };
    { $name:ident($bits:ty), rw } => {
        reg!($name, $bits);

        impl RegisterReadable for $name {}
        impl RegisterWritable for $name {}
    };
    { $name:ident($bits:ty), rwi=$initial:literal } => {
        reg!($name($bits));

        impl RegisterReadable for $name {}
        impl RegisterWritable for $name {}
        impl RegisterInitial for $name {
            const INITIAL_VALUE: Self::Bits = $initial;
        }
    };
}
