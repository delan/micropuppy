use core::alloc::Layout;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

/// A physical address, with an in-memory representation equivalent to a regular pointer to a value
/// of type `T`. Uses the kernel's 1:1 physical memory mapping for accesses via [`Self::ptr()`] and
/// [`Self::ptr_mut()`].
///
/// As with a regular pointer, this type confers no guarantees of validity or alignment.
///
/// The physical address may:
/// - be null
/// - **be unmapped in the kernel's 1:1 physical memory mapping** (an issue unique to physical
///   addresses)
/// - point to uninitialised memory
/// - alias an existing immutable or mutable reference
/// - be misaligned for a value of type `T`
/// - lack any other guarantee as described in [`core::ptr`] or [the nomicon]
///
/// [the nomicon]: https://doc.rust-lang.org/nightly/nomicon/
#[derive(Debug)]
#[repr(transparent)]
pub struct PhysicalAddress<T> {
    addr: usize,
    phantom: PhantomData<T>,
}

impl<T> PhysicalAddress<T> {
    /// The base of the kernel's 1:1 physical memory mapping.
    const PHYS_BASE: usize = 0xffff_0000_0000_0000;

    pub fn from_addr(addr: usize) -> Self {
        Self {
            addr,
            phantom: PhantomData,
        }
    }

    /// Returns the physical address.
    pub fn addr(self) -> usize {
        self.addr
    }

    /// Returns a regular pointer using the kernel's 1:1 physical memory mapping.
    pub fn ptr(self) -> *const T {
        (Self::PHYS_BASE + self.addr) as *const _
    }

    /// Returns a regular, mutable pointer using the kernel's 1:1 physical memory mapping.
    pub fn ptr_mut(self) -> *mut T {
        (Self::PHYS_BASE + self.addr) as *mut _
    }

    /// Casts to a physical address of another type.
    pub fn cast<U>(self) -> PhysicalAddress<U> {
        PhysicalAddress {
            addr: self.addr,
            phantom: PhantomData,
        }
    }
}

// a pointer is Clone even if T isn't Clone (so we can't just `#[derive(Clone)]`)
impl<T> Clone for PhysicalAddress<T> {
    fn clone(&self) -> Self {
        self.cast()
    }
}

// a pointer is Copy even if T isn't Copy (so we can't just `#[derive(Copy)]`)
impl<T> Copy for PhysicalAddress<T> {}

/// An owned page.
#[derive(Debug)]
#[repr(transparent)]
pub struct PageBox<T>(PhysicalAddress<T>);

impl<T> PageBox<T> {
    /// Allocates a new page and places `val` into it.
    pub fn new(val: T) -> Self {
        let layout = Layout::for_value(&val);
        let pa = PageAllocator.alloc(layout).cast::<T>();

        unsafe { pa.ptr_mut().write_volatile(val) };

        Self(pa)
    }

    pub fn leak(self) -> PhysicalAddress<T> {
        self.0
    }
}

impl<T> Drop for PageBox<T> {
    fn drop(&mut self) {
        unsafe { self.0.ptr_mut().drop_in_place() }
    }
}

impl<T> Deref for PageBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.ptr() }
    }
}

impl<T> DerefMut for PageBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.ptr_mut() }
    }
}

// TODO: move this somewhere better, and implement a better allocator that actually tracks
// allocations
static mut ALLOC_BASE: usize = 0x4000_0000 + 0x10_0000;

struct PageAllocator;

impl PageAllocator {
    const PAGE_SIZE: usize = 0x1000;

    /// Allocates a page in physical memory and returns the physical address of the page.
    fn alloc(&self, layout: Layout) -> PhysicalAddress<[u8; Self::PAGE_SIZE]> {
        // we don't support zero-sized allocations
        // TODO: should we support zero-sized allocations?
        assert!(layout.size() > 0);
        // this is a single page, so we can't support an allocation larger than a page
        assert!(layout.size() <= Self::PAGE_SIZE);
        // Layout::align() is guaranteed to be a power of two, so this ensures that the layout's
        // alignment is compatible with page alignment
        assert!(layout.align() <= Self::PAGE_SIZE);

        unsafe {
            let pa = PhysicalAddress::from_addr(ALLOC_BASE);
            ALLOC_BASE += Self::PAGE_SIZE;
            pa
        }
    }
}
