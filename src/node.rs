use std::{hash::Hash, sync::Arc};

mod node_test;
use parking_lot::RwLock;

use crate::utils::NodeValue;

#[derive(Debug, Default, Clone, Hash, Eq)]
pub struct Edge<T>
where
    T: NodeValue,
{
    pub(crate) label: u8,
    pub(crate) node: Arc<Node<T>>,
}

impl<T: NodeValue> PartialOrd for Edge<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.label.cmp(&other.label))
    }
}

impl<T: NodeValue> PartialEq for Edge<T> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

#[derive(Debug, Default)]
pub struct Edges<T>(RwLock<Vec<Edge<T>>>)
where
    T: NodeValue;

impl<T: NodeValue> Clone for Edges<T> {
    fn clone(&self) -> Self {
        Self(RwLock::new(self.0.read().clone()))
    }
}

impl<T: NodeValue> Hash for Edges<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.read().hash(state);
    }
}

impl<T: NodeValue> PartialEq for Edges<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.read().as_slice().eq(other.0.read().as_slice())
    }
}

impl<T: NodeValue> Edges<T> {
    // add_edge adds an edge to the edges maintaining sorted order
    fn add_edge(&self, edge: Edge<T>) {
        let insert_idx = self
            .0
            .read()
            .binary_search_by(|e| e.label.cmp(&edge.label))
            .unwrap_or_else(|idx| idx);
        self.0.write().insert(insert_idx, edge);
    }

    // replace_edge replaces the node of the edge with the same label
    fn replace_edge(&self, edge: Edge<T>) {
        let self_edges = self.0.read();
        let self_edges_slice = self_edges.as_slice();
        let edge_idx = self_edges_slice
            .binary_search_by(|e| e.label.cmp(&edge.label))
            .unwrap_or_else(|idx| idx);
        if edge_idx < self_edges_slice.len() && self_edges_slice[edge_idx].label == edge.label {
            drop(self_edges); // release read lock before acquiring write lock
            self.0.write()[edge_idx].node = edge.node;
        } else {
            panic!("replace missing edge");
        }
    }

    // get_edge return the index and node of the edge with the given label
    fn get_edge(&self, label: u8) -> Option<(usize, Arc<Node<T>>)> {
        let self_edges = self.0.read();
        let self_edges_slice = self_edges.as_slice();
        let edge_idx = self_edges_slice
            .binary_search_by(|e| e.label.cmp(&label))
            .unwrap_or_else(|idx| idx);

        if edge_idx < self_edges_slice.len() && self_edges_slice[edge_idx].label == label {
            let node = self_edges[edge_idx].node.clone();
            Some((edge_idx, node))
        } else {
            None
        }
    }

    // fn get_lower_bound_edge(&self, label: u8) -> Option<(usize, &Node<T>)> {
    //     let edge_idx = self
    //         .edges
    //         .binary_search_by(|e| e.label.cmp(&label))
    //         .unwrap_or_else(|idx| idx);
    //     if edge_idx < self.edges.len() {
    //         Some((edge_idx, &self.edges[edge_idx].node))
    //     } else {
    //         None
    //     }
    // }

    // fn delete_edge(&mut self, label: u8) {
    //     let edge_idx = self
    //         .edges
    //         .binary_search_by(|e| e.label.cmp(&label))
    //         .unwrap_or_else(|idx| idx);
    //     if edge_idx < self.edges.len() && self.edges[edge_idx].label == label {
    //         self.edges.remove(edge_idx);
    //     }
    // }
}

/// An immutable node in the radix tree, which may contains a value if it is a leaf node.
/// It also contains edges to its child nodes if exists.
#[derive(Debug, Default)]
pub struct Node<T>
where
    T: NodeValue,
{
    // TODO: add Node update signal
    // TODO: optimise this with Vec<u8>

    // prefix to reach this node
    prefix: String,

    // used to store possible leaf
    leaf: Option<RwLock<LeafNode<T>>>,

    // edges to child nodes
    edges: Edges<T>,
}

impl<T: NodeValue> Clone for Node<T> {
    fn clone(&self) -> Self {
        Self {
            prefix: self.prefix.clone(),
            leaf: self.leaf.as_ref().map(|l| RwLock::new(l.read().clone())),
            edges: self.edges.clone(),
        }
    }
}

impl<T: NodeValue> Hash for Node<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.prefix.hash(state);
    }
}

impl<T: NodeValue> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.prefix == other.prefix && self.edges == other.edges {
            if let (Some(l1), Some(l2)) = (&self.leaf, &other.leaf) {
                return l1.read().eq(&l2.read());
            } else if self.leaf.is_none() && other.leaf.is_none() {
                return true;
            }
        }
        false
    }
}

impl<T: NodeValue> Eq for Node<T> {}

impl<T: NodeValue> Node<T> {
    pub(crate) fn is_leaf(&self) -> bool {
        self.leaf.is_some()
    }

    pub(crate) fn add_edge(&self, edge: Edge<T>) {
        self.edges.add_edge(edge);
    }

    pub(crate) fn replace_edge(&self, edge: Edge<T>) {
        self.edges.replace_edge(edge);
    }

    pub(crate) fn get_edge(&self, label: u8) -> Option<(usize, Arc<Node<T>>)> {
        self.edges.get_edge(label)
    }

    // pub fn get(&self, label: &str) -> Option<T> {
    //     let mut search = label;
    //     loop {
    //         if search.is_empty() {
    //             if self.is_leaf() && self.leaf.is_some() {
    //                 return Some(self.leaf.as_ref().unwrap().value.clone());
    //             }
    //             break;
    //         }

    //         if self.get_edge(search.as_bytes()[0]).is_none() {
    //             break;
    //         }

    //         if search.starts_with(self.prefix.as_str()) {
    //             search = &search[self.prefix.len()..];
    //         } else {
    //             break;
    //         }
    //     }
    //     None
    // }

    // pub fn longest_prefix(&self, key: &str) -> Option<(&str, T)> {
    //     let mut last: Option<&LeafNode<T>> = None;
    //     let mut search = key;
    //     loop {
    //         if self.is_leaf() {
    //             last = self.leaf.as_ref().map(|l| l.as_ref())
    //         }

    //         if search.is_empty() {
    //             break;
    //         }

    //         if self.get_edge(search.as_bytes()[0]).is_none() {
    //             break;
    //         }

    //         if search.starts_with(self.prefix.as_str()) {
    //             search = &search[self.prefix.len()..];
    //         } else {
    //             break;
    //         }
    //     }
    //     if last.is_some() {
    //         return Some((last.unwrap().key.as_str(), last.unwrap().value.clone()));
    //     }
    //     None
    // }

    // pub fn minimum(&self) -> Option<(&str, T)> {
    //     let mut current = self;
    //     loop {
    //         if current.is_leaf() {
    //             return Some((
    //                 current.leaf.as_ref().unwrap().key.as_str(),
    //                 current.leaf.as_ref().unwrap().value.clone(),
    //             ));
    //         }
    //         if !current.edges.is_empty() {
    //             current = &current.edges.first().unwrap().node;
    //         } else {
    //             break;
    //         }
    //     }
    //     None
    // }

    // pub fn maximum(&self) -> Option<(&str, T)> {
    //     let mut current = self;
    //     loop {
    //         if !current.edges.is_empty() {
    //             current = &current.edges.last().unwrap().node;
    //             continue;
    //         }
    //         if current.is_leaf() {
    //             return Some((
    //                 current.leaf.as_ref().unwrap().key.as_str(),
    //                 current.leaf.as_ref().unwrap().value.clone(),
    //             ));
    //         } else {
    //             break;
    //         }
    //     }
    //     None
    // }

    // pub fn into_node_iterator(self) -> impl Iterator<Item = (&'static str, T)> {
    //     NodeIterator {
    //         node: Some(Arc::new(self)),
    //         stack: Vec::new(),
    //     }
    // }
}

/// A leaf node represents the end of a key in the radix tree and holds the associated value.
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct LeafNode<T>
where
    T: NodeValue,
{
    pub(crate) value: T,
    pub(crate) key: String,
}
