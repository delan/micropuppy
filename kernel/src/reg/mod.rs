use core::cmp;
use core::ops;

pub mod memory_mapped;
pub mod system;

pub trait RegisterType:
    Copy
    + Default
    + From<bool>
    + cmp::PartialEq<Self>
    + ops::BitAnd<Self, Output = Self>
    + ops::BitOr<Self, Output = Self>
    + ops::Not<Output = Self>
    + ops::Shl<usize, Output = Self>
    + ops::Shr<usize, Output = Self>
{
    fn zero() -> Self;
    fn mask<const WIDTH: usize>() -> Self;
}

pub trait RegisterSpec {
    type Type: RegisterType;
}

pub trait RegisterReadable: RegisterSpec {}

pub trait RegisterWritable: RegisterSpec {}

pub trait RegisterDefault: RegisterSpec {
    const DEFAULT_VALUE: Self::Type;
}

pub struct RegisterRead<S: RegisterSpec> {
    pub bits: S::Type,
}

impl<S: RegisterSpec> RegisterRead<S> {
    fn new(bits: S::Type) -> Self {
        Self { bits }
    }

    pub fn bits(&self) -> S::Type {
        self.bits
    }

    pub fn bit<const OFFSET: usize>(&self) -> bool {
        self.field::<OFFSET, 1>() != S::Type::zero()
    }

    pub fn field<const OFFSET: usize, const SIZE: usize>(&self) -> S::Type {
        (self.bits >> OFFSET) & S::Type::mask::<SIZE>()
    }
}

pub struct RegisterWrite<S: RegisterSpec> {
    pub bits: S::Type,
}

impl<S: RegisterSpec> RegisterWrite<S> {
    fn zero() -> Self {
        Self {
            bits: Default::default(),
        }
    }
}

impl<S: RegisterSpec + RegisterDefault> RegisterWrite<S> {
    fn default() -> Self {
        Self {
            bits: S::DEFAULT_VALUE,
        }
    }
}

impl<S: RegisterSpec> RegisterWrite<S> {
    pub unsafe fn bits(&mut self, bits: S::Type) {
        self.bits = bits;
    }

    pub unsafe fn bit<const OFFSET: usize>(&mut self, bit: bool) {
        self.field::<OFFSET, 1>(bit.into());
    }

    pub unsafe fn field<const OFFSET: usize, const SIZE: usize>(&mut self, field: S::Type) {
        let mask = S::Type::mask::<SIZE>();

        self.bits = (self.bits & !(mask << OFFSET)) | ((field & mask) << OFFSET);
    }
}

impl RegisterType for u64 {
    fn zero() -> Self {
        0
    }

    fn mask<const WIDTH: usize>() -> Self {
        u64::MAX >> (32 - WIDTH)
    }
}

impl RegisterSpec for u64 {
    type Type = u64;
}

impl RegisterType for u32 {
    fn zero() -> Self {
        0
    }

    fn mask<const WIDTH: usize>() -> Self {
        u32::MAX >> (32 - WIDTH)
    }
}

impl RegisterSpec for u32 {
    type Type = u32;
}
