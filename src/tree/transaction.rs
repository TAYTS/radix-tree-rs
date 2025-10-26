use std::sync::{
    Arc,
    atomic::{self, AtomicU32},
};

use lru::LruCache;
use parking_lot::RwLock;

use crate::{NodeValue, node::Node};

const DEFAULT_MODIFIED_CACHE_SIZE: usize = 8192;

pub struct Txn<T>
where
    T: NodeValue,
{
    // root is the modified root node of the tree
    root: RwLock<Arc<Node<T>>>,

    // size tracks the size of tree as it is modified during the transaction
    size: AtomicU32,

    // writable is a cache of nodes created during the transaction.
    writable: Option<LruCache<Node<T>, ()>>,
}

impl<T: NodeValue> Txn<T> {
    fn internal_insert(&self, _key: &str, _value: T) -> (Option<Arc<Node<T>>>, Option<T>) {
        todo!("Implement internal insert logic");
        return (None, None);
    }
}

impl<T: NodeValue> Txn<T> {
    /// Insert add/update a given key. If the key already exists, its value is updated and the old value is returned.
    pub fn insert(&self, _key: &str, _value: T) -> Option<T> {
        let (new_node, old_value) = self.internal_insert(_key, _value);

        if let Some(node) = new_node {
            let mut root_guard = self.root.write();
            *root_guard = node;
        }

        if old_value.is_some() {
            // TODO: revisit the memory ordering here
            self.size.fetch_add(1, atomic::Ordering::Relaxed);
        }
        old_value
    }
}
