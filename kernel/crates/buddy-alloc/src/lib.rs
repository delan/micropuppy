// #![cfg_attr(not(test), no_std)]
use core::{fmt, iter};

use bitvec::prelude::*;

pub struct Tree<'s> {
    edges: &'s mut BitSlice<u8, Msb0>,
    depth: usize,
}

#[derive(PartialEq, Debug)]
pub struct Block {
    offset: usize,
    size: usize,
}

impl<'s> Tree<'s> {
    pub fn new(storage: &'s mut [u8], depth: usize) -> Self {
        let sself = Self {
            edges: storage.view_bits_mut(),
            depth,
        };

        // a tree of depth 0 contains no nodes, and a tree of depth 1 is a boolean (and has no
        // edges)
        assert!(depth >= 2, "tree must be at least depth 2");

        // we must be able to store a complete tree's worth of edges
        let num_nodes = (1 << depth) - 1;
        let num_edges = num_nodes - 1;
        assert!(
            sself.edges.len() >= num_edges,
            "storage must be at least {num_edges} bits wide to store a tree of depth {depth}"
        );

        // initially, every edge is present (i.e. every block is unallocated)
        sself.edges.fill(true);

        sself
    }

    pub fn allocate(&mut self, size: usize) -> Option<Block> {
        let order = match size {
            0 => return None,
            1 => 0,
            _ => (size - 1).ilog2() as usize + 1,
        };
        let depth = self.depth - order - 1;

        // find a reachable node of the correct size
        fn find(
            tree: &Tree,
            node: NodeIndex,
            predicate: &impl Fn(NodeIndex) -> bool,
        ) -> Option<NodeIndex> {
            if predicate(node) {
                return Some(node);
            }

            let (left_edge, left) = node.left_edge();
            if tree.is_edge_present(left_edge) {
                if let Some(left) = find(tree, left, predicate) {
                    return Some(left);
                }
            }

            let (right_edge, right) = node.right_edge();
            if tree.is_edge_present(right_edge) {
                if let Some(right) = find(tree, right, predicate) {
                    return Some(right);
                }
            }

            None
        }

        let node = find(self, NodeIndex::root(), &|node| {
            node.depth() == depth
                && (self.is_edge_present(node.left_edge().0)
                    && self.is_edge_present(node.right_edge().0)
                    || order == 0)
        })?;
        dbg!(node, node.depth(), node.offset());

        let mut blorp = Some((EdgeIndex(node.0 - 1), node));
        while let Some((blap, blop)) = blorp {
            self.edges.set(blap.0, false);
            if self.is_edge_present(blap.buddy()) {
                break;
            }

            blorp = blop.parent();
        }

        Some(Block {
            offset: node.offset() << (self.depth - node.depth() - 1),
            size,
        })
    }

    fn is_edge_present(&self, edge: EdgeIndex) -> bool {
        let num_nodes = (1 << self.depth) - 1;
        let num_edges = num_nodes - 1;

        edge.0 < num_edges && self.edges[edge.0]
    }

    fn nodes(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        let num_nodes = (1 << self.depth) - 1;

        (0..num_nodes).map(NodeIndex)
    }

    fn dot(&self) -> Dot {
        Dot(self)
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
struct NodeIndex(usize);

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
struct EdgeIndex(usize);

impl NodeIndex {
    fn root() -> NodeIndex {
        NodeIndex(0)
    }

    fn parent(self) -> Option<(EdgeIndex, NodeIndex)> {
        if self.0 != 0 {
            Some((EdgeIndex(self.0 - 1), NodeIndex((self.0 - 1) / 2)))
        } else {
            None
        }
    }

    fn left_edge(self) -> (EdgeIndex, NodeIndex) {
        (EdgeIndex(2 * self.0), NodeIndex(2 * self.0 + 1))
    }

    fn right_edge(self) -> (EdgeIndex, NodeIndex) {
        (EdgeIndex(2 * self.0 + 1), NodeIndex(2 * self.0 + 2))
    }

    fn depth(self) -> usize {
        (self.0 + 1).ilog2() as usize
    }

    fn offset(self) -> usize {
        self.0 + 1 - (1 << self.depth())
    }
}

impl EdgeIndex {
    fn buddy(self) -> EdgeIndex {
        Self(self.0 ^ 1)
    }
}

struct Dot<'t, 's>(&'t Tree<'s>);

impl fmt::Display for Dot<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tree = self.0;

        writeln!(f, "digraph {{")?;
        for node in tree.nodes() {
            writeln!(f, "  n{};", node.0)?;

            let (left_edge, left) = node.left_edge();
            if tree.is_edge_present(left_edge) {
                writeln!(f, "  n{} -> n{};", node.0, left.0)?;
            }

            let (right_edge, right) = node.right_edge();
            if tree.is_edge_present(right_edge) {
                writeln!(f, "  n{} -> n{};", node.0, right.0)?;
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
    fn allocate() {
        let mut tree = [0; 2];
        let mut tree = Tree::new(&mut tree, 4);
        //        0         size = 8
        //    0       4     size = 4
        //  0   2   4   6   size = 2
        // 0 1 2 3 4 5 6 7  size = 1

        eprintln!("{}", tree.dot());

        assert_eq!(tree.allocate(1), Some(Block { offset: 0, size: 1 }));
        eprintln!("{}", tree.dot());

        assert_eq!(tree.allocate(1), Some(Block { offset: 1, size: 1 }));
        eprintln!("{}", tree.dot());

        assert_eq!(tree.allocate(1), Some(Block { offset: 2, size: 1 }));
        eprintln!("{}", tree.dot());

        assert_eq!(tree.allocate(2), Some(Block { offset: 4, size: 2 }));
        eprintln!("{}", tree.dot());

        // assert_eq!(tree.allocate(1), Some(0));
        // assert_eq!(tree.allocate(1), Some(1));
        // assert_eq!(tree.allocate(1), Some(2));
        // assert_eq!(tree.allocate(2), Some(4));
    }
}
