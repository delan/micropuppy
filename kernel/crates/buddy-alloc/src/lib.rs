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
        // a tree with depth 0 has a single node, and is just a boolean
        assert!(depth >= 1, "tree must have depth of at least 1");

        // we must be able to store a complete tree's worth of edges
        let edges = storage.view_bits_mut();
        let num_nodes = (1 << (depth + 1)) - 1;
        let num_edges = num_nodes - 1; // every node except the root has one edge
        assert!(
            edges.len() >= num_edges,
            "storage must be at least {num_edges} bits wide to store a tree of depth {depth}"
        );

        // initially, every edge is present (i.e. every block is unallocated)
        edges.fill(true);

        Self { edges, depth }
    }

    pub fn allocate(&mut self, size: usize) -> Option<Block> {
        let height = match size {
            0 => return None,
            1 => 0,
            _ => (size - 1).ilog2() as usize + 1,
        };
        let depth = self.depth - height;

        // find a free node at the correct depth
        fn find(tree: &Tree, node: NodeIndex, depth: usize) -> Option<NodeIndex> {
            if node.depth() == depth && tree.is_node_free(node) {
                return Some(node);
            }

            let (left, left_edge) = node.left();
            if tree.is_edge_free(left_edge) {
                if let Some(left) = find(tree, left, depth) {
                    return Some(left);
                }
            }

            let (right, right_edge) = node.right();
            if tree.is_edge_free(right_edge) {
                if let Some(right) = find(tree, right, depth) {
                    return Some(right);
                }
            }

            None
        }

        let node = find(self, NodeIndex::root(), depth)?;
        dbg!(node, node.depth(), node.offset());

        let mut blblb = Some(node);
        while let Some(bbb) = blblb {
            if let Some((parent, parent_edge)) = bbb.parent() {
                self.edges.set(parent_edge.0, false);

                if !self.is_edge_free(parent_edge.buddy()) {
                    blblb = Some(parent);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        self.edges.set(node.parent().unwrap().1 .0, false);

        // let mut blorp = Some((EdgeIndex(node.0 - 1), node));
        // while let Some((blap, blop)) = blorp {
        //     self.edges.set(blap.0, false);
        //     if self.is_edge_free(blap.buddy()) {
        //         break;
        //     }

        //     blorp = blop.parent();
        // }

        Some(Block {
            offset: node.offset() << (self.depth - node.depth()),
            size: 1 << height,
        })
    }

    fn is_node_free(&self, node: NodeIndex) -> bool {
        if node.depth() != self.depth {
            let (_, left_edge) = node.left();
            let (_, right_edge) = node.right();

            self.is_edge_free(left_edge) && self.is_edge_free(right_edge)
        } else {
            let (_, parent_edge) = node.parent().unwrap();
            self.is_edge_free(parent_edge)
        }
    }

    fn is_edge_free(&self, edge: EdgeIndex) -> bool {
        let num_nodes = (1 << (self.depth + 1)) - 1;
        let num_edges = num_nodes - 1;

        edge.0 < num_edges && self.edges[edge.0]
    }

    fn nodes(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        let num_nodes = (1 << (self.depth + 1)) - 1;

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

    fn parent(self) -> Option<(NodeIndex, EdgeIndex)> {
        if self.0 != 0 {
            let parent = NodeIndex((self.0 - 1) / 2);
            let parent_edge = EdgeIndex(self.0 - 1);

            Some((parent, parent_edge))
        } else {
            None
        }
    }

    fn left(self) -> (NodeIndex, EdgeIndex) {
        let left = NodeIndex(2 * self.0 + 1);
        let left_edge = EdgeIndex(2 * self.0);

        (left, left_edge)
    }

    fn right(self) -> (NodeIndex, EdgeIndex) {
        let right = NodeIndex(2 * self.0 + 2);
        let right_edge = EdgeIndex(2 * self.0 + 1);

        (right, right_edge)
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

            let (left, left_edge) = node.left();
            if tree.is_edge_free(left_edge) {
                writeln!(f, "  n{} -> n{};", node.0, left.0)?;
            }

            let (right, right_edge) = node.right();
            if tree.is_edge_free(right_edge) {
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
        let mut tree = Tree::new(&mut tree, 3);
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

        assert_eq!(tree.allocate(3), Some(Block { offset: 4, size: 4 }));
        eprintln!("{}", tree.dot());

        // assert_eq!(tree.allocate(1), Some(0));
        // assert_eq!(tree.allocate(1), Some(1));
        // assert_eq!(tree.allocate(1), Some(2));
        // assert_eq!(tree.allocate(2), Some(4));
    }
}
