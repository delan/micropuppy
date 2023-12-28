use core::{fmt, iter};

use bitvec::prelude::*;
use num::AsUsize;

/// A binary tree tracking the state of arbitrarily-sized memory blocks within a buddy allocation
/// scheme.
#[derive(Debug)]
pub struct Tree<'s> {
    /// Bit-level storage of block states.
    storage: &'s mut BitSlice<u8, Msb0>,
    /// Count of leaf blocks in the tree.
    leaf_blocks: usize,
    /// Total depth of the tree, or equivalently, the number of edges between the root block and a
    /// leaf block.
    depth: usize,
    /// Block index of the first leaf block.
    first_leaf: usize,
}

/// A successful allocation, measured in blocks.
#[derive(PartialEq, Eq, Debug)]
pub struct Allocation {
    /// Start of the allocation.
    pub offset: usize,
    /// Number of blocks spanned by this allocation.
    ///
    /// May be larger than the requested number of blocks.
    pub size: usize,
}

#[derive(PartialEq, Eq, Debug)]
pub struct OutOfMemoryError;

#[derive(PartialEq, Eq, Debug)]
pub struct DoubleFreeError;

impl<'s> Tree<'s> {
    /// Size, in bits, of a non-leaf block.
    const NONLEAF_BITS: usize = 2;
    /// Size, in bits, of a leaf block.
    const LEAF_BITS: usize = 1;

    /// Returns the number of bits required to store a tree with at least the specified number of
    /// leaf blocks.
    pub fn storage_bits_required(leaf_blocks: usize) -> usize {
        assert!(leaf_blocks > 0, "tree must have at least 1 leaf block");

        let leaf_blocks = leaf_blocks.next_power_of_two();
        let nonleaf_blocks = leaf_blocks - 1;

        nonleaf_blocks * Self::NONLEAF_BITS + leaf_blocks * Self::LEAF_BITS
    }

    /// Creates a new tree with all blocks initially marked as free.
    pub fn new(storage: &'s mut [u8], leaf_blocks: usize) -> Self {
        // i have no leaf blocks and i must store state (a tree with no leaf blocks can't manage any
        // allocations)
        assert!(leaf_blocks > 0, "tree must have at least 1 leaf block");

        let depth = leaf_blocks.next_power_of_two().ilog2().as_usize();
        let first_leaf = (1 << depth) - 1;

        // we must be able to store a complete tree's worth of blocks
        let storage = storage.view_bits_mut();
        let bits = Self::storage_bits_required(leaf_blocks);
        assert!(
            storage.len() >= bits,
            "storage must be at least {bits} bits wide to store a tree with {leaf_blocks} leaf blocks"
        );

        // the storage we're provided might be wider than required
        let storage = &mut storage[0..bits];

        // initially, every block is free
        // TODO: can we do this without inlining the encoding of BlockState::Free?
        storage.fill(false);

        Self {
            storage,
            leaf_blocks,
            depth,
            first_leaf,
        }
    }

    /// Attempts to allocate `size` blocks.
    ///
    /// If successful, the returned [`Allocation`] may be larger than the requested size due to
    /// rounding.
    pub fn allocate(&mut self, size: usize) -> Result<Allocation, OutOfMemoryError> {
        // determine block height and depth for requested allocation
        let height = match size {
            0 => return Err(OutOfMemoryError),
            1 => 0,
            _ => (size - 1).ilog2() as usize + 1,
        };
        let depth = self.depth - height;

        // find a free block at the requested depth
        let block = self.preorder(|block| {
            let at_requested_depth = block.depth() == depth;
            match (at_requested_depth, self.state(block)) {
                // if we're at the requested depth and have found a free block, claim it
                (true, BlockState::Free) => Action::Yield(block),
                // ...but, if the block isn't free (because it's either been allocated or
                // subdivided), there's no point descending further since the block's sub-blocks
                // will all have a higher depth (and thus smaller size) than requested.
                (true, _) => Action::Skip,
                // if we're not yet at the requested depth, don't descend into blocks with no
                // reachable, free sub-blocks
                (false, BlockState::Allocated | BlockState::SuperblockFull) => Action::Skip,
                // ...but, descend into blocks that may have reachable, free sub-blocks.
                (false, _) => Action::Descend,
            }
        });

        // if we didn't find a block, we're out of memory (at the requested allocation size)
        let block = block.ok_or(OutOfMemoryError)?;

        // mark the block as allocated
        self.set_state(block, BlockState::Allocated);

        // we know the state of our block has changed from free to allocated.
        //
        // we now need to mark every superblock of our block as either a superblock or a full
        // superblock.
        // - a block where both sub-blocks are either full superblocks or allocated becomes a full
        //   superblock (no new allocations can take place within the block)
        // - otherwise, the block must have at least one superblock as a sub-block, and thus becomes
        //   a superblock (the block cannot be allocated, but it contains sub-blocks available for
        //   allocation)
        //
        // since we just allocated a block, it's not possible for any of the superblocks to become
        // free.
        let mut buddies = self.buddies(block);

        // mark as many blocks as full as possible
        for (buddy, block) in &mut buddies {
            let block_is_full = match self.state(buddy) {
                BlockState::Allocated | BlockState::SuperblockFull => true,
                BlockState::Free | BlockState::Superblock => false,
            };

            if !block_is_full {
                // since the item has been consumed from the iterator, we need to mark the block as
                // a superblock here otherwise it will be missed by the loop below
                self.set_state(block, BlockState::Superblock);
                break;
            }

            self.set_state(block, BlockState::SuperblockFull);
        }

        // mark remaining blocks as superblocks
        for (_, block) in &mut buddies {
            self.set_state(block, BlockState::Superblock);
        }

        Ok(Allocation {
            offset: block.offset() << height,
            size: 1 << height,
        })
    }

    /// Frees a previous [`Allocation`], identified by its offset.
    pub fn free(&mut self, offset: usize) -> Result<(), DoubleFreeError> {
        // find the block corresponding to this allocation - the offset does not uniquely identify a
        // block, but does uniquely identify an allocation
        let block = self.preorder(|block| {
            let height = self.depth - block.depth();
            let at_correct_offset = block.offset() << height == offset;
            match (self.state(block), at_correct_offset) {
                // if we've found an allocated block with the correct offset, it's the block
                // corresponding to the allocation
                (BlockState::Allocated, true) => Action::Yield(block),
                // ...but, if the block is allocated and has the wrong offset, there's no point
                // searching its subblocks as they can't possibly contain our allocation.
                (BlockState::Allocated, false) => Action::Skip,
                // a free block has no allocated sub-blocks, so it can't possibly contain our
                // allocation
                (BlockState::Free, _) => Action::Skip,
                // ...but if the block has allocated sub-blocks, we need to search them for our
                // allocation.
                (BlockState::Superblock | BlockState::SuperblockFull, _) => Action::Descend,
            }
        });

        // if we couldn't find the block, we've either been passed garbage or we're experiencing a
        // double free
        let block = block.ok_or(DoubleFreeError)?;

        // mark the block as free
        self.set_state(block, BlockState::Free);

        // we know the state of our block has changed from allocated to free.
        //
        // we now need to mark every superblock of our block as either free or as a (no longer full)
        // superblock.
        // - a block with two free children becomes free (the block could now be allocated)
        // - otherwise, the block has at least one allocated sub-block, and thus becomes a
        //   superblock
        //
        // since we just freed a block, it's not possible for any of the superblocks to become full.
        let mut buddies = self.buddies(block);

        // mark as many blocks as free as possible
        for (buddy, block) in &mut buddies {
            if self.state(buddy) != BlockState::Free {
                // since the item has been consumed from the iterator, we need to mark the block as
                // a superblock here otherwise it will be missed by the loop below
                self.set_state(block, BlockState::Superblock);
                break;
            }

            self.set_state(block, BlockState::Free);
        }

        // mark remaining blocks as subdivided
        for (_, block) in &mut buddies {
            self.set_state(block, BlockState::Superblock);
        }

        Ok(())
    }

    fn preorder<T>(&self, mut visitor: impl FnMut(BlockIndex) -> Action<T>) -> Option<T> {
        fn preorder<T>(
            tree: &Tree,
            block: BlockIndex,
            visitor: &mut impl FnMut(BlockIndex) -> Action<T>,
        ) -> Option<T> {
            if !tree.has_block(block) {
                return None;
            }

            let action = visitor(block);
            match action {
                Action::Yield(value) => Some(value),
                Action::Skip => None,
                Action::Descend => {
                    let (left, right) = block.subblocks();

                    preorder(tree, left, visitor).or_else(|| preorder(tree, right, visitor))
                }
            }
        }

        preorder(self, BlockIndex::root(), &mut visitor)
    }

    fn state(&self, block: BlockIndex) -> BlockState {
        assert!(self.has_block(block));

        if block.0 < self.first_leaf {
            let index = 2 * block.0;
            let subdivided = self.storage[index];
            let allocated_or_full = self.storage[index + 1];

            match (subdivided, allocated_or_full) {
                (false, false) => BlockState::Free,
                (false, true) => BlockState::Allocated,
                (true, false) => BlockState::Superblock,
                (true, true) => BlockState::SuperblockFull,
            }
        } else {
            let index = 2 * self.first_leaf + (block.0 - self.first_leaf);
            let allocated = self.storage[index];

            match allocated {
                false => BlockState::Free,
                true => BlockState::Allocated,
            }
        }
    }

    fn set_state(&mut self, block: BlockIndex, state: BlockState) {
        assert!(self.has_block(block));

        if block.0 < self.first_leaf {
            let index = 2 * block.0;
            let (subdivided, allocated_or_full) = match state {
                BlockState::Free => (false, false),
                BlockState::Allocated => (false, true),
                BlockState::Superblock => (true, false),
                BlockState::SuperblockFull => (true, true),
            };

            self.storage.set(index, subdivided);
            self.storage.set(index + 1, allocated_or_full);
        } else {
            let index = 2 * self.first_leaf + (block.0 - self.first_leaf);
            let allocated = match state {
                BlockState::Free => false,
                BlockState::Allocated => true,
                BlockState::Superblock | BlockState::SuperblockFull => {
                    panic!("leaf blocks cannot be superblocks")
                }
            };

            self.storage.set(index, allocated);
        }
    }

    fn blocks(&self) -> impl Iterator<Item = BlockIndex> + '_ {
        (0..self.block_count()).map(BlockIndex)
    }

    fn buddies(&self, block: BlockIndex) -> impl Iterator<Item = (BlockIndex, BlockIndex)> {
        let mut block = block;

        iter::from_fn(move || {
            let superblock = block.superblock();
            let buddy = block.buddy();

            if let Some(superblock) = superblock {
                block = superblock;
            }

            buddy.zip(superblock)
        })
    }

    fn has_block(&self, block: BlockIndex) -> bool {
        block.0 < self.block_count()
    }

    fn block_count(&self) -> usize {
        (1 << (self.depth + 1)) - 1
    }

    pub fn dot(&self) -> Dot {
        Dot(self)
    }
}

#[derive(Debug)]
enum Action<T> {
    Yield(T),
    Skip,
    Descend,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BlockState {
    /// Block has not been subdivided nor allocated.
    Free,
    /// Block has not been subdivided but has been allocated.
    Allocated,
    /// Block is a superblock and has one or more allocated sub-blocks.
    Superblock,
    /// Block is a superblock and has no reachable and free sub-blocks.
    SuperblockFull,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct BlockIndex(usize);

impl BlockIndex {
    fn root() -> Self {
        Self(0)
    }

    pub fn is_root(self) -> bool {
        self.0 == 0
    }

    pub fn superblock(self) -> Option<Self> {
        if !self.is_root() {
            Some(Self((self.0 - 1) / 2))
        } else {
            None
        }
    }

    pub fn buddy(self) -> Option<Self> {
        if !self.is_root() {
            Some(Self(((self.0 - 1) ^ 1) + 1))
        } else {
            None
        }
    }

    pub fn subblocks(self) -> (Self, Self) {
        let left = Self(2 * self.0 + 1);
        let right = Self(2 * self.0 + 2);

        (left, right)
    }

    pub fn depth(self) -> usize {
        (self.0 + 1).ilog2() as usize
    }

    pub fn offset(self) -> usize {
        self.0 + 1 - (1 << self.depth())
    }
}

#[derive(Debug)]
pub struct Dot<'t, 's>(&'t Tree<'s>);

impl fmt::Display for Dot<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tree = self.0;

        writeln!(f, "digraph {{")?;
        writeln!(f, "  node [style=filled, fixedsize=true];")?;
        for block in tree.blocks() {
            const GREEN: &str = "#9dd5c0";
            const BLUE: &str = "#27a4dd";
            const RED: &str = "#f1646c";
            let (fillcolor, shape) = match tree.state(block) {
                BlockState::Free => (GREEN, "circle"),
                BlockState::Superblock => (BLUE, "Mcircle"),
                BlockState::Allocated => (RED, "square"),
                BlockState::SuperblockFull => (RED, "Msquare"),
            };

            writeln!(
                f,
                "  n{} [fillcolor=\"{}\", shape=\"{}\"];",
                block.0, fillcolor, shape
            )?;

            let (left, right) = block.subblocks();
            for child in [left, right] {
                if tree.has_block(child) {
                    writeln!(f, "  n{} -> n{};", block.0, child.0)?;
                }
            }
        }
        write!(f, "}}")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_depth_required() {
        macro_rules! assert_storage_bits_required {
            ($leaf_blocks:expr, $storage_bits_required:expr) => {
                let bits = Tree::storage_bits_required($leaf_blocks);
                assert_eq!(
                    bits, $storage_bits_required,
                    "with leaf_blocks = {}",
                    $leaf_blocks
                );
            };
        }

        // 0 leaf blocks:
        // -> should panic, no test

        // 1 leaf block:
        //        1
        for leaf_blocks in [1] {
            assert_storage_bits_required!(leaf_blocks, 1 * Tree::LEAF_BITS);
        }

        // 2 leaf blocks:
        //        2
        //    1       1
        for leaf_blocks in [2] {
            assert_storage_bits_required!(
                leaf_blocks,
                1 * Tree::NONLEAF_BITS + 2 * Tree::LEAF_BITS
            );
        }

        // 3 to 4 leaf blocks:
        //        2
        //    2       2
        //  1   1   1   1
        for leaf_blocks in [3, 4] {
            assert_storage_bits_required!(
                leaf_blocks,
                3 * Tree::NONLEAF_BITS + 4 * Tree::LEAF_BITS
            );
        }

        // 5 to 8 leaf blocks:
        //        2
        //    2       2
        //  2   2   2   2
        // 1 1 1 1 1 1 1 1
        for leaf_blocks in [5, 6, 7, 8] {
            assert_storage_bits_required!(
                leaf_blocks,
                7 * Tree::NONLEAF_BITS + 8 * Tree::LEAF_BITS
            );
        }
    }

    // offsets:
    //        0         depth = 0, height = 3
    //    0       4     depth = 1, height = 2
    //  0   2   4   6   depth = 2, height = 1
    // 0 1 2 3 4 5 6 7  depth = 3, height = 0
    //
    // block indices:
    //        0         depth = 0, height = 3
    //    1       2     depth = 1, height = 2
    //  3   4   5   6   depth = 2, height = 1
    // 7 8 9 a b c d e  depth = 3, height = 0

    #[test]
    fn allocate() {
        let mut storage = [0; 4];
        let mut tree = Tree::new(&mut storage, 8);

        // block index 7
        assert_eq!(tree.allocate(1), Ok(Allocation { offset: 0, size: 1 }));
        eprintln!("{}", tree.dot());

        // block index 8
        assert_eq!(tree.allocate(1), Ok(Allocation { offset: 1, size: 1 }));
        eprintln!("{}", tree.dot());

        // block index 9
        assert_eq!(tree.allocate(1), Ok(Allocation { offset: 2, size: 1 }));
        eprintln!("{}", tree.dot());

        // block index 5
        assert_eq!(tree.allocate(2), Ok(Allocation { offset: 4, size: 2 }));
        eprintln!("{}", tree.dot());

        // block index 10
        assert_eq!(tree.allocate(1), Ok(Allocation { offset: 3, size: 1 }));
        eprintln!("{}", tree.dot());

        // block index 13
        assert_eq!(tree.allocate(1), Ok(Allocation { offset: 6, size: 1 }));
        eprintln!("{}", tree.dot());

        // block index 14
        assert_eq!(tree.allocate(1), Ok(Allocation { offset: 7, size: 1 }));
        eprintln!("{}", tree.dot());

        assert_eq!(tree.allocate(1), Err(OutOfMemoryError));
    }

    #[test]
    fn preorder_descend() {
        let mut storage = [0; 4];
        let tree = Tree::new(&mut storage, 8);

        let mut preorder = Vec::with_capacity(tree.block_count());
        let result = tree.preorder(|block| -> Action<()> {
            preorder.push(block);

            Action::Descend
        });

        assert_eq!(
            preorder,
            [0, 1, 3, 7, 8, 4, 9, 10, 2, 5, 11, 12, 6, 13, 14]
                .into_iter()
                .map(BlockIndex)
                .collect::<Vec<_>>()
        );
        assert_eq!(result, None);
    }

    #[test]
    fn preorder_skip() {
        let mut storage = [0; 4];
        let tree = Tree::new(&mut storage, 8);

        let mut preorder = Vec::with_capacity(tree.block_count());
        let result = tree.preorder(|block| -> Action<()> {
            preorder.push(block);

            if block.0 == 4 || block.0 == 2 {
                Action::Skip
            } else {
                Action::Descend
            }
        });

        assert_eq!(
            preorder,
            [0, 1, 3, 7, 8, 4, 2]
                .into_iter()
                .map(BlockIndex)
                .collect::<Vec<_>>()
        );
        assert_eq!(result, None);
    }

    #[test]
    fn preorder_yield() {
        let mut storage = [0; 4];
        let tree = Tree::new(&mut storage, 8);

        let mut preorder = Vec::with_capacity(tree.block_count());
        let result = tree.preorder(|block| {
            preorder.push(block);

            if block.0 == 5 {
                Action::Yield(block)
            } else {
                Action::Descend
            }
        });

        assert_eq!(
            preorder,
            [0, 1, 3, 7, 8, 4, 9, 10, 2, 5]
                .into_iter()
                .map(BlockIndex)
                .collect::<Vec<_>>()
        );
        assert_eq!(result, Some(BlockIndex(5)));
    }

    #[test]
    fn block_index() {
        // depth 0, height 3
        let block = BlockIndex(0);
        assert_eq!(block.superblock(), None);
        assert_eq!(block.buddy(), None);
        assert_eq!(block.depth(), 0);
        assert_eq!(block.offset(), 0);

        // depth 1, height 2
        let block = BlockIndex(1);
        assert_eq!(block.superblock(), Some(BlockIndex(0)));
        assert_eq!(block.buddy(), Some(BlockIndex(2)));
        assert_eq!(block.depth(), 1);
        assert_eq!(block.offset(), 0);

        let block = BlockIndex(2);
        assert_eq!(block.superblock(), Some(BlockIndex(0)));
        assert_eq!(block.buddy(), Some(BlockIndex(1)));
        assert_eq!(block.depth(), 1);
        assert_eq!(block.offset(), 1);

        // depth 2, height 1
        let block = BlockIndex(3);
        assert_eq!(block.superblock(), Some(BlockIndex(1)));
        assert_eq!(block.buddy(), Some(BlockIndex(4)));
        assert_eq!(block.depth(), 2);
        assert_eq!(block.offset(), 0);

        let block = BlockIndex(4);
        assert_eq!(block.superblock(), Some(BlockIndex(1)));
        assert_eq!(block.buddy(), Some(BlockIndex(3)));
        assert_eq!(block.depth(), 2);
        assert_eq!(block.offset(), 1);

        let block = BlockIndex(5);
        assert_eq!(block.superblock(), Some(BlockIndex(2)));
        assert_eq!(block.buddy(), Some(BlockIndex(6)));
        assert_eq!(block.depth(), 2);
        assert_eq!(block.offset(), 2);

        let block = BlockIndex(6);
        assert_eq!(block.superblock(), Some(BlockIndex(2)));
        assert_eq!(block.buddy(), Some(BlockIndex(5)));
        assert_eq!(block.depth(), 2);
        assert_eq!(block.offset(), 3);

        // depth 3, height 0
        let block = BlockIndex(7);
        assert_eq!(block.superblock(), Some(BlockIndex(3)));
        assert_eq!(block.buddy(), Some(BlockIndex(8)));
        assert_eq!(block.depth(), 3);
        assert_eq!(block.offset(), 0);

        let block = BlockIndex(8);
        assert_eq!(block.superblock(), Some(BlockIndex(3)));
        assert_eq!(block.buddy(), Some(BlockIndex(7)));
        assert_eq!(block.depth(), 3);
        assert_eq!(block.offset(), 1);

        let block = BlockIndex(9);
        assert_eq!(block.superblock(), Some(BlockIndex(4)));
        assert_eq!(block.buddy(), Some(BlockIndex(10)));
        assert_eq!(block.depth(), 3);
        assert_eq!(block.offset(), 2);

        let block = BlockIndex(10);
        assert_eq!(block.superblock(), Some(BlockIndex(4)));
        assert_eq!(block.buddy(), Some(BlockIndex(9)));
        assert_eq!(block.depth(), 3);
        assert_eq!(block.offset(), 3);

        let block = BlockIndex(11);
        assert_eq!(block.superblock(), Some(BlockIndex(5)));
        assert_eq!(block.buddy(), Some(BlockIndex(12)));
        assert_eq!(block.depth(), 3);
        assert_eq!(block.offset(), 4);

        let block = BlockIndex(12);
        assert_eq!(block.superblock(), Some(BlockIndex(5)));
        assert_eq!(block.buddy(), Some(BlockIndex(11)));
        assert_eq!(block.depth(), 3);
        assert_eq!(block.offset(), 5);

        let block = BlockIndex(13);
        assert_eq!(block.superblock(), Some(BlockIndex(6)));
        assert_eq!(block.buddy(), Some(BlockIndex(14)));
        assert_eq!(block.depth(), 3);
        assert_eq!(block.offset(), 6);

        let block = BlockIndex(14);
        assert_eq!(block.superblock(), Some(BlockIndex(6)));
        assert_eq!(block.buddy(), Some(BlockIndex(13)));
        assert_eq!(block.depth(), 3);
        assert_eq!(block.offset(), 7);
    }
}
