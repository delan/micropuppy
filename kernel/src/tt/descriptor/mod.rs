use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

mod block;
mod page;
mod table;

#[derive(Debug)]
pub struct Descriptor<L, Ty = Unknown> {
    bits: u64,
    phantom: PhantomData<(L, Ty)>,
}

impl<L, Ty> Descriptor<L, Ty> {
    /// Bit representation of an invalid descriptor.
    pub const INVALID_BITS: u64 = 0;

    pub unsafe fn from_bits_unchecked(bits: u64) -> Self {
        Self {
            bits,
            phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> u64 {
        let bits = self.bits;

        core::mem::forget(self);

        bits
    }
}

impl<L> Descriptor<L> {
    pub fn from_bits(bits: u64) -> Option<Self> {
        let valid = (bits & 1) == 1;

        if valid {
            Some(unsafe { Self::from_bits_unchecked(bits) })
        } else {
            None
        }
    }
}

impl<L, Ty> Drop for Descriptor<L, Ty> {
    fn drop(&mut self) {
        todo!("drop for descriptor")
    }
}

pub struct DescriptorRefMut<'tt, L, Ty = Unknown> {
    inner: ManuallyDrop<Descriptor<L, Ty>>,
    phantom: PhantomData<&'tt mut ()>,
}

impl<L> DescriptorRefMut<'_, L> {
    pub fn from_bits(bits: u64) -> Option<Self> {
        Descriptor::from_bits(bits).map(|inner| Self {
            inner: ManuallyDrop::new(inner),
            phantom: PhantomData,
        })
    }
}

impl<L, Ty> DescriptorRefMut<'_, L, Ty> {
    pub unsafe fn from_bits_unchecked(bits: u64) -> Self {
        let inner = Descriptor::from_bits_unchecked(bits);

        Self {
            inner: ManuallyDrop::new(inner),
            phantom: PhantomData,
        }
    }
}

impl<L, Ty> Drop for DescriptorRefMut<'_, L, Ty> {
    fn drop(&mut self) {
        // do nothing
    }
}

impl<L, Ty> Deref for DescriptorRefMut<'_, L, Ty> {
    type Target = Descriptor<L, Ty>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<L, Ty> DerefMut for DescriptorRefMut<'_, L, Ty> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct DescriptorBuilder<L, Ty = Unknown> {
    bits: u64,
    phantom: PhantomData<(L, Ty)>,
}

impl<L> Default for DescriptorBuilder<L> {
    fn default() -> Self {
        Self {
            bits: 0,
            phantom: PhantomData,
        }
    }
}

macro_rules! define_descriptor {
    ($ty:ident) => {
        #[derive(Debug)]
        pub struct $ty;
    };
    ($ty:ident, $descriptor:ident, $builder:ident) => {
        #[derive(Debug)]
        pub struct $ty;

        type $descriptor<L> = Descriptor<L, $ty>;
        type $builder<L> = DescriptorBuilder<L, $ty>;

        impl<L> From<$descriptor<L>> for Descriptor<L> {
            fn from(value: $descriptor<L>) -> Self {
                unsafe { Descriptor::from_bits_unchecked(value.into_inner()) }
            }
        }
    };
}

define_descriptor!(Unknown);
define_descriptor!(Table, TableDescriptor, TableDescriptorBuilder);
define_descriptor!(Block, BlockDescriptor, BlockDescriptorBuilder);
define_descriptor!(Page, PageDescriptor, PageDescriptorBuilder);
