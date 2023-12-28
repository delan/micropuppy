use core::{fmt, iter};

use bitvec::prelude::*;
use num::AsUsize;

#[derive(Debug)]
pub struct Tree<'s> {
    storage: &'s mut BitSlice<u8, Msb0>,
    leaf_blocks: usize,
    depth: usize,
    first_leaf: usize,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Allocation {
    pub offset: usize,
    pub size: usize,
}

#[derive(PartialEq, Eq, Debug)]
pub struct OutOfMemoryError;

#[derive(PartialEq, Eq, Debug)]
pub struct DoubleFreeError;

#[derive(Debug)]
pub enum Action<T> {
    Yield(T),
    Skip,
    Descend,
}

impl<'s> Tree<'s> {
    /// Size, in bits, of a non-leaf block
    const NONLEAF_BITS: usize = 2;
    /// Size, in bits, of a leaf block
    const LEAF_BITS: usize = 1;

    /// Returns the number of bits required to store a tree with at least the specified number of
    /// leaf blocks.
    pub fn storage_bits_required(leaf_blocks: usize) -> usize {
        assert!(leaf_blocks != 0, "tree must have at least 1 leaf block");

        let leaf_blocks = leaf_blocks.next_power_of_two();
        let nonleaf_blocks = leaf_blocks - 1;

        nonleaf_blocks * Self::NONLEAF_BITS + leaf_blocks * Self::LEAF_BITS
    }

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
                // ...but, if the block isn't free, there's no point descending further since the
                // block's sub-blocks will all have a higher depth (and thus smaller size) than
                // requested.
                (true, _) => Action::Skip,
                // if we're not yet at the requested depth, don't descend into blocks with no
                // reachable, free sub-blocks
                (false, BlockState::Full | BlockState::Allocated) => Action::Skip,
                // ...but, descend into blocks that may have reachable, free sub-blocks.
                (false, _) => Action::Descend,
            }
        });

        // if we didn't find a block, we're out of memory (at the requested allocation size)
        let block = block.ok_or(OutOfMemoryError)?;

        // mark the block as allocated
        self.set_state(block, BlockState::Allocated);

        // we know the state of our allocated block has changed from free to allocated.
        //
        // we now need to mark every superblock of our block as either subdivided or full.
        // - a block with two full or allocated sub-blocks becomes full (no new allocations can
        //   take place within the block)
        // - otherwise, the block must have at least one subdivided sub-block, and thus becomes
        //   subdivided (the block cannot be allocated, but it contains sub-blocks available for
        //   allocation)
        //
        // since we just allocated a sub-block, it's not possible for any of the superblocks to
        // become free.
        let mut buddies = self.buddies(block);

        // mark as many superblocks as full as possible
        for (buddy, superblock) in &mut buddies {
            let superblock_is_full = match self.state(buddy) {
                BlockState::Full | BlockState::Allocated => true,
                BlockState::Free | BlockState::Subdivided => false,
            };

            if !superblock_is_full {
                // since the item has been consumed from the iterator, we need to mark the
                // superblock as subdivided here otherwise it will be missed by the loop below
                self.set_state(superblock, BlockState::Subdivided);
                break;
            }

            self.set_state(superblock, BlockState::Full);
        }

        // mark remaining superblocks as subdivided
        for (_, superblock) in &mut buddies {
            self.set_state(superblock, BlockState::Subdivided);
        }

        Ok(Allocation {
            offset: block.offset() << height,
            size: 1 << height,
        })
    }

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
                (BlockState::Subdivided | BlockState::Full, _) => Action::Descend,
            }
        });

        // if we couldn't find the block, we've either been passed garbage or we're experiencing a
        // double free
        let block = block.ok_or(DoubleFreeError)?;

        // mark the block as free
        self.set_state(block, BlockState::Free);

        // we know the state of our allocated block has changed from allocated to free.
        //
        // we now need to mark every superblock of our block as either free or subdivided.
        // - a block with two free children becomes free (the block could now be allocated)
        // - otherwise, the block has at least one allocated sub-block, and thus becomes subdivided
        //
        // since we just freed a sub-block, it's not possible for any of the superblocks to become
        // full.
        let mut buddies = self.buddies(block);

        // mark as many superblocks as free as possible
        for (buddy, superblock) in &mut buddies {
            let superblock_is_free = self.state(buddy) == BlockState::Free;

            if !superblock_is_free {
                // since the item has been consumed from the iterator, we need to mark the
                // superblock as subdivided here otherwise it will be missed by the loop below
                self.set_state(superblock, BlockState::Subdivided);
                break;
            }

            self.set_state(superblock, BlockState::Free);
        }

        // mark remaining superblocks as subdivided
        for (_, superblock) in &mut buddies {
            self.set_state(superblock, BlockState::Subdivided);
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

    fn state(&self, node: BlockIndex) -> BlockState {
        assert!(self.has_block(node));

        if node.0 < self.first_leaf {
            let index = 2 * node.0;
            let subdivided = self.storage[index];
            let allocated_or_full = self.storage[index + 1];

            match (subdivided, allocated_or_full) {
                (false, false) => BlockState::Free,
                (false, true) => BlockState::Allocated,
                (true, false) => BlockState::Subdivided,
                (true, true) => BlockState::Full,
            }
        } else {
            let index = 2 * self.first_leaf + (node.0 - self.first_leaf);
            let allocated = self.storage[index];

            if !allocated {
                BlockState::Free
            } else {
                BlockState::Allocated
            }
        }
    }

    fn set_state(&mut self, node: BlockIndex, state: BlockState) {
        assert!(self.has_block(node));

        if node.0 < self.first_leaf {
            let index = 2 * node.0;
            let (subdivided, allocated_or_full) = match state {
                BlockState::Free => (false, false),
                BlockState::Allocated => (false, true),
                BlockState::Subdivided => (true, false),
                BlockState::Full => (true, true),
            };

            self.storage.set(index, subdivided);
            self.storage.set(index + 1, allocated_or_full);
        } else {
            let index = 2 * self.first_leaf + (node.0 - self.first_leaf);
            let allocated = match state {
                BlockState::Free => false,
                BlockState::Allocated => true,
                _ => panic!(),
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockState {
    /// Block has no allocated sub-blocks.
    Free,
    /// Block has been subdivided and has one or more allocated sub-blocks and one or more
    /// reachable and free sub-blocks.
    Subdivided,
    /// Block has been subdivided and has no reachable and free sub-blocks.
    Full,
    /// Block has been allocated.
    Allocated,
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
        for node_index in tree.blocks() {
            const GREEN: &str = "#9dd5c0";
            const BLUE: &str = "#27a4dd";
            const RED: &str = "#f1646c";
            let (fillcolor, shape) = match tree.state(node_index) {
                BlockState::Free => (GREEN, "circle"),
                BlockState::Subdivided => (BLUE, "Mcircle"),
                BlockState::Full => (RED, "Msquare"),
                BlockState::Allocated => (RED, "square"),
            };

            writeln!(
                f,
                "  n{} [fillcolor=\"{}\", shape=\"{}\"];",
                node_index.0, fillcolor, shape
            )?;

            let (left, right) = node_index.subblocks();
            for child in [left, right] {
                if tree.has_block(child) {
                    writeln!(f, "  n{} -> n{};", node_index.0, child.0)?;
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
    // node indices:
    //        0         depth = 0, height = 3
    //    1       2     depth = 1, height = 2
    //  3   4   5   6   depth = 2, height = 1
    // 7 8 9 a b c d e  depth = 3, height = 0

    #[test]
    fn allocate() {
        let mut storage = [0; 4];
        let mut tree = Tree::new(&mut storage, 8);

        // node index 7
        assert_eq!(tree.allocate(1), Ok(Allocation { offset: 0, size: 1 }));
        eprintln!("{}", tree.dot());

        // node index 8
        assert_eq!(tree.allocate(1), Ok(Allocation { offset: 1, size: 1 }));
        eprintln!("{}", tree.dot());

        // node index 9
        assert_eq!(tree.allocate(1), Ok(Allocation { offset: 2, size: 1 }));
        eprintln!("{}", tree.dot());

        // node index 5
        assert_eq!(tree.allocate(2), Ok(Allocation { offset: 4, size: 2 }));
        eprintln!("{}", tree.dot());

        // node index 10
        assert_eq!(tree.allocate(1), Ok(Allocation { offset: 3, size: 1 }));
        eprintln!("{}", tree.dot());

        // node index 13
        assert_eq!(tree.allocate(1), Ok(Allocation { offset: 6, size: 1 }));
        eprintln!("{}", tree.dot());

        // node index 14
        assert_eq!(tree.allocate(1), Ok(Allocation { offset: 7, size: 1 }));
        eprintln!("{}", tree.dot());

        assert_eq!(tree.allocate(1), Err(OutOfMemoryError));
    }

    #[test]
    fn preorder_descend() {
        let mut storage = [0; 4];
        let tree = Tree::new(&mut storage, 8);

        let mut preorder = Vec::with_capacity(tree.block_count());
        let result = tree.preorder(|node| -> Action<()> {
            preorder.push(node);

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
        let result = tree.preorder(|node| -> Action<()> {
            preorder.push(node);

            if node.0 == 4 || node.0 == 2 {
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
        let result = tree.preorder(|node| {
            preorder.push(node);

            if node.0 == 5 {
                Action::Yield(node)
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
    fn node_index() {
        // depth 0, height 3
        let node = BlockIndex(0);
        assert_eq!(node.superblock(), None);
        assert_eq!(node.buddy(), None);
        assert_eq!(node.depth(), 0);
        assert_eq!(node.offset(), 0);

        // depth 1, height 2
        let node = BlockIndex(1);
        assert_eq!(node.superblock(), Some(BlockIndex(0)));
        assert_eq!(node.buddy(), Some(BlockIndex(2)));
        assert_eq!(node.depth(), 1);
        assert_eq!(node.offset(), 0);

        let node = BlockIndex(2);
        assert_eq!(node.superblock(), Some(BlockIndex(0)));
        assert_eq!(node.buddy(), Some(BlockIndex(1)));
        assert_eq!(node.depth(), 1);
        assert_eq!(node.offset(), 1);

        // depth 2, height 1
        let node = BlockIndex(3);
        assert_eq!(node.superblock(), Some(BlockIndex(1)));
        assert_eq!(node.buddy(), Some(BlockIndex(4)));
        assert_eq!(node.depth(), 2);
        assert_eq!(node.offset(), 0);

        let node = BlockIndex(4);
        assert_eq!(node.superblock(), Some(BlockIndex(1)));
        assert_eq!(node.buddy(), Some(BlockIndex(3)));
        assert_eq!(node.depth(), 2);
        assert_eq!(node.offset(), 1);

        let node = BlockIndex(5);
        assert_eq!(node.superblock(), Some(BlockIndex(2)));
        assert_eq!(node.buddy(), Some(BlockIndex(6)));
        assert_eq!(node.depth(), 2);
        assert_eq!(node.offset(), 2);

        let node = BlockIndex(6);
        assert_eq!(node.superblock(), Some(BlockIndex(2)));
        assert_eq!(node.buddy(), Some(BlockIndex(5)));
        assert_eq!(node.depth(), 2);
        assert_eq!(node.offset(), 3);

        // depth 3, height 0
        let node = BlockIndex(7);
        assert_eq!(node.superblock(), Some(BlockIndex(3)));
        assert_eq!(node.buddy(), Some(BlockIndex(8)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 0);

        let node = BlockIndex(8);
        assert_eq!(node.superblock(), Some(BlockIndex(3)));
        assert_eq!(node.buddy(), Some(BlockIndex(7)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 1);

        let node = BlockIndex(9);
        assert_eq!(node.superblock(), Some(BlockIndex(4)));
        assert_eq!(node.buddy(), Some(BlockIndex(10)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 2);

        let node = BlockIndex(10);
        assert_eq!(node.superblock(), Some(BlockIndex(4)));
        assert_eq!(node.buddy(), Some(BlockIndex(9)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 3);

        let node = BlockIndex(11);
        assert_eq!(node.superblock(), Some(BlockIndex(5)));
        assert_eq!(node.buddy(), Some(BlockIndex(12)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 4);

        let node = BlockIndex(12);
        assert_eq!(node.superblock(), Some(BlockIndex(5)));
        assert_eq!(node.buddy(), Some(BlockIndex(11)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 5);

        let node = BlockIndex(13);
        assert_eq!(node.superblock(), Some(BlockIndex(6)));
        assert_eq!(node.buddy(), Some(BlockIndex(14)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 6);

        let node = BlockIndex(14);
        assert_eq!(node.superblock(), Some(BlockIndex(6)));
        assert_eq!(node.buddy(), Some(BlockIndex(13)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 7);
    }
}
