use core::marker::PhantomData;

use crate::tt::FinalLevel;

use super::*;

impl<L: FinalLevel> DescriptorBuilder<L> {
    pub fn page(self, pa: usize) -> PageDescriptorBuilder<L> {
        // TODO: verify PA alignment and size, attributes
        let bits = pa as u64 | 0b11;

        PageDescriptorBuilder {
            bits,
            phantom: PhantomData,
        }
    }
}

impl<L: FinalLevel> PageDescriptorBuilder<L> {
    pub fn access_flag(mut self, access_flag: bool) -> PageDescriptorBuilder<L> {
        if access_flag {
            self.bits |= 1 << 10;
        } else {
            self.bits &= !(1 << 10);
        }

        self
    }

    pub fn build(self) -> PageDescriptor<L> {
        unsafe { PageDescriptor::from_bits_unchecked(self.bits) }
    }
}
