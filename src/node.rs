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
    /// add_edge adds an edge to the edges maintaining sorted order
    fn add_edge(&self, edge: Edge<T>) {
        let insert_idx = self
            .0
            .read()
            .binary_search_by(|e| e.label.cmp(&edge.label))
            .unwrap_or_else(|idx| idx);
        self.0.write().insert(insert_idx, edge);
    }

    /// replace_edge replaces the node of the edge with the same label
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

    /// get_edge return the index and node of the edge with the given label
    fn get_edge(&self, label: u8) -> Option<(usize, Arc<Node<T>>)> {
        let self_edges = self.0.read();
        let edge_idx = self_edges
            .as_slice()
            .binary_search_by(|e| e.label.cmp(&label))
            .unwrap_or_else(|idx| idx);
        if edge_idx < self_edges.len() && self_edges[edge_idx].label == label {
            let node = self_edges[edge_idx].node.clone();
            Some((edge_idx, node))
        } else {
            None
        }
    }

    /// get_lower_bound_edge returns the index and node of the lowest edge with label >= given label
    fn get_lower_bound_edge(&self, label: u8) -> Option<(usize, Arc<Node<T>>)> {
        let self_edges = self.0.read();
        let edge_idx = self_edges
            .as_slice()
            .binary_search_by(|e| e.label.cmp(&label))
            .unwrap_or_else(|idx| idx);
        if edge_idx < self_edges.len() {
            let node = self_edges[edge_idx].node.clone();
            Some((edge_idx, node))
        } else {
            None
        }
    }

    /// delete_edge removes the edge with the given label
    fn delete_edge(&self, label: u8) {
        let self_edges = self.0.read();
        let self_edges_slice = self_edges.as_slice();
        let edge_idx = self_edges_slice
            .binary_search_by(|e| e.label.cmp(&label))
            .unwrap_or_else(|idx| idx);
        if edge_idx < self_edges_slice.len() && self_edges_slice[edge_idx].label == label {
            drop(self_edges); // release read lock before acquiring write lock
            self.0.write().remove(edge_idx);
        }
    }

    /// is_empty returns true if there are no edges
    fn is_empty(&self) -> bool {
        self.0.read().is_empty()
    }

    /// first returns the first edge's node if exists
    fn first(&self) -> Option<Arc<Node<T>>> {
        let self_edges = self.0.read();
        if !self_edges.is_empty() {
            Some(self_edges[0].node.clone())
        } else {
            None
        }
    }

    /// last returns the last edge's node if exists
    fn last(&self) -> Option<Arc<Node<T>>> {
        let self_edges = self.0.read();
        if !self_edges.is_empty() {
            Some(self_edges[self_edges.len() - 1].node.clone())
        } else {
            None
        }
    }
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
    pub(crate) prefix: String,

    // used to store possible leaf
    pub(crate) leaf: RwLock<Option<Arc<LeafNode<T>>>>,

    // edges to child nodes
    pub(crate) edges: Edges<T>,
}

impl<T: NodeValue> Clone for Node<T> {
    fn clone(&self) -> Self {
        Self {
            prefix: self.prefix.clone(),
            leaf: RwLock::new(self.leaf.read().clone()),
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
            let l1 = self.leaf.read();
            let l1 = l1.as_ref();
            let l2 = other.leaf.read();
            let l2 = l2.as_ref();
            if let (Some(l1), Some(l2)) = (l1, l2) {
                return l1.eq(l2);
            } else if l1.is_none() && l2.is_none() {
                return true;
            }
        }
        false
    }
}

impl<T: NodeValue> Eq for Node<T> {}

impl<T: NodeValue> Node<T> {
    /// new creates a new node with the given prefix and optional leaf node
    pub(crate) fn new(prefix: &str, leaf: Option<LeafNode<T>>) -> Self {
        Self {
            prefix: prefix.to_string(),
            leaf: RwLock::new(leaf.map(|l| Arc::new(l))),
            ..Default::default()
        }
    }

    /// is_leaf returns true if the node is a leaf node.
    /// This should only be used internally as a Node is immutable from outside.
    pub(crate) fn is_leaf(&self) -> bool {
        self.leaf.read().is_some()
    }

    /// replace_leaf replaces the leaf node
    pub(crate) fn replace_leaf(&self, leaf: Option<LeafNode<T>>) {
        let mut write_guard = self.leaf.write();
        let leaf_node = leaf.map(|l| Arc::new(l));
        *write_guard = leaf_node;
    }

    /// add_edge adds an edge to the node
    pub(crate) fn add_edge(&self, edge: Edge<T>) {
        self.edges.add_edge(edge);
    }

    /// replace_edge replaces the node of the edge with the same label
    pub(crate) fn replace_edge(&self, edge: Edge<T>) {
        self.edges.replace_edge(edge);
    }

    /// get_edge return the index and node of the edge with the given label
    pub(crate) fn get_edge(&self, label: u8) -> Option<(usize, Arc<Node<T>>)> {
        self.edges.get_edge(label)
    }

    /// get_lower_bound_edge returns the index and node of the lowest edge with label >= given label
    pub(crate) fn get_lower_bound_edge(&self, label: u8) -> Option<(usize, Arc<Node<T>>)> {
        self.edges.get_lower_bound_edge(label)
    }

    /// delete_edge removes the edge with the given label
    pub(crate) fn delete_edge(&self, label: u8) {
        self.edges.delete_edge(label);
    }

    /// get returns the value associated with the given key if exists
    pub fn get(&self, key: &str) -> Option<T> {
        let mut search_bytes = key.as_bytes();
        let mut current_node: Option<Arc<Node<T>>> = None;

        loop {
            let node = match current_node.as_ref() {
                Some(n) => n,
                None => self,
            };

            if search_bytes.is_empty() {
                if node.is_leaf() {
                    let value = node.leaf.read().as_ref().unwrap().value.clone();
                    return Some(value);
                }
                break;
            }

            let node = match node.get_edge(search_bytes[0]) {
                Some((_, n)) => {
                    current_node.replace(n.clone());
                    n
                }
                None => break,
            };

            if search_bytes.starts_with(node.prefix.as_str().as_bytes()) {
                search_bytes = &search_bytes[node.prefix.len()..];
            } else {
                break;
            }
        }
        None
    }

    /// longest_prefix returns the key and value with the longest prefix match for the given key
    pub fn longest_prefix(&self, key: &str) -> Option<(String, T)> {
        let mut last: Option<Arc<LeafNode<T>>> = None;
        let mut search_bytes = key.as_bytes();
        let mut current_node: Option<Arc<Node<T>>> = None;

        loop {
            let node = match current_node.as_ref() {
                Some(n) => n,
                None => self,
            };

            if node.is_leaf() {
                last.replace(node.leaf.read().as_ref().unwrap().clone());
            }

            if search_bytes.is_empty() {
                break;
            }

            let node = match node.get_edge(search_bytes[0]) {
                Some((_, n)) => {
                    current_node.replace(n.clone());
                    n
                }
                None => break,
            };

            if search_bytes.starts_with(node.prefix.as_str().as_bytes()) {
                search_bytes = &search_bytes[node.prefix.len()..];
            } else {
                break;
            }
        }

        match last {
            // TODO: need to optimise to return &str instead of String
            // consider using [FastStr]
            Some(leaf) => Some((leaf.key.clone(), leaf.value.clone())),
            None => None,
        }
    }

    /// minimum returns the key and value with the minimum key in the subtree.
    pub fn minimum(&self) -> Option<(String, T)> {
        let mut current_node: Option<Arc<Node<T>>> = None;
        loop {
            let node = match current_node.as_ref() {
                Some(n) => n,
                None => self,
            };

            if node.is_leaf() {
                let leaf_node = node.leaf.read();
                let leaf_node = leaf_node.as_ref().unwrap();
                return Some((leaf_node.key.clone(), leaf_node.value.clone()));
            }

            match node.edges.first() {
                Some(first_edge_node) => {
                    current_node.replace(first_edge_node);
                }
                None => break,
            }
        }
        None
    }

    /// maximum returns the key and value with the maximum key in the subtree.
    pub fn maximum(&self) -> Option<(String, T)> {
        let mut current_node: Option<Arc<Node<T>>> = None;
        loop {
            let node = match current_node.as_ref() {
                Some(n) => n,
                None => self,
            };

            if let Some(last_edge_node) = node.edges.last() {
                current_node.replace(last_edge_node);
                continue;
            }

            if node.is_leaf() {
                let leaf_node = node.leaf.read();
                let leaf_node = leaf_node.as_ref().unwrap();
                return Some((leaf_node.key.clone(), leaf_node.value.clone()));
            } else {
                break;
            }
        }
        None
    }
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
