#![cfg_attr(not(test), no_std)]

use core::{fmt, slice};

use buddy_alloc::tree::{DoubleFreeError, OutOfMemoryError, Tree};

pub const PAGE_SIZE: usize = 4096;

pub struct Allocator {
    tree: Tree<'static>,
    heap: *const [u8; PAGE_SIZE],
    tree_len: usize,
    heap_len_pages: usize,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Allocation {
    pub ptr: *mut [u8; PAGE_SIZE],
    pub size: usize,
}

impl Allocator {
    pub fn new(start: *const u8, end: *const u8) -> Self {
        // Treat end as a page pointer.
        assert_eq!(end.align_offset(PAGE_SIZE), 0, "end must be page-aligned");
        let end = end as *const [u8; PAGE_SIZE];

        // Round start to a page pointer, for length calculations only.
        let align_offset = start.align_offset(PAGE_SIZE);
        let start_aligned = unsafe { start.add(align_offset) } as *const [u8; PAGE_SIZE];

        // Create a tree for that many pages, even though in reality some of it
        // will be occupied by the tree itself.
        let tree_block_count = unsafe { end.offset_from(start_aligned) } as usize;
        let tree_len = Tree::storage_required(tree_block_count);
        let tree_depth = Tree::depth_required(tree_block_count);

        let storage = unsafe { slice::from_raw_parts_mut(start as *mut _, tree_len) };

        let tree_end = unsafe { start.add(tree_len) };
        let padding = tree_end.align_offset(PAGE_SIZE);
        let heap = unsafe { tree_end.add(padding) } as *const _;
        let heap_len_pages = unsafe { end.offset_from(heap) } as usize;

        Self {
            tree: Tree::new(storage, tree_depth),
            heap,
            tree_len,
            heap_len_pages,
        }
    }

    pub fn allocate(&mut self, block_count: usize) -> Result<Allocation, OutOfMemoryError> {
        let allocation = self.tree.allocate(block_count)?;

        if !self.is_within_heap(&allocation) {
            // Free the allocation, so it can be used by future allocations that are smaller
            // and/or have enough adjacent space.
            self.tree
                .free(allocation.offset)
                .expect("Guaranteed by allocate");

            return Err(OutOfMemoryError);
        }

        Ok(Allocation {
            ptr: unsafe { self.heap.add(allocation.offset) } as *mut _,
            size: block_count * PAGE_SIZE,
        })
    }

    pub fn free(&mut self, allocation: Allocation) -> Result<(), DoubleFreeError> {
        let offset = unsafe { allocation.ptr.offset_from(self.heap) };

        if offset < 0 || offset as usize > self.heap_len_pages {
            return Err(DoubleFreeError);
        }

        self.tree.free(offset as usize)
    }

    /// Return false iff the given allocation overflows the actual end of the heap, which may be
    /// less than the space representable by the tree.
    fn is_within_heap(&self, allocation: &buddy_alloc::tree::Allocation) -> bool {
        allocation.offset + allocation.size <= self.heap_len_pages
    }
}

impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Allocator")
            .field("heap", &self.heap)
            .field("tree_len", &self.tree_len)
            .field("heap_len_pages", &self.heap_len_pages)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use core::alloc::Layout;

    use super::*;

    #[test]
    fn allocator() -> Result<(), Error> {
        let layout = Layout::from_size_align(0x100000, 0x100000)?;
        let base = unsafe { std::alloc::alloc(layout) };
        let start = unsafe { base.add(0x1100) };
        let end = unsafe { base.add(0x100000) };

        let mut allocator = Allocator::new(start as *const _, end as *const _);
        assert_eq!(allocator.tree_len, 128);
        assert_eq!(allocator.heap_len_pages, 254);

        let a1 = allocator.allocate(13)?;
        let a2 = allocator.allocate(13)?;
        let a3 = allocator.allocate(13)?;
        assert_eq!(unsafe { (a1.ptr as *const u8).offset_from(base) }, 0x2000);
        assert_eq!(unsafe { (a2.ptr as *const u8).offset_from(base) }, 0x12000);
        assert_eq!(unsafe { (a3.ptr as *const u8).offset_from(base) }, 0x22000);
        assert_eq!(a1.size, 0xD000);
        assert_eq!(a2.size, 0xD000);
        assert_eq!(a3.size, 0xD000);

        allocator.free(a2)?;
        let a4 = allocator.allocate(17)?;
        let a5 = allocator.allocate(4)?;
        assert_eq!(unsafe { (a4.ptr as *const u8).offset_from(base) }, 0x42000);
        assert_eq!(unsafe { (a5.ptr as *const u8).offset_from(base) }, 0x12000);
        assert_eq!(a4.size, 0x11000);
        assert_eq!(a5.size, 0x4000);

        Ok(())
    }

    #[test]
    fn heap_overflow() -> Result<(), Error> {
        let layout = Layout::from_size_align(0x100000, 0x100000)?;
        let base = unsafe { std::alloc::alloc(layout) };
        let start = unsafe { base.add(0x1100) };
        let end = unsafe { base.add(0x5000) };

        // The tree has depth 2, so it can manage the allocation of up to 4 blocks,
        // but there are only 3 pages of usable heap space (0x2000..0x5000).
        let mut allocator = Allocator::new(start as *const _, end as *const _);
        eprintln!("{}", allocator.tree.dot());
        assert_eq!(allocator.tree_len, 2);
        assert_eq!(allocator.heap_len_pages, 3);

        // Allocate 2 blocks (offset 0, size 2).
        let a1 = allocator.allocate(2)?;
        assert_eq!(unsafe { (a1.ptr as *const u8).offset_from(base) }, 0x2000);
        assert_eq!(a1.size, 0x2000);

        // The tree thinks it can allocate another 2 blocks (offset 2, size 2),
        // but it would overflow our heap, so we still return OutOfMemoryError.
        assert_eq!(allocator.allocate(2), Err(OutOfMemoryError));

        // Allocate 1 block (offset 2, size 1).
        let a2 = allocator.allocate(1)?;
        assert_eq!(unsafe { (a2.ptr as *const u8).offset_from(base) }, 0x4000);
        assert_eq!(a2.size, 0x1000);

        Ok(())
    }

    #[derive(Debug)]
    enum Error {
        LayoutError,
        OutOfMemoryError,
        DoubleFreeError,
    }

    impl From<core::alloc::LayoutError> for Error {
        fn from(_: core::alloc::LayoutError) -> Self {
            Self::LayoutError
        }
    }

    impl From<OutOfMemoryError> for Error {
        fn from(_: OutOfMemoryError) -> Self {
            Self::OutOfMemoryError
        }
    }

    impl From<DoubleFreeError> for Error {
        fn from(_: DoubleFreeError) -> Self {
            Self::DoubleFreeError
        }
    }
}
