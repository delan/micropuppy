use core::{fmt, slice};

use buddy_alloc::tree::{OutOfMemoryError, Tree};

use crate::PAGE_SIZE;

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
        extern "C" {
            static BUDDY_ALLOC_TREE: u8;
        }

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

        let base = unsafe { &BUDDY_ALLOC_TREE } as *const u8 as *mut u8;

        let storage = unsafe { slice::from_raw_parts_mut(base, tree_len) };

        let tree_end = unsafe { base.add(tree_len) };
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
