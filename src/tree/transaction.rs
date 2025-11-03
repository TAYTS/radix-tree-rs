use std::{
    num::NonZeroUsize,
    sync::{
        Arc,
        atomic::{self, AtomicU32},
    },
};

use lru::LruCache;
use parking_lot::RwLock;

use crate::{
    NodeValue,
    node::{Edge, LeafNode, Node},
    utils::longest_prefix,
};

const DEFAULT_MODIFIED_CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(8192).unwrap();

pub struct Txn<T>
where
    T: NodeValue,
{
    // root is the modified root node of the tree
    root: RwLock<Arc<Node<T>>>,

    // size tracks the size of tree as it is modified during the transaction
    size: AtomicU32,

    // writable is a cache of nodes created during the transaction.
    writable: Option<LruCache<Arc<Node<T>>, ()>>,
}

impl<T: NodeValue> Txn<T> {
    fn internal_insert(
        &mut self,
        node: Arc<Node<T>>,
        key: &str,
        search: &str,
        value: T,
    ) -> (Option<Arc<Node<T>>>, Option<T>) {
        // reach the end of the search key,
        // replace the leaf node with the new leaf node(new value)
        if search.is_empty() {
            let mut old_value: Option<T> = None;
            if node.is_leaf() {
                let leaf_value = node.leaf.read().as_ref().unwrap().value.clone();
                old_value = Some(leaf_value);
            }

            let new_node = self.get_writable_node(node);
            let leaf_node = LeafNode {
                key: key.to_string(),
                value: value,
            };
            new_node.replace_leaf(Some(leaf_node));
            return (Some(new_node), old_value);
        }

        let node_edge = node.get_edge(search.as_bytes()[0]);

        // no edge found, insert new edge
        if node_edge.is_none() {
            let new_leaf_node = LeafNode {
                key: key.to_string(),
                value: value,
            };
            let new_node = Node {
                prefix: RwLock::new(search.to_string()),
                leaf: RwLock::new(Some(Arc::new(new_leaf_node))),
                ..Default::default()
            };
            let new_edge = Edge {
                label: search.as_bytes()[0],
                node: Arc::new(new_node),
            };
            let writable_node = self.get_writable_node(node);
            writable_node.add_edge(new_edge);
            return (Some(writable_node), None);
        }

        let (edge_idx, child_node) = node_edge.unwrap();

        let common_prefix_len = longest_prefix(search, child_node.prefix.read().as_str());
        if common_prefix_len == child_node.prefix.read().len() {
            let new_search = &search[common_prefix_len..];
            let (new_child_node, old_value) =
                self.internal_insert(child_node, key, new_search, value);
            if let Some(new_child_node) = new_child_node {
                let writable_node = self.get_writable_node(node);
                let new_edge = Edge {
                    label: search.as_bytes()[0],
                    node: new_child_node,
                };
                // TODO: maybe we should use `replace_edge` here
                writable_node.replace_edge_at(edge_idx, new_edge);
                return (Some(writable_node), old_value);
            }
            return (None, old_value);
        }

        // split the node at the current longest common prefix
        // between the search key and the child node's prefix
        let split_node: Arc<Node<T>> = Arc::new(Node {
            prefix: RwLock::new(search[..common_prefix_len].to_string()),
            ..Default::default()
        });

        let writable_node = self.get_writable_node(node);
        writable_node.replace_edge(Edge {
            label: search.as_bytes()[0],
            node: split_node.clone(),
        });

        // move the existing child node under the split node
        let modified_child_node = self.get_writable_node(child_node);
        split_node.add_edge(Edge {
            label: modified_child_node.prefix.read().as_bytes()[common_prefix_len],
            node: modified_child_node.clone(),
        });
        {
            // update the prefix of the modified child node to remove the split node common prefix
            let mut prefix_write_guard = modified_child_node.prefix.write();
            prefix_write_guard.replace_range(..common_prefix_len, "");
        }

        // update search to remove the split node common prefix
        let search = &search[common_prefix_len..];

        // create new leaf node and associate with the split node
        let new_leaf_node = LeafNode {
            key: key.to_string(),
            value: value,
        };

        // reach the end of the search key,
        // associate the new leaf node with the split node
        if search.is_empty() {
            split_node.replace_leaf(Some(new_leaf_node));
            return (Some(writable_node), None);
        }

        let new_edge = Edge {
            label: search.as_bytes()[0],
            node: Arc::new(Node {
                prefix: RwLock::new(search.to_string()),
                leaf: RwLock::new(Some(Arc::new(new_leaf_node))),
                ..Default::default()
            }),
        };
        split_node.add_edge(new_edge);

        (Some(writable_node), None)
    }

    /// get_writable_node returns a new modifiable node for the current transaction if the given node has not been modified
    /// otherwise, it returns the existing modified node in the current transaction
    fn get_writable_node(&mut self, node: Arc<Node<T>>) -> Arc<Node<T>> {
        // TODO: maybe we should create new type on top of `Node<T>` to expose the mutable methods

        if self.writable.is_none() {
            let lru = LruCache::new(DEFAULT_MODIFIED_CACHE_SIZE);
            self.writable.replace(lru);
        }

        // current node has been modified in this transaction
        // return the existing modified node
        if self.writable.as_ref().unwrap().contains(&node) {
            return node;
        }

        // clone the node to prevent modifying the original node
        let clone_node = Arc::new((*node).clone());
        self.writable.as_mut().unwrap().put(clone_node.clone(), ());
        clone_node
    }
}

impl<T: NodeValue> Txn<T> {
    /// Insert add/update a given key. If the key already exists, its value is updated and the old value is returned.
    pub fn insert(&mut self, key: &str, value: T) -> Option<T> {
        let root = self.root.read().clone();
        let (new_node, old_value) = self.internal_insert(root, key, key, value);

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
