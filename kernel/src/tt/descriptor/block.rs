use crate::tt::IntermediateLevel;

use super::*;

impl<L: IntermediateLevel> DescriptorBuilder<L> {
    pub fn block(&mut self, pa: usize) -> BlockDescriptor<L> {
        // TODO: verify PA alignment and size, attributes
        let bits = pa | 0b01;

        unsafe { BlockDescriptor::from_bits_unchecked(bits as u64) }
    }
}
