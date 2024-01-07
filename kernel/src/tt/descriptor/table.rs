use core::marker::PhantomData;

use crate::tt::page::PageBox;
use crate::tt::table::TranslationTable;
use crate::tt::IntermediateLevel;

use super::*;

impl<L: IntermediateLevel> DescriptorBuilder<L> {
    pub fn table(self, next_table: PageBox<TranslationTable<L>>) -> TableDescriptorBuilder<L> {
        // TODO: verify PA alignment and size, attributes
        let bits = next_table.leak().addr() as u64 | 0b11;

        TableDescriptorBuilder {
            bits,
            phantom: PhantomData,
        }
    }
}

impl<L: IntermediateLevel> TableDescriptorBuilder<L> {
    pub fn build(self) -> TableDescriptor<L> {
        unsafe { TableDescriptor::from_bits_unchecked(self.bits) }
    }
}

impl<L: IntermediateLevel> Descriptor<L> {
    pub fn table(&self) -> Option<&TableDescriptor<L>> {
        // TODO: check if this is actually a table with the low two bits
        unsafe { core::mem::transmute(self) }
    }

    pub fn table_mut(&mut self) -> Option<&mut TableDescriptor<L>> {
        // TODO: check if this is actually a table with the low two bits
        unsafe { core::mem::transmute(self) }
    }
}

impl<L: IntermediateLevel> TableDescriptor<L> {
    pub fn translation_table(&self) -> &TranslationTable<L::Next> {
        let ptr = self.next_level_table_address() as *const _;

        unsafe { &*ptr }
    }

    pub fn translation_table_mut(&mut self) -> &mut TranslationTable<L::Next> {
        let ptr = self.next_level_table_address() as *mut _;

        unsafe { &mut *ptr }
    }

    fn next_level_table_address(&self) -> usize {
        self.bits as usize & 0x0000fffffffff000
    }
}
