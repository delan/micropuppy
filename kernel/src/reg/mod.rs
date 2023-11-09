//! Provides safe, strongly-typed access to registers (e.g. memory-mapped or system registers).
use core::cmp;
use core::ops::{self, RangeInclusive};

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

    /// Returns a contiguous sequence of `width` bits with value `1`, beginning at the LSB.
    fn mask(width: usize) -> Self;
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

    /// Returns the value of the bit at offset `offset`.
    pub fn bit(&self, offset: usize) -> bool {
        self.field(offset..=offset) != S::Bits::zero()
    }

    /// Returns the value of a contiguous bit field with its LSB at the offset `range.start()` and
    /// MSB at the offset `range.end()`.
    ///
    /// ```text
    ///   bits: | ... | |x|x|x| ... |x|x| | ... | | | | |
    ///                  ^             ^               ^
    /// offset:          range.end()   range.start()   0
    /// ```
    pub fn field(&self, range: RangeInclusive<usize>) -> S::Bits {
        let FieldSpec { offset, size } = range.field_spec();

        (self.bits >> offset) & S::Bits::mask(size)
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

    /// Sets the value of the bit at offset `offset`.
    ///
    /// # Safety
    /// Setting an unsupported value may result in undefined behaviour. Refer to the register's
    /// definition to determine valid values.
    pub unsafe fn bit(&mut self, offset: usize, bit: bool) {
        self.field(offset..=offset, bit.into());
    }

    /// Sets the value of a contiguous bit field with its LSB at the offset `range.start()` and MSB
    /// at the offset `range.end()`.
    ///
    /// ```text
    ///   bits: | ... | |x|x|x| ... |x|x| | ... | | | | |
    ///                  ^             ^               ^
    /// offset:          range.end()   range.start()   0
    /// ```
    ///
    /// # Safety
    /// Setting an unsupported value may result in undefined behaviour. Refer to the register's
    /// definition to determine valid values.
    pub unsafe fn field(&mut self, range: RangeInclusive<usize>, field: S::Bits) {
        let FieldSpec { offset, size } = range.field_spec();
        let mask = S::Bits::mask(size);

        self.bits = (self.bits & !(mask << offset)) | ((field & mask) << offset);
    }
}

macro_rules! register_bits {
    ($ty:ty, $width:literal) => {
        impl RegisterBits for $ty {
            fn zero() -> Self {
                0
            }

            fn mask(width: usize) -> Self {
                <$ty>::MAX >> ($width - width)
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

/// Offset and size of a field.
struct FieldSpec {
    /// Offset of the LSB of this field.
    offset: usize,
    /// Number of bits within this field.
    size: usize,
}

/// Extension trait to provide offset and size values from a [`RangeInclusive`].
trait RangeInclusiveExt {
    /// Field offset and size represented by this [`RangeInclusive`].
    fn field_spec(&self) -> FieldSpec;
}

impl RangeInclusiveExt for RangeInclusive<usize> {
    fn field_spec(&self) -> FieldSpec {
        FieldSpec {
            offset: *self.start(),
            size: self.end() + 1 - self.start(),
        }
    }
}
