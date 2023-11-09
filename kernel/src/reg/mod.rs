//! Provides safe, strongly-typed access to registers (e.g. memory-mapped or system registers).
use core::cmp;
use core::ops;

pub mod memory_mapped;
pub mod system;

/// Useful types for implementing registers: [`RegisterSpec`] and associated marker traits, along
/// with [`RegisterReader`] and [`RegisterWriter`].
pub mod prelude {
    pub use super::RegisterSpec;

    // Markers for RegisterSpec.
    pub use super::{RegisterInitial, RegisterReadable, RegisterWritable};

    // Required to implement named bit/field accessors.
    pub use super::{RegisterReader, RegisterWriter};
}

/// Values which can be used as the underlying storage for a register.
pub trait RegisterBits:
    Copy
    + From<bool>
    + cmp::PartialEq<Self>
    + ops::BitAnd<Self, Output = Self>
    + ops::BitOr<Self, Output = Self>
    + ops::Not<Output = Self>
    + ops::Shl<usize, Output = Self>
    + ops::Shr<usize, Output = Self>
{
    /// Returns `0`.
    fn zero() -> Self;

    /// Returns a contiguous sequence of `WIDTH` bits with value `1`, beginning at the LSB.
    fn mask<const WIDTH: usize>() -> Self;
}

pub trait RegisterSpec {
    /// The type of the raw value of the register.
    type Bits: RegisterBits;
}

/// Marker for register specs (i.e. types implementing [`RegisterSpec`]) indicating that the
/// underlying register can be read from.
pub trait RegisterReadable: RegisterSpec {}

/// Marker for register specs (i.e. types implementing [`RegisterSpec`]) indicating that the
/// underlying register can be written to.
pub trait RegisterWritable: RegisterSpec {}

/// Marker for writable register specs (i.e. types implementing [`RegisterSpec`] and
/// [`RegisterWritable`]) which provides a safe, potentially non-zero initial value for the register
/// during write operations.
///
/// Writing the initial value to the register must **not** result in any undefined behaviour.
pub trait RegisterInitial: RegisterWritable {
    /// The initial value.
    const INITIAL_VALUE: Self::Bits;
}

/// Provides read access to the fields of a register.
pub struct RegisterReader<S: RegisterSpec> {
    bits: S::Bits,
}

/// Provides write access to the fields of a register.
pub struct RegisterWriter<S: RegisterSpec> {
    bits: S::Bits,
}

impl<S: RegisterSpec> RegisterReader<S> {
    fn new(bits: S::Bits) -> Self {
        Self { bits }
    }
}

impl<S: RegisterSpec> RegisterWriter<S> {
    fn zero() -> Self {
        Self {
            bits: S::Bits::zero(),
        }
    }
}

impl<S: RegisterSpec + RegisterInitial> RegisterWriter<S> {
    fn initial() -> Self {
        Self {
            bits: S::INITIAL_VALUE,
        }
    }
}

impl<S: RegisterSpec> RegisterReader<S> {
    /// Returns the raw value.
    pub fn bits(&self) -> S::Bits {
        self.bits
    }

    /// Returns the value of the bit at offset `OFFSET`.
    pub fn bit<const OFFSET: usize>(&self) -> bool {
        self.field::<OFFSET, 1>() != S::Bits::zero()
    }

    /// Returns the value of a contiguous `SIZE`-bit field with its LSB at the offset `OFFSET`.
    pub fn field<const OFFSET: usize, const SIZE: usize>(&self) -> S::Bits {
        (self.bits >> OFFSET) & S::Bits::mask::<SIZE>()
    }
}

impl<S: RegisterSpec> RegisterWriter<S> {
    /// Sets the raw value.
    ///
    /// # Safety
    /// Setting an unsupported value may result in undefined behaviour. Refer to the register's
    /// definition to determine valid values.
    pub unsafe fn bits(&mut self, bits: S::Bits) {
        self.bits = bits;
    }

    /// Sets the value of the bit at offset `OFFSET`.
    ///
    /// # Safety
    /// Setting an unsupported value may result in undefined behaviour. Refer to the register's
    /// definition to determine valid values.
    pub unsafe fn bit<const OFFSET: usize>(&mut self, bit: bool) {
        self.field::<OFFSET, 1>(bit.into());
    }

    /// Sets the value of a contiguous `SIZE`-bit field with its LSB at the offset `OFFSET`. Values
    /// larger than the field size will be masked to fit the field.
    ///
    /// # Safety
    /// Setting an unsupported value may result in undefined behaviour. Refer to the register's
    /// definition to determine valid values.
    pub unsafe fn field<const OFFSET: usize, const SIZE: usize>(&mut self, field: S::Bits) {
        let mask = S::Bits::mask::<SIZE>();

        self.bits = (self.bits & !(mask << OFFSET)) | ((field & mask) << OFFSET);
    }
}

macro_rules! register_bits {
    ($ty:ty, $width:literal) => {
        impl RegisterBits for $ty {
            fn zero() -> Self {
                0
            }

            fn mask<const WIDTH: usize>() -> Self {
                <$ty>::MAX >> ($width - WIDTH)
            }
        }

        impl RegisterSpec for $ty {
            type Bits = $ty;
        }
    };
}

register_bits!(u64, 64);
register_bits!(u32, 32);
register_bits!(u16, 16);
register_bits!(u8, 8);
