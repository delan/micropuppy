use core::cmp;
use core::ops;

pub mod memory_mapped;
pub mod system;

pub trait RegisterBits:
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
    type Bits: RegisterBits;
}

pub trait RegisterReadable: RegisterSpec {}

pub trait RegisterWritable: RegisterSpec {}

pub trait RegisterDefault: RegisterSpec {
    const DEFAULT_VALUE: Self::Bits;
}

pub struct RegisterReader<S: RegisterSpec> {
    pub bits: S::Bits,
}

impl<S: RegisterSpec> RegisterReader<S> {
    fn new(bits: S::Bits) -> Self {
        Self { bits }
    }

    pub fn bits(&self) -> S::Bits {
        self.bits
    }

    pub fn bit<const OFFSET: usize>(&self) -> bool {
        self.field::<OFFSET, 1>() != S::Bits::zero()
    }

    pub fn field<const OFFSET: usize, const SIZE: usize>(&self) -> S::Bits {
        (self.bits >> OFFSET) & S::Bits::mask::<SIZE>()
    }
}

pub struct RegisterWriter<S: RegisterSpec> {
    pub bits: S::Bits,
}

impl<S: RegisterSpec> RegisterWriter<S> {
    fn zero() -> Self {
        Self {
            bits: Default::default(),
        }
    }
}

impl<S: RegisterSpec + RegisterDefault> RegisterWriter<S> {
    fn default() -> Self {
        Self {
            bits: S::DEFAULT_VALUE,
        }
    }
}

impl<S: RegisterSpec> RegisterWriter<S> {
    pub unsafe fn bits(&mut self, bits: S::Bits) {
        self.bits = bits;
    }

    pub unsafe fn bit<const OFFSET: usize>(&mut self, bit: bool) {
        self.field::<OFFSET, 1>(bit.into());
    }

    pub unsafe fn field<const OFFSET: usize, const SIZE: usize>(&mut self, field: S::Bits) {
        let mask = S::Bits::mask::<SIZE>();

        self.bits = (self.bits & !(mask << OFFSET)) | ((field & mask) << OFFSET);
    }
}

impl RegisterBits for u64 {
    fn zero() -> Self {
        0
    }

    fn mask<const WIDTH: usize>() -> Self {
        u64::MAX >> (32 - WIDTH)
    }
}

impl RegisterSpec for u64 {
    type Bits = u64;
}

impl RegisterBits for u32 {
    fn zero() -> Self {
        0
    }

    fn mask<const WIDTH: usize>() -> Self {
        u32::MAX >> (32 - WIDTH)
    }
}

impl RegisterSpec for u32 {
    type Bits = u32;
}
