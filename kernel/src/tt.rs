use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU64, Ordering};

pub struct TranslationTable([AtomicU64; 512]);

impl TranslationTable {
    pub fn new() -> Self {
        Self(unsafe { core::mem::transmute([0u64; 512]) })
    }

    pub fn map_contiguous(&mut self, va_start: usize, va_end: usize, pa_start: usize, flags: &str) {
        // for each page in the range {
        self.map_page(va_start, pa_start, flags);
        // }
    }

    fn map_page(&mut self, va: usize, pa: usize, flags: &str) {
        // 4KiB translation granule
        //   level -1: IA[51:48] (4-bit)
        //   level  0: IA[47:39] (9-bit)
        //   level  1: IA[38:30] (9-bit)
        //   level  2: IA[29:21] (9-bit)
        //   level  3: IA[20:12] (9-bit)
        const MASK: usize = 0b1_1111_1111;
        let level0_index = (va >> 39) & MASK;
        let level1_index = (va >> 30) & MASK;
        let level2_index = (va >> 21) & MASK;
        let level3_index = (va >> 12) & MASK;

        let level0 = self;
        let mut level0_descriptor = level0
            .get_or_set_with(level0_index, || {
                TableDescriptor::new(PageBox::new(TranslationTable::new()))
            })
            .table()
            .expect("table descriptor");

        let mut level1 = level0_descriptor.table_mut();
        let mut level1_descriptor = level1
            .get_or_set_with(level1_index, || {
                TableDescriptor::new(PageBox::new(TranslationTable::new()))
            })
            .table()
            .expect("table descriptor");

        let mut level2 = level1_descriptor.table_mut();
        let mut level2_descriptor = level2
            .get_or_set_with(level2_index, || {
                TableDescriptor::new(PageBox::new(TranslationTable::new()))
            })
            .table()
            .expect("table descriptor");

        let mut level3 = level2_descriptor.table_mut();
        level3.set(0, Some(PageDescriptor::new(0x12345)));
    }

    fn get_or_set_with<F, D>(&mut self, index: usize, f: F) -> Descriptor
    where
        F: FnOnce() -> D,
        D: Into<Descriptor>,
    {
        self.get(index).unwrap_or_else(|| {
            let descriptor = f().into();
            self.set(index, Some(descriptor));
            descriptor
        })
    }

    fn get(&self, index: usize) -> Option<Descriptor> {
        let descriptor = Descriptor::from_bits(self.0[index].load(Ordering::SeqCst));
        if let Some(descriptor) = descriptor {
            Some(descriptor)
        } else {
            None
        }
    }

    fn set<D>(&mut self, index: usize, descriptor: Option<D>)
    where
        D: Into<Descriptor>,
    {
        let bits = match descriptor {
            Some(descriptor) => descriptor.into().bits,
            None => Descriptor::INVALID_BITS,
        };

        self.0[index].store(bits, Ordering::SeqCst);
    }
}

#[derive(Clone, Copy)]
struct Descriptor<Ty = ()> {
    bits: u64,
    phantom: PhantomData<Ty>,
}

impl Descriptor {
    const INVALID_BITS: u64 = 0;

    fn from_bits(bits: u64) -> Option<Self> {
        let valid = (bits & 1) == 1;

        if valid {
            Some(unsafe { Self::from_bits_unchecked(bits) })
        } else {
            None
        }
    }
}

impl<Ty> Descriptor<Ty> {
    unsafe fn from_bits_unchecked(bits: u64) -> Self {
        Self {
            bits,
            phantom: PhantomData,
        }
    }
}

macro_rules! impl_descriptor {
    ($ty:ident, $name_upper:ident) => {
        type $ty = Descriptor<$name_upper>;

        #[derive(Clone, Copy)]
        struct $name_upper;
    };
    ($ty:ident, $name_upper:ident, $name_lower:ident) => {
        impl_descriptor!($ty, $name_upper);

        impl From<$ty> for Descriptor {
            fn from(value: $ty) -> Self {
                unsafe { Descriptor::from_bits_unchecked(value.bits) }
            }
        }

        impl Descriptor {
            fn $name_lower(self) -> Option<$ty> {
                Some(unsafe { $ty::from_bits_unchecked(self.bits) })
            }
        }
    };
}

impl_descriptor!(TableDescriptor, Table, table);
impl_descriptor!(BlockDescriptor, Block, block);
impl_descriptor!(PageDescriptor, Page, page);

impl TableDescriptor {
    fn new(table: PageBox<TranslationTable>) -> Self {
        // TODO: do it properly
        unsafe { Self::from_bits_unchecked(table.leak() as u64 | 0b11) }
    }

    fn table_address(&self) -> usize {
        self.bits as usize & 0x0000fffffffff000
    }

    fn table(&self) -> &TranslationTable {
        let ptr = (PHYS_BASE + self.table_address()) as *const _;

        unsafe { &*ptr }
    }

    fn table_mut(&mut self) -> &mut TranslationTable {
        let ptr = (PHYS_BASE + self.table_address()) as *mut _;

        unsafe { &mut *ptr }
    }
}

impl PageDescriptor {
    fn new(pa: usize) -> Self {
        // TODO: do it properly
        unsafe { Self::from_bits_unchecked(pa as u64 | 0b11) }
    }
}

const PHYS_BASE: usize = 0xffff_0000_0000_0000;

pub struct PageBox<T> {
    pub pa: usize,
    phantom: PhantomData<T>,
}

static mut ALLOC_BASE: usize = 0x4000_0000 + 0x100000;

impl<T> PageBox<T> {
    pub fn new(x: T) -> Self {
        let pa = unsafe { ALLOC_BASE };
        unsafe {
            ALLOC_BASE += 0x1000;
        }
        let ptr_mut = (PHYS_BASE + pa) as *mut T;

        unsafe { ptr_mut.write_volatile(x) };

        unsafe { Self::from_pa(pa) }
    }

    unsafe fn from_pa(pa: usize) -> Self {
        Self {
            pa,
            phantom: PhantomData,
        }
    }

    fn leak(self) -> usize {
        self.pa
    }

    pub fn ptr(&self) -> *const T {
        (PHYS_BASE + self.pa) as *const T
    }

    pub fn ptr_mut(&self) -> *mut T {
        (PHYS_BASE + self.pa) as *mut T
    }
}

impl<T> Drop for PageBox<T> {
    fn drop(&mut self) {
        unsafe { self.ptr_mut().drop_in_place() }
    }
}

impl<T> Deref for PageBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl<T> DerefMut for PageBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr_mut() }
    }
}
