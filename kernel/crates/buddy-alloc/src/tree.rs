use core::{fmt, iter};

use bitvec::prelude::*;

#[derive(Debug)]
pub struct Tree<'s> {
    storage: &'s mut BitSlice<u8, Msb0>,
    depth: usize,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Allocation {
    pub offset: usize,
    pub size: usize,
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

        // initially, every node is unallocated and available
        storage.fill(false);

        Self { storage, depth }
    }

    pub fn allocate(&mut self, size: usize) -> Option<Allocation> {
        eprintln!("allocating size {size}");

        // determine node height and depth for requested allocation
        let height = match size {
            0 => return None,
            1 => 0,
            _ => (size - 1).ilog2() as usize + 1,
        };
        let depth = self.depth - height;

        // find an available node with the requested depth
        let node = self.preorder(|node| {
            eprintln!("preorder visiting {node:?}");

            let node_is_requested_depth = node.depth() == depth;
            match self.state(node) {
                NodeState::Full | NodeState::Allocated => Action::Skip,
                NodeState::Subdivided if node_is_requested_depth => Action::Skip,
                NodeState::Free if node_is_requested_depth => Action::Yield(node),
                _ => Action::Descend,
            }
        });

        // update the tree and generate the actual allocation offset and size
        node.map(|node| {
            // mark the node as allocated
            self.set_state(node, NodeState::Allocated);

            // we know the node corresponding to our allocation has just gone from Free to
            // Allocated. we need to do two things:
            // - mark every node in our node's path to the root as, at minimum, Subdivided
            //   - a node with one allocated child becomes unavailable
            //   - a node with two allocated children becomes allocated
            // - as we climb the tree, if our node's buddy (sibling) is also allocated:
            //   - both the children of the node's parent must be allocated
            //   - thus, the parent should be marked as allocated too
            let mut buddies = self.buddies(node);

            // mark parents as allocated until we reach a node where our buddy isn't allocated
            for (buddy, parent) in &mut buddies {
                if self.state(buddy).has_reachable_unallocated_children() {
                    // since the item has been consumed from the iterator, we need to mark the
                    // parent as unavailable here otherwise it will be missed by the loop below
                    self.set_state(parent, NodeState::Subdivided);
                    break;
                }

                self.set_state(parent, NodeState::Full);
            }

            // mark remaining parents as subdivided
            for (_, parent) in &mut buddies {
                self.set_state(parent, NodeState::Subdivided);
            }

            Allocation {
                offset: node.offset() << height,
                size: 1 << height,
            }
        })
    }

    pub fn free(&mut self, offset: usize) {
        let node = self.preorder(|node| {
            eprintln!("preorder visiting {node:?} -> {:?}", self.state(node));

            match self.state(node) {
                NodeState::Free => Action::Skip,
                NodeState::Subdivided | NodeState::Full => Action::Descend,
                NodeState::Allocated => {
                    let height = dbg!(self.depth - node.depth());
                    if dbg!(node.offset() << height) == dbg!(offset) {
                        Action::Yield(node)
                    } else {
                        Action::Descend
                    }
                }
            }
        });

        dbg!(node);

        // if we couldn't find the node, we've either been passed garbage or we're experiencing a
        // double free
        let node = node.expect("allocation being freed should be allocated (double free?)");

        // mark the node as available
        self.set_state(node, NodeState::Free);

        // we know the node corresponding to our allocation has just gone from Allocated to Free. we
        // need to do two things:
        // - mark every node in our node's path to the root as, at minimum, Subdivided
        //   - a node with no allocated children becomes available
        //   - a node with one allocated child becomes unavailable
        let mut buddies = self.buddies(node);

        // mark parents as Free until we reach a node where our buddy isn't available
        for (buddy, parent) in &mut buddies {
            if self.state(buddy) != NodeState::Free {
                // since the item has been consumed from the iterator, we need to mark the
                // parent as unavailable here otherwise it will be missed by the loop below
                self.set_state(parent, NodeState::Subdivided);
                break;
            }

            self.set_state(parent, NodeState::Free);
        }

        // mark remaining parents as unavailable
        for (_, parent) in &mut buddies {
            self.set_state(parent, NodeState::Subdivided);
        }
    }

    fn preorder<T>(&self, mut visitor: impl FnMut(NodeIndex) -> Action<T>) -> Option<T> {
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

    fn state(&self, node: NodeIndex) -> NodeState {
        assert!(self.has_node(node));

        let bit0 = self.storage[2 * node.0];
        let bit1 = self.storage[2 * node.0 + 1];

        match (bit0, bit1) {
            (false, false) => NodeState::Free,
            (false, true) => NodeState::Subdivided,
            (true, false) => NodeState::Full,
            (true, true) => NodeState::Allocated,
        }
    }

    fn set_state(&mut self, node: NodeIndex, state: NodeState) {
        assert!(self.has_node(node));

        let (bit0, bit1) = match state {
            NodeState::Free => (false, false),
            NodeState::Subdivided => (false, true),
            NodeState::Full => (true, false),
            NodeState::Allocated => (true, true),
        };

        self.storage.set(2 * node.0, bit0);
        self.storage.set(2 * node.0 + 1, bit1);
    }

    fn nodes(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        (0..self.node_count()).map(NodeIndex)
    }

    fn buddies(&self, node: NodeIndex) -> impl Iterator<Item = (NodeIndex, NodeIndex)> {
        let mut node = node;

        iter::from_fn(move || {
            let parent = node.parent();
            let buddy = node.buddy();

            if let Some(parent) = parent {
                node = parent;
            }

            buddy.zip(parent)
        })
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NodeState {
    /// Has no allocated children.
    Free,
    /// Has one or more allocated children and one or more reachable, unallocated children.
    Subdivided,
    /// Has no reachable, unallocated children.
    Full,
    /// Has been allocated.
    Allocated,
}

impl NodeState {
    fn has_reachable_unallocated_children(self) -> bool {
        match self {
            Self::Free | Self::Subdivided => true,
            Self::Full | Self::Allocated => false,
        }
    }
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

    pub fn buddy(self) -> Option<Self> {
        (!self.is_root()).then(|| Self(((self.0 - 1) ^ 1) + 1))
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
            const GREEN: &str = "#9dd5c0";
            const BLUE: &str = "#27a4dd";
            const RED: &str = "#f1646c";
            let (fillcolor, shape) = match tree.state(node_index) {
                NodeState::Free => (GREEN, "circle"),
                NodeState::Subdivided => (BLUE, "Mcircle"),
                NodeState::Full => (RED, "Msquare"),
                NodeState::Allocated => (RED, "square"),
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
    fn allocate() {
        let mut storage = [0; 4];
        let mut tree = Tree::new(&mut storage, 3);

        // node index 7
        assert_eq!(tree.allocate(1), Some(Allocation { offset: 0, size: 1 }));
        eprintln!("{}", tree.dot());

        // node index 8
        assert_eq!(tree.allocate(1), Some(Allocation { offset: 1, size: 1 }));
        eprintln!("{}", tree.dot());

        // node index 9
        assert_eq!(tree.allocate(1), Some(Allocation { offset: 2, size: 1 }));
        eprintln!("{}", tree.dot());

        // node index 5
        assert_eq!(tree.allocate(2), Some(Allocation { offset: 4, size: 2 }));
        eprintln!("{}", tree.dot());

        // node index 10
        assert_eq!(tree.allocate(1), Some(Allocation { offset: 3, size: 1 }));
        eprintln!("{}", tree.dot());

        // node index 13
        assert_eq!(tree.allocate(1), Some(Allocation { offset: 6, size: 1 }));
        eprintln!("{}", tree.dot());

        // node index 14
        assert_eq!(tree.allocate(1), Some(Allocation { offset: 7, size: 1 }));
        eprintln!("{}", tree.dot());

        assert_eq!(tree.allocate(1), None);
    }

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
        assert_eq!(node.buddy(), None);
        assert_eq!(node.left_child(), NodeIndex(1));
        assert_eq!(node.right_child(), NodeIndex(2));
        assert_eq!(node.depth(), 0);
        assert_eq!(node.offset(), 0);

        // depth 1, height 2
        let node = NodeIndex(1);
        assert_eq!(node.parent(), Some(NodeIndex(0)));
        assert_eq!(node.buddy(), Some(NodeIndex(2)));
        assert_eq!(node.left_child(), NodeIndex(3));
        assert_eq!(node.right_child(), NodeIndex(4));
        assert_eq!(node.depth(), 1);
        assert_eq!(node.offset(), 0);

        let node = NodeIndex(2);
        assert_eq!(node.parent(), Some(NodeIndex(0)));
        assert_eq!(node.buddy(), Some(NodeIndex(1)));
        assert_eq!(node.left_child(), NodeIndex(5));
        assert_eq!(node.right_child(), NodeIndex(6));
        assert_eq!(node.depth(), 1);
        assert_eq!(node.offset(), 1);

        // depth 2, height 1
        let node = NodeIndex(3);
        assert_eq!(node.parent(), Some(NodeIndex(1)));
        assert_eq!(node.buddy(), Some(NodeIndex(4)));
        assert_eq!(node.left_child(), NodeIndex(7));
        assert_eq!(node.right_child(), NodeIndex(8));
        assert_eq!(node.depth(), 2);
        assert_eq!(node.offset(), 0);

        let node = NodeIndex(4);
        assert_eq!(node.parent(), Some(NodeIndex(1)));
        assert_eq!(node.buddy(), Some(NodeIndex(3)));
        assert_eq!(node.left_child(), NodeIndex(9));
        assert_eq!(node.right_child(), NodeIndex(10));
        assert_eq!(node.depth(), 2);
        assert_eq!(node.offset(), 1);

        let node = NodeIndex(5);
        assert_eq!(node.parent(), Some(NodeIndex(2)));
        assert_eq!(node.buddy(), Some(NodeIndex(6)));
        assert_eq!(node.left_child(), NodeIndex(11));
        assert_eq!(node.right_child(), NodeIndex(12));
        assert_eq!(node.depth(), 2);
        assert_eq!(node.offset(), 2);

        let node = NodeIndex(6);
        assert_eq!(node.parent(), Some(NodeIndex(2)));
        assert_eq!(node.buddy(), Some(NodeIndex(5)));
        assert_eq!(node.left_child(), NodeIndex(13));
        assert_eq!(node.right_child(), NodeIndex(14));
        assert_eq!(node.depth(), 2);
        assert_eq!(node.offset(), 3);

        // depth 3, height 0
        let node = NodeIndex(7);
        assert_eq!(node.parent(), Some(NodeIndex(3)));
        assert_eq!(node.buddy(), Some(NodeIndex(8)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 0);

        let node = NodeIndex(8);
        assert_eq!(node.parent(), Some(NodeIndex(3)));
        assert_eq!(node.buddy(), Some(NodeIndex(7)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 1);

        let node = NodeIndex(9);
        assert_eq!(node.parent(), Some(NodeIndex(4)));
        assert_eq!(node.buddy(), Some(NodeIndex(10)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 2);

        let node = NodeIndex(10);
        assert_eq!(node.parent(), Some(NodeIndex(4)));
        assert_eq!(node.buddy(), Some(NodeIndex(9)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 3);

        let node = NodeIndex(11);
        assert_eq!(node.parent(), Some(NodeIndex(5)));
        assert_eq!(node.buddy(), Some(NodeIndex(12)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 4);

        let node = NodeIndex(12);
        assert_eq!(node.parent(), Some(NodeIndex(5)));
        assert_eq!(node.buddy(), Some(NodeIndex(11)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 5);

        let node = NodeIndex(13);
        assert_eq!(node.parent(), Some(NodeIndex(6)));
        assert_eq!(node.buddy(), Some(NodeIndex(14)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 6);

        let node = NodeIndex(14);
        assert_eq!(node.parent(), Some(NodeIndex(6)));
        assert_eq!(node.buddy(), Some(NodeIndex(13)));
        assert_eq!(node.depth(), 3);
        assert_eq!(node.offset(), 7);
    }
}
