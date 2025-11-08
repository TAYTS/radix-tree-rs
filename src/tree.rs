#![allow(dead_code)]

mod transaction;
#[cfg(test)]
mod tree_test;

use std::sync::Arc;

use parking_lot::lock_api::RwLock;

use crate::{node::Node, tree::transaction::Txn, utils::NodeValue};

/// Immutable radix tree with prefix based lookup.
#[derive(Debug, PartialEq, Eq)]
pub struct Tree<T>
where
    T: NodeValue,
{
    root: Arc<Node<T>>,
    size: u32,
}

impl<T: NodeValue> Tree<T> {
    /// Create a new empty tree.
    pub fn new() -> Self {
        Tree {
            root: Node::default().into(),
            size: 0,
        }
    }

    /// Get the number of node in the tree.
    pub fn len(&self) -> u32 {
        self.size
    }

    /// Get the root node of the tree.
    pub fn root(&self) -> Arc<Node<T>> {
        self.root.clone()
    }

    /// Get the value associated with the given key if exists.
    pub fn get(&self, key: &str) -> Option<T> {
        self.root.get(key)
    }

    /// Create a new transaction for the tree.
    pub fn start_transaction(&self) -> Txn<T> {
        Txn {
            root: RwLock::new(self.root.clone()),
            size: self.size.into(),
            writable: None,
        }
    }

    /// Insert a key-value pair into the tree, returning the new tree and the old value if exists.
    pub fn insert(&self, key: &str, value: T) -> (Tree<T>, Option<T>) {
        let mut txn = self.start_transaction();
        let old_value = txn.insert(key, value);
        let new_tree = txn.commit();
        (new_tree, old_value)
    }

    /// Delete a key from the tree, returning the new tree and the old value if exists.
    pub fn delete(&self, key: &str) -> (Tree<T>, Option<T>) {
        let mut txn = self.start_transaction();
        let old_value = txn.delete(key);
        let new_tree = txn.commit();
        (new_tree, old_value)
    }

    /// Delete all keys with the given prefix from the tree, returning the new tree and a boolean indicating if any keys were deleted.
    pub fn delete_prefix(&self, prefix: &str) -> (Tree<T>, bool) {
        let mut txn = self.start_transaction();
        let has_deleted = txn.delete_prefix(prefix);
        let new_tree = txn.commit();
        (new_tree, has_deleted)
    }
}
