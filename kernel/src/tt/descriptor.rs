use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

use super::page::PageBox;
use super::table::TranslationTable;
use super::{FinalLevel, IntermediateLevel};

#[derive(Debug)]
pub struct Descriptor<L, Ty = Unknown> {
    bits: u64,
    phantom: PhantomData<(L, Ty)>,
}

impl<L> Descriptor<L> {
    /// Bit representation of an invalid descriptor.
    pub const INVALID_BITS: u64 = 0;

    pub fn from_bits(bits: u64) -> Option<Self> {
        let valid = (bits & 1) == 1;

        if valid {
            Some(unsafe { Self::from_bits_unchecked(bits) })
        } else {
            None
        }
    }
}

impl<L, Ty> Descriptor<L, Ty> {
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

impl<L, Ty> Drop for Descriptor<L, Ty> {
    fn drop(&mut self) {
        todo!("drop for descriptor")
    }
}

pub struct DescriptorBuilder<L, Ty = Unknown>(PhantomData<(L, Ty)>);

impl<L> Default for DescriptorBuilder<L> {
    fn default() -> Self {
        Self(PhantomData)
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

#[derive(Debug)]
pub struct Unknown;

pub struct Table;
type TableDescriptor<L> = Descriptor<L, Table>;

impl<L: IntermediateLevel> From<TableDescriptor<L>> for Descriptor<L> {
    fn from(value: TableDescriptor<L>) -> Self {
        unsafe { Descriptor::from_bits_unchecked(value.into_inner()) }
    }
}

impl<L: IntermediateLevel> DescriptorBuilder<L> {
    pub fn table(&mut self, next_table: PageBox<TranslationTable<L>>) -> TableDescriptor<L> {
        // TODO: verify PA alignment and size, attributes
        let bits = next_table.leak().addr() | 0b11;

        unsafe { TableDescriptor::from_bits_unchecked(bits as u64) }
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

struct Block;
type BlockDescriptor<L> = Descriptor<L, Block>;

impl<L: IntermediateLevel> DescriptorBuilder<L> {
    pub fn block(&mut self, pa: usize) -> BlockDescriptor<L> {
        // TODO: verify PA alignment and size, attributes
        let bits = pa | 0b01;

        unsafe { BlockDescriptor::from_bits_unchecked(bits as u64) }
    }
}

pub struct Page;
type PageDescriptor<L> = Descriptor<L, Page>;

impl<L: FinalLevel> DescriptorBuilder<L> {
    pub fn page(&mut self, pa: usize) -> PageDescriptor<L> {
        // TODO: verify PA alignment and size, attributes
        let bits = pa | (1 << 10) | 0b11;

        unsafe { PageDescriptor::from_bits_unchecked(bits as u64) }
    }
}

impl<L: FinalLevel> From<PageDescriptor<L>> for Descriptor<L> {
    fn from(value: PageDescriptor<L>) -> Self {
        unsafe { Descriptor::from_bits_unchecked(value.into_inner()) }
    }
}

// macro_rules! impl_descriptor {
//     ($ty:ident, $name_upper:ident) => {
//         type $ty = Descriptor<$name_upper>;

//         #[derive(Clone, Copy)]
//         struct $name_upper;
//     };
//     ($ty:ident, $name_upper:ident, $name_lower:ident) => {
//         impl_descriptor!($ty, $name_upper);

//         impl<L> From<$ty> for Descriptor<L> {
//             fn from(value: $ty) -> Self {
//                 unsafe { Descriptor::from_bits_unchecked(value.bits) }
//             }
//         }

//         impl<L> Descriptor<L> {
//             fn $name_lower(self) -> Option<$ty> {
//                 Some(unsafe { $ty::from_bits_unchecked(self.bits) })
//             }
//         }
//     };
// }

// impl_descriptor!(TableDescriptor, Table, table);
// impl_descriptor!(BlockDescriptor, Block, block);
// impl_descriptor!(PageDescriptor, Page, page);

// impl TableDescriptor {
//     fn new(table: PageBox<TranslationTable>) -> Self {
//         // TODO: do it properly
//         unsafe { Self::from_bits_unchecked(table.leak() as u64 | 0b11) }
//     }

//     fn table_address(&self) -> usize {
//         self.bits as usize & 0x0000fffffffff000
//     }

//     fn table(&self) -> &TranslationTable {
//         let ptr = (PHYS_BASE + self.table_address()) as *const _;

//         unsafe { &*ptr }
//     }

//     fn table_mut(&mut self) -> &mut TranslationTable {
//         let ptr = (PHYS_BASE + self.table_address()) as *mut _;

//         unsafe { &mut *ptr }
//     }
// }

// impl PageDescriptor {
//     fn new(pa: usize) -> Self {
//         // TODO: do it properly
//         unsafe { Self::from_bits_unchecked(pa as u64 | 0b11) }
//     }
// }
