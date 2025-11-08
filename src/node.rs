#[cfg(test)]
mod node_test;

use std::{hash::Hash, sync::Arc};

use parking_lot::RwLock;

use crate::utils::NodeValue;

#[derive(Debug, Default, Clone, Hash, Eq)]
pub struct Edge<T>
where
    T: NodeValue,
{
    pub(crate) label: u8,
    // TODO: re-check if we need deep clone here
    pub(crate) node: Arc<Node<T>>,
}

impl<T: NodeValue> PartialOrd for Edge<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.label.cmp(&other.label))
    }
}

impl<T: NodeValue> PartialEq for Edge<T> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label && self.node == other.node
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

impl<T: NodeValue> From<Vec<Edge<T>>> for Edges<T> {
    fn from(vec: Vec<Edge<T>>) -> Self {
        Self(RwLock::new(vec))
    }
}

impl<T: NodeValue> Edges<T> {
    /// Adds an edge to the edges while maintaining sorted order.
    fn add_edge(&self, edge: Edge<T>) {
        let insert_idx = self
            .0
            .read()
            .binary_search_by(|e| e.label.cmp(&edge.label))
            .unwrap_or_else(|idx| idx);
        self.0.write().insert(insert_idx, edge);
    }

    /// Replaces the node of the edge with the same label.
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

    /// Replaces the node of the edge at the given index.
    fn replace_edge_at(&self, index: usize, edge: Edge<T>) {
        let self_edges = self.0.read();
        let self_edges_slice = self_edges.as_slice();
        if index < self_edges_slice.len() && self_edges_slice[index].label == edge.label {
            drop(self_edges); // release read lock before acquiring write lock
            self.0.write()[index].node = edge.node;
        } else {
            panic!("replace edge at invalid index or label mismatch");
        }
    }

    /// Returns the index and node of the edge with the given label.
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

    /// Returns the node of the edge at the given index.
    fn get_edge_at(&self, index: usize) -> Option<Arc<Node<T>>> {
        let self_edges = self.0.read();
        if index < self_edges.len() {
            Some(self_edges[index].node.clone())
        } else {
            None
        }
    }

    /// Returns the index and node of the lowest edge with label >= given label.
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

    /// Deletes the edge with the given label.
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

    /// Returns true if there are no edges.
    fn is_empty(&self) -> bool {
        self.0.read().is_empty()
    }

    /// Returns the number of edges.
    fn len(&self) -> usize {
        self.0.read().len()
    }

    /// Returns the first edge's node if exists.
    fn first(&self) -> Option<Arc<Node<T>>> {
        let self_edges = self.0.read();
        if !self_edges.is_empty() {
            Some(self_edges[0].node.clone())
        } else {
            None
        }
    }

    /// Returns the last edge's node if exists.
    fn last(&self) -> Option<Arc<Node<T>>> {
        let self_edges = self.0.read();
        if !self_edges.is_empty() {
            Some(self_edges[self_edges.len() - 1].node.clone())
        } else {
            None
        }
    }

    /// Removes all edges data.
    fn clear(&self) {
        self.0.write().clear();
    }

    /// Removes all edges and resets allocated capacity.
    fn reset(&self) {
        self.0.write().clear();
        *self.0.write() = Vec::new();
    }

    /// Removes the last edge and returns it if exists.
    fn pop(&self) -> Option<Edge<T>> {
        self.0.write().pop()
    }

    /// Drains all edges from self and inserts them into other
    fn collect_into(&self, other: &Edges<T>) {
        let mut self_guard = self.0.write();
        let mut other_guard = other.0.write();

        let self_len = self_guard.len();
        let other_capacity = other_guard.capacity();
        if other_capacity < self_len {
            other_guard.reserve(self_len - other_capacity);
        }

        let self_iter = self_guard.drain(..).into_iter();
        other_guard.extend(self_iter);
    }

    /// Iterates over each edge and applies the given function
    fn for_each<F>(&self, f: F)
    where
        F: FnMut(&Edge<T>),
    {
        self.0.read().iter().for_each(f);
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

    // prefix ignored
    pub(crate) prefix: RwLock<String>,

    // used to store possible leaf
    pub(crate) leaf: RwLock<Option<Arc<LeafNode<T>>>>,

    // edges to child nodes
    pub(crate) edges: Edges<T>,
}

impl<T: NodeValue> Clone for Node<T> {
    fn clone(&self) -> Self {
        Self {
            prefix: RwLock::new(self.prefix.read().clone()),
            leaf: RwLock::new(self.leaf.read().clone()),
            edges: self.edges.clone(),
        }
    }
}

impl<T: NodeValue> Hash for Node<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.prefix.read().hash(state);
    }
}

impl<T: NodeValue> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.prefix.read().as_str() != other.prefix.read().as_str() {
            return false;
        }
        if self.leaf.read().as_ref() != other.leaf.read().as_ref() {
            return false;
        }
        return self.edges == other.edges;
    }
}

impl<T: NodeValue> Eq for Node<T> {}

impl<T: NodeValue> Node<T> {
    /// Creates a new node with the given prefix and optional leaf node.
    pub(crate) fn new(prefix: &str, leaf: Option<LeafNode<T>>) -> Self {
        Self {
            prefix: RwLock::new(prefix.to_string()),
            leaf: RwLock::new(leaf.map(|l| Arc::new(l))),
            ..Default::default()
        }
    }

    /// Returns true if the node is a leaf node.
    pub(crate) fn is_leaf(&self) -> bool {
        self.leaf.read().is_some()
    }

    /// Replaces the prefix of the node.
    pub(crate) fn replace_prefix(&self, prefix: &str) {
        let mut write_guard = self.prefix.write();
        *write_guard = prefix.to_string();
    }

    /// Replaces the leaf node.
    pub(crate) fn replace_leaf(&self, leaf: Option<LeafNode<T>>) {
        let mut write_guard = self.leaf.write();
        let leaf_node = leaf.map(|l| Arc::new(l));
        *write_guard = leaf_node;
    }

    /// Adds an edge to the node.
    pub(crate) fn add_edge(&self, edge: Edge<T>) {
        self.edges.add_edge(edge);
    }

    /// Replaces the node of the edge with the same label.
    pub(crate) fn replace_edge(&self, edge: Edge<T>) {
        self.edges.replace_edge(edge);
    }

    /// Replaces the node of the edge at the given index.
    pub(crate) fn replace_edge_at(&self, index: usize, edge: Edge<T>) {
        self.edges.replace_edge_at(index, edge);
    }

    /// Returns the index and node of the edge with the given label.
    pub(crate) fn get_edge(&self, label: u8) -> Option<(usize, Arc<Node<T>>)> {
        self.edges.get_edge(label)
    }

    /// Returns the node of the edge at the given index.
    pub(crate) fn get_edge_at(&self, index: usize) -> Option<Arc<Node<T>>> {
        self.edges.get_edge_at(index)
    }

    /// Returns the index and node of the lowest edge with label >= given label.
    pub(crate) fn get_lower_bound_edge(&self, label: u8) -> Option<(usize, Arc<Node<T>>)> {
        self.edges.get_lower_bound_edge(label)
    }

    /// Deletes the edge with the given label.
    pub(crate) fn delete_edge(&self, label: u8) {
        self.edges.delete_edge(label);
    }

    /// Returns the value associated with the given key if exists.
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

            if search_bytes.starts_with(node.prefix.read().as_str().as_bytes()) {
                search_bytes = &search_bytes[node.prefix.read().len()..];
            } else {
                break;
            }
        }
        None
    }

    /// Returns the key and value with the longest prefix match for the given key.
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

            if search_bytes.starts_with(node.prefix.read().as_str().as_bytes()) {
                search_bytes = &search_bytes[node.prefix.read().len()..];
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

    /// Returns the key and value with the minimum key in the subtree.
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

    /// Returns the key and value with the maximum key in the subtree.
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

    /// Returns true if there are no edges.
    pub fn empty_edge(&self) -> bool {
        self.edges.is_empty()
    }

    /// Returns the number of edges.
    pub fn edge_len(&self) -> usize {
        self.edges.len()
    }

    /// Returns the first edge's node if exists.
    pub fn first_edge(&self) -> Option<Arc<Node<T>>> {
        self.edges.first()
    }

    /// Returns the last edge's node if exists.
    pub fn last_edge(&self) -> Option<Arc<Node<T>>> {
        self.edges.last()
    }

    /// Clears all edges.
    pub fn clear_edges(&self) {
        self.edges.clear();
    }

    /// Resets all edges and clears capacity.
    pub fn reset_edges(&self) {
        self.edges.reset();
    }

    /// Removes and returns the last edge.
    pub fn pop_edge(&self) -> Option<Edge<T>> {
        self.edges.pop()
    }

    /// Collects all edges from self and inserts them into other.
    pub fn collect_into_edges(&self, edges: &Edges<T>) {
        self.edges.collect_into(edges)
    }

    /// Iterates over each edge and applies the given function.
    pub fn for_each_edge<F>(&self, f: F)
    where
        F: FnMut(&Edge<T>),
    {
        self.edges.for_each(f);
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
