use core::fmt;

use bitvec::prelude::*;

#[derive(Debug)]
pub struct Tree<'s> {
    storage: &'s mut BitSlice<u8, Msb0>,
    depth: usize,
}

#[derive(Debug)]
pub enum Action<T> {
    Yield(T),
    Skip,
    Descend,
}

impl<'s> Tree<'s> {
    pub fn new(storage: &'s mut [u8], depth: usize) -> Self {
        // a tree with depth 0 has a single node, and is just a boolean
        assert!(depth >= 1, "tree must have depth of at least 1");

        // we must be able to store a complete tree's worth of nodes
        let storage = storage.view_bits_mut();
        let node_count = (1 << (depth + 1)) - 1;
        let bits = node_count * 2;
        assert!(
            storage.len() >= bits,
            "storage must be at least {bits} bits wide to store a tree of depth {depth}"
        );

        let storage = &mut storage[0..bits];

        // initially, every node is unallocated
        storage.fill(false);

        Self { storage, depth }
    }

    pub fn preorder<T>(&self, mut visitor: impl FnMut(NodeIndex) -> Action<T>) -> Option<T> {
        fn preorder<T>(
            tree: &Tree,
            node: NodeIndex,
            visitor: &mut impl FnMut(NodeIndex) -> Action<T>,
        ) -> Option<T> {
            if !tree.has_node(node) {
                return None;
            }

            let action = visitor(node);
            match action {
                Action::Yield(value) => Some(value),
                Action::Skip => None,
                Action::Descend => {
                    let (left, right) = node.children();

                    preorder(tree, left, visitor).or_else(|| preorder(tree, right, visitor))
                }
            }
        }

        preorder(self, NodeIndex::root(), &mut visitor)
    }

    pub fn node(&self, node: NodeIndex) -> Node {
        assert!(self.has_node(node));

        let available = !self.storage[2 * node.0];
        let allocated = self.storage[2 * node.0 + 1];

        Node {
            available,
            allocated,
        }
    }

    pub fn mark_unavailable(&mut self, node: NodeIndex) {
        self.storage.set(2 * node.0, true);
    }

    pub fn allocate(&mut self, node: NodeIndex) {
        self.storage.set(2 * node.0 + 1, true);
    }

    fn nodes(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        (0..self.node_count()).map(NodeIndex)
    }

    fn has_node(&self, node: NodeIndex) -> bool {
        node.0 < self.node_count()
    }

    fn node_count(&self) -> usize {
        (1 << (self.depth + 1)) - 1
    }

    pub fn dot(&self) -> Dot {
        Dot(self)
    }
}

#[derive(Debug)]
pub struct Node {
    /// has unallocated children
    pub available: bool,
    /// allocated, or all children allocated
    pub allocated: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct NodeIndex(usize);

impl NodeIndex {
    fn root() -> Self {
        Self(0)
    }

    pub fn parent(self) -> Option<Self> {
        (!self.is_root()).then(|| Self((self.0 - 1) / 2))
    }

    pub fn children(self) -> (Self, Self) {
        let left = Self(2 * self.0 + 1);
        let right = Self(2 * self.0 + 2);

        (left, right)
    }

    pub fn left_child(self) -> Self {
        let (left, _) = self.children();

        left
    }

    pub fn right_child(self) -> Self {
        let (_, right) = self.children();

        right
    }

    pub fn depth(self) -> usize {
        (self.0 + 1).ilog2() as usize
    }

    pub fn offset(self) -> usize {
        self.0 + 1 - (1 << self.depth())
    }

    pub fn is_root(self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug)]
pub struct Dot<'t, 's>(&'t Tree<'s>);

impl fmt::Display for Dot<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tree = self.0;

        writeln!(f, "digraph {{")?;
        writeln!(f, "  node [style=filled, fixedsize=true];")?;
        for node_index in tree.nodes() {
            let node = tree.node(node_index);

            let fillcolor = if node.available {
                "#9dd5c0" // green
            } else {
                "#f1646c" // red
            };
            let shape = if node.allocated {
                "doublecircle"
            } else {
                "circle"
            };
            writeln!(
                f,
                "  n{} [fillcolor=\"{}\", shape=\"{}\"];",
                node_index.0, fillcolor, shape
            )?;

            let (left, right) = node_index.children();
            for child in [left, right] {
                if tree.has_node(child) {
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
    fn preorder_descend() {
        let mut storage = [0; 4];
        let mut tree = Tree::new(&mut storage, 3);

        let mut preorder = Vec::with_capacity(tree.node_count());
        let result = tree.preorder(|node| -> Action<()> {
            preorder.push(node);

            Action::Descend
        });

        assert_eq!(
            preorder,
            [0, 1, 3, 7, 8, 4, 9, 10, 2, 5, 11, 12, 6, 13, 14]
                .into_iter()
                .map(NodeIndex)
                .collect::<Vec<_>>()
        );
        assert_eq!(result, None);
    }

    #[test]
    fn preorder_skip() {
        let mut storage = [0; 4];
        let mut tree = Tree::new(&mut storage, 3);

        let mut preorder = Vec::with_capacity(tree.node_count());
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
                .map(NodeIndex)
                .collect::<Vec<_>>()
        );
        assert_eq!(result, None);
    }

    #[test]
    fn preorder_yield() {
        let mut storage = [0; 4];
        let mut tree = Tree::new(&mut storage, 3);

        let mut preorder = Vec::with_capacity(tree.node_count());
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
                .map(NodeIndex)
                .collect::<Vec<_>>()
        );
        assert_eq!(result, Some(NodeIndex(5)));
    }

    #[test]
    fn node_index() {
        // depth 0, height 3
        let node = NodeIndex(0);
        assert_eq!(node.parent(), None);
        assert_eq!(node.left_child(), NodeIndex(1));
        assert_eq!(node.right_child(), NodeIndex(2));
        assert_eq!(node.depth(), 0);
        assert_eq!(node.offset(), 0);

        // depth 1, height 2
        let node = NodeIndex(1);
        assert_eq!(node.parent(), Some(NodeIndex(0)));
        assert_eq!(node.left_child(), NodeIndex(3));
        assert_eq!(node.right_child(), NodeIndex(4));
        assert_eq!(node.depth(), 1);
        assert_eq!(node.offset(), 0);

        let node = NodeIndex(2);
        assert_eq!(node.parent(), Some(NodeIndex(0)));
        assert_eq!(node.left_child(), NodeIndex(5));
        assert_eq!(node.right_child(), NodeIndex(6));
        assert_eq!(node.depth(), 1);
        assert_eq!(node.offset(), 1);

        // depth 2, height 1
        let node = NodeIndex(3);
        assert_eq!(node.parent(), Some(NodeIndex(1)));
        assert_eq!(node.left_child(), NodeIndex(7));
        assert_eq!(node.right_child(), NodeIndex(8));
        assert_eq!(node.depth(), 2);
        assert_eq!(node.offset(), 0);

        let node = NodeIndex(4);
        assert_eq!(node.parent(), Some(NodeIndex(1)));
        assert_eq!(node.left_child(), NodeIndex(9));
        assert_eq!(node.right_child(), NodeIndex(10));
        assert_eq!(node.depth(), 2);
        assert_eq!(node.offset(), 1);

        let node = NodeIndex(5);
        assert_eq!(node.parent(), Some(NodeIndex(2)));
        assert_eq!(node.left_child(), NodeIndex(11));
        assert_eq!(node.right_child(), NodeIndex(12));
        assert_eq!(node.depth(), 2);
        assert_eq!(node.offset(), 2);

        let node = NodeIndex(6);
        assert_eq!(node.parent(), Some(NodeIndex(2)));
        assert_eq!(node.left_child(), NodeIndex(13));
        assert_eq!(node.right_child(), NodeIndex(14));
        assert_eq!(node.depth(), 2);
        assert_eq!(node.offset(), 3);

        // depth 3, height 0
        let node = NodeIndex(7);
        assert_eq!(node.parent(), Some(NodeIndex(3)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 0);

        let node = NodeIndex(8);
        assert_eq!(node.parent(), Some(NodeIndex(3)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 1);

        let node = NodeIndex(9);
        assert_eq!(node.parent(), Some(NodeIndex(4)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 2);

        let node = NodeIndex(10);
        assert_eq!(node.parent(), Some(NodeIndex(4)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 3);

        let node = NodeIndex(11);
        assert_eq!(node.parent(), Some(NodeIndex(5)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 4);

        let node = NodeIndex(12);
        assert_eq!(node.parent(), Some(NodeIndex(5)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 5);

        let node = NodeIndex(13);
        assert_eq!(node.parent(), Some(NodeIndex(6)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 6);

        let node = NodeIndex(14);
        assert_eq!(node.parent(), Some(NodeIndex(6)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 7);
    }
}
