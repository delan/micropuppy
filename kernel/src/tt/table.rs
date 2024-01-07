use core::marker::PhantomData;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::tt::page::PageBox;

use super::descriptor::{Descriptor, DescriptorBuilder, DescriptorRefMut};
use super::Level0;

/// A translation table of 512 entries with an in-memory representation equivalent to both `[u64;
/// 512]` and a hardware translation table. Each entry is an 8-byte [`Descriptor`] owned by this
/// translation table.
#[derive(Debug)]
#[repr(C, align(0x1000))]
pub struct TranslationTable<L> {
    descriptors: [AtomicU64; 512],
    phantom: PhantomData<L>,
}

impl<L> TranslationTable<L> {
    /// Creates a new translation table initialised with invalid descriptors.
    pub fn new() -> Self {
        const INVALID: AtomicU64 = AtomicU64::new(Descriptor::<()>::INVALID_BITS);

        Self {
            descriptors: [INVALID; 512],
            phantom: PhantomData,
        }
    }

    /// Returns the descriptor at `index` from the translation table if the descriptor is valid,
    /// otherwise, uses `build` to create a new descriptor which is stored at `index` and returned.
    fn get_mut_or_set<'tt, B, D>(&'tt mut self, index: usize, build: B) -> DescriptorRefMut<'tt, L>
    where
        B: FnOnce(DescriptorBuilder<L>) -> D,
        D: Into<Descriptor<L>>,
    {
        // TODO: ordering
        let bits = self.descriptors[index].load(Ordering::SeqCst);
        let descriptor = DescriptorRefMut::from_bits(bits);

        if let Some(descriptor) = descriptor {
            descriptor
        } else {
            let descriptor = build(DescriptorBuilder::default()).into();
            let bits = descriptor.into_inner();

            // TODO: ordering
            self.descriptors[index].store(bits, Ordering::SeqCst);

            unsafe { DescriptorRefMut::from_bits_unchecked(bits) }
        }
    }

    /// Replaces a potentially valid descriptor with a new descriptor, returning the previous
    /// descriptor if it was valid.
    fn replace<F, D>(&mut self, index: usize, build: F) -> Option<Descriptor<L>>
    where
        F: FnOnce(DescriptorBuilder<L>) -> D,
        D: Into<Descriptor<L>>,
    {
        let descriptor = build(DescriptorBuilder::default()).into();
        let new_bits = descriptor.into_inner();

        let old_bits = self.descriptors[index].swap(new_bits, Ordering::SeqCst);

        Descriptor::from_bits(old_bits)
    }
}

impl TranslationTable<Level0> {
    pub fn map_contiguous(&mut self, va_start: usize, va_end: usize, pa_start: usize, flags: &str) {
        let mut va = va_start;
        let mut pa = pa_start;
        while va < va_end {
            self.map_page(va, pa, flags);
            va += 0x1000;
            pa += 0x1000;
        }
    }

    /// Creates a mapping between `virtual_address` and the `physical_address`.
    fn map_page(&mut self, virtual_address: usize, physical_address: usize, flags: &str) {
        // 4KiB translation granule
        //   level -1: IA[51:48] (4-bit)
        //   level  0: IA[47:39] (9-bit)
        //   level  1: IA[38:30] (9-bit)
        //   level  2: IA[29:21] (9-bit)
        //   level  3: IA[20:12] (9-bit)
        const MASK: usize = 0b1_1111_1111;
        let level0_index = (virtual_address >> 39) & MASK;
        let level1_index = (virtual_address >> 30) & MASK;
        let level2_index = (virtual_address >> 21) & MASK;
        let level3_index = (virtual_address >> 12) & MASK;

        let mut level0_descriptor = self.get_mut_or_set(level0_index, |builder| {
            builder.table(PageBox::new(TranslationTable::new())).build()
        });

        let level1 = level0_descriptor
            .table_mut()
            .expect("level 0 descriptor should be a table descriptor")
            .translation_table_mut();

        let mut level1_descriptor = level1.get_mut_or_set(level1_index, |builder| {
            builder.table(PageBox::new(TranslationTable::new())).build()
        });

        let level2 = level1_descriptor
            .table_mut()
            .expect("level 1 descriptor should be a table descriptor")
            .translation_table_mut();
        let mut level2_descriptor = level2.get_mut_or_set(level2_index, |builder| {
            builder.table(PageBox::new(TranslationTable::new())).build()
        });

        let level3 = level2_descriptor
            .table_mut()
            .expect("level 2 descriptor should be a table descriptor")
            .translation_table_mut();
        let old_level3_descriptor = level3.replace(level3_index, |builder| {
            builder.page(physical_address).access_flag(true).build()
        });

        // TODO: drop old_level3_descriptor correctly
        // log::debug!("old_level3_descriptor = {:?}", old_level3_descriptor);
        core::mem::forget(old_level3_descriptor);
    }
}
