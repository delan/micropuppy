use core::mem::MaybeUninit;
use core::{fmt, slice};

use buddy_alloc::tree::{OutOfMemoryError, Tree};

use crate::PAGE_SIZE;

pub struct Allocator {
    tree: Tree<'static>,
    heap: *const (),
    block_count: usize,
    tree_len: usize,
}

impl Allocator {
    pub fn new(block_count: usize) -> Self {
        extern "C" {
            static BUDDY_ALLOC_TREE: MaybeUninit<u8>;
        }
        let storage_required = Tree::storage_required(block_count);
        let base = unsafe { &BUDDY_ALLOC_TREE } as *const _ as *mut _;

        let storage =
            unsafe { slice::from_raw_parts_mut::<MaybeUninit<u8>>(base, storage_required) };
        for byte in storage.iter_mut() {
            byte.write(0);
        }
        let storage = unsafe { MaybeUninit::slice_assume_init_mut(storage) };

        let base = unsafe { base.add(storage_required) };
        let padding = base.align_offset(PAGE_SIZE);
        let heap = unsafe { base.add(padding) } as *const _;

        Self {
            // TODO not all of the space this represents will be usable, because even if block_count
            // was a power of two, the tree storage eats into the start of this space
            tree: Tree::new(storage, Tree::depth_required(block_count)),
            heap,
            block_count,
            tree_len: storage_required,
        }
    }

    pub fn allocate(&mut self, block_count: usize) -> Result<(), OutOfMemoryError> {
        let allocation = self.tree.allocate(block_count)?;
        dbg!(allocation);

        Ok(())
    }
}

impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Allocator")
            .field("heap", &self.heap)
            .field("block_count", &self.block_count)
            .field("tree_len", &self.tree_len)
            .finish()
    }
}
