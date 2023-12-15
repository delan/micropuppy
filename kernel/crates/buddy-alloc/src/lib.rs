// #![cfg_attr(not(test), no_std)]
mod tree;

use tree::{Action, Tree};

#[derive(Debug)]
struct BuddyAllocator<'s> {
    tree: Tree<'s>,
}

#[derive(PartialEq, Eq, Debug)]
struct Allocation {
    offset: usize,
    size: usize,
}

impl<'s> BuddyAllocator<'s> {
    fn new(storage: &'s mut [u8]) -> Self {
        Self {
            // TODO: depth from pool size
            tree: Tree::new(storage, 3),
        }
    }

    fn allocate(&mut self, size: usize) -> Option<Allocation> {
        let height = match size {
            0 => return None,
            1 => 0,
            _ => (size - 1).ilog2() as usize + 1,
        };
        let depth = 3 - height; // TODO: use tree depth

        let node = self.tree.preorder(|node_index| {
            let node = self.tree.node(node_index);

            if node.allocated {
                Action::Skip
            } else if node.available && node_index.depth() == depth {
                Action::Yield(node_index)
            } else {
                Action::Descend
            }
        });

        dbg!(node);

        node.map(|node| {
            self.tree.allocate(node);
            self.tree.mark_unavailable(node);

            let mut parent_index = node.parent();
            while let Some(node_index) = parent_index {
                self.tree.mark_unavailable(node_index);

                let (left_index, right_index) = node_index.children();
                if self.tree.node(left_index).allocated && self.tree.node(right_index).allocated {
                    self.tree.allocate(node_index);
                }

                parent_index = node_index.parent();
            }

            Allocation {
                offset: node.offset() << height,
                size: 1 << height,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut storage = [0; 4];
        let mut allocator = BuddyAllocator::new(&mut storage);
        //        0         depth = 0, order = 3
        //    0       4     depth = 1, order = 2
        //  0   2   4   6   depth = 2, order = 1
        // 0 1 2 3 4 5 6 7  depth = 3, order = 0

        assert_eq!(
            allocator.allocate(1),
            Some(Allocation { offset: 0, size: 1 })
        );
        eprintln!("{}", allocator.tree.dot());
        assert_eq!(
            allocator.allocate(1),
            Some(Allocation { offset: 1, size: 1 })
        );
        eprintln!("{}", allocator.tree.dot());
        assert_eq!(
            allocator.allocate(1),
            Some(Allocation { offset: 2, size: 1 })
        );
        eprintln!("{}", allocator.tree.dot());
        assert_eq!(
            allocator.allocate(2),
            Some(Allocation { offset: 4, size: 2 })
        );
        eprintln!("{}", allocator.tree.dot());
        assert_eq!(
            allocator.allocate(1),
            Some(Allocation { offset: 3, size: 1 })
        );
        eprintln!("{}", allocator.tree.dot());
        assert_eq!(
            allocator.allocate(1),
            Some(Allocation { offset: 6, size: 1 })
        );
        eprintln!("{}", allocator.tree.dot());
    }
}
