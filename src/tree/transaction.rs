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
    // TODO: maybe don't need RwLock here
    pub root: RwLock<Arc<Node<T>>>,

    // size tracks the size of tree as it is modified during the transaction
    pub size: AtomicU32,

    // writable is a cache of nodes created during the transaction.
    pub writable: Option<LruCache<Arc<Node<T>>, ()>>,
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

        if old_value.is_none() {
            // TODO: revisit the memory ordering here
            self.size.fetch_add(1, atomic::Ordering::Relaxed);
        }
        old_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_txn_insert() {
        let mut txn: Txn<u32> = Txn {
            root: RwLock::new(Arc::new(Node::default())),
            size: AtomicU32::new(0),
            writable: None,
        };

        {
            let result = txn.insert("001", 1);
            assert!(result.is_none());
            assert_eq!(txn.size.load(atomic::Ordering::Relaxed), 1);
            let root = txn.root.read();
            assert_eq!(root.prefix.read().as_str(), "");

            let edge = root.get_edge(b'0');
            assert!(edge.is_some());
            let (_, child_node) = edge.unwrap();
            assert_eq!(
                *child_node,
                Node {
                    prefix: RwLock::new("001".to_string()),
                    leaf: RwLock::new(Some(Arc::new(LeafNode {
                        key: "001".to_string(),
                        value: 1,
                    }))),
                    ..Default::default()
                }
            );
        }

        {
            // insert another key with common prefix "00" should split the node
            let result = txn.insert("002", 2);
            assert!(result.is_none());
            assert_eq!(txn.size.load(atomic::Ordering::Relaxed), 2);
            let root = txn.root.read();
            assert_eq!(root.prefix.read().as_str(), "");

            let edge = root.get_edge(b'0');
            assert!(edge.is_some());
            let (_, child_node) = edge.unwrap();
            assert_eq!(
                *child_node,
                Node {
                    prefix: RwLock::new("00".to_string()),
                    leaf: RwLock::new(None),
                    edges: vec![
                        Edge {
                            label: b'1',
                            node: Arc::new(Node {
                                prefix: RwLock::new("1".to_string()),
                                leaf: RwLock::new(Some(Arc::new(LeafNode {
                                    key: "001".to_string(),
                                    value: 1,
                                }))),
                                ..Default::default()
                            }),
                        },
                        Edge {
                            label: b'2',
                            node: Arc::new(Node {
                                prefix: RwLock::new("2".to_string()),
                                leaf: RwLock::new(Some(Arc::new(LeafNode {
                                    key: "002".to_string(),
                                    value: 2,
                                }))),
                                ..Default::default()
                            }),
                        },
                    ]
                    .into(),
                }
            );
        }

        {
            // insert another key with common prefix "00", should append to the edges
            let result = txn.insert("003", 3);
            assert!(result.is_none());
            assert_eq!(txn.size.load(atomic::Ordering::Relaxed), 3);
            let root = txn.root.read();
            assert_eq!(root.prefix.read().as_str(), "");

            let edge = root.get_edge(b'0');
            assert!(edge.is_some());
            let (_, child_node) = edge.unwrap();
            assert_eq!(
                *child_node,
                Node {
                    prefix: RwLock::new("00".to_string()),
                    leaf: RwLock::new(None),
                    edges: vec![
                        Edge {
                            label: b'1',
                            node: Arc::new(Node {
                                prefix: RwLock::new("1".to_string()),
                                leaf: RwLock::new(Some(Arc::new(LeafNode {
                                    key: "001".to_string(),
                                    value: 1,
                                }))),
                                ..Default::default()
                            }),
                        },
                        Edge {
                            label: b'2',
                            node: Arc::new(Node {
                                prefix: RwLock::new("2".to_string()),
                                leaf: RwLock::new(Some(Arc::new(LeafNode {
                                    key: "002".to_string(),
                                    value: 2,
                                }))),
                                ..Default::default()
                            }),
                        },
                        Edge {
                            label: b'3',
                            node: Arc::new(Node {
                                prefix: RwLock::new("3".to_string()),
                                leaf: RwLock::new(Some(Arc::new(LeafNode {
                                    key: "003".to_string(),
                                    value: 3,
                                }))),
                                ..Default::default()
                            }),
                        },
                    ]
                    .into(),
                }
            );
        }

        {
            // insert another key with shorter common prefix "0", should split the node again
            let result = txn.insert("010", 10);
            assert!(result.is_none());
            assert_eq!(txn.size.load(atomic::Ordering::Relaxed), 4);
            let root = txn.root.read();
            assert_eq!(root.prefix.read().as_str(), "");

            let edge = root.get_edge(b'0');
            assert!(edge.is_some());
            let (_, child_node) = edge.unwrap();
            assert_eq!(
                *child_node,
                Node {
                    prefix: RwLock::new("0".to_string()),
                    leaf: RwLock::new(None),
                    edges: vec![
                        Edge {
                            label: b'0',
                            node: Arc::new(Node {
                                prefix: RwLock::new("0".to_string()),
                                leaf: RwLock::new(None),
                                edges: vec![
                                    Edge {
                                        label: b'1',
                                        node: Arc::new(Node {
                                            prefix: RwLock::new("1".to_string()),
                                            leaf: RwLock::new(Some(Arc::new(LeafNode {
                                                key: "001".to_string(),
                                                value: 1,
                                            }))),
                                            ..Default::default()
                                        }),
                                    },
                                    Edge {
                                        label: b'2',
                                        node: Arc::new(Node {
                                            prefix: RwLock::new("2".to_string()),
                                            leaf: RwLock::new(Some(Arc::new(LeafNode {
                                                key: "002".to_string(),
                                                value: 2,
                                            }))),
                                            ..Default::default()
                                        }),
                                    },
                                    Edge {
                                        label: b'3',
                                        node: Arc::new(Node {
                                            prefix: RwLock::new("3".to_string()),
                                            leaf: RwLock::new(Some(Arc::new(LeafNode {
                                                key: "003".to_string(),
                                                value: 3,
                                            }))),
                                            ..Default::default()
                                        }),
                                    }
                                ]
                                .into(),
                            }),
                        },
                        Edge {
                            label: b'1',
                            node: Arc::new(Node {
                                prefix: RwLock::new("10".to_string()),
                                leaf: RwLock::new(Some(Arc::new(LeafNode {
                                    key: "010".to_string(),
                                    value: 10,
                                }))),
                                ..Default::default()
                            }),
                        },
                    ]
                    .into(),
                }
            );
        }

        {
            // insert another key with no common prefix, should add new edge to the root
            let result = txn.insert("100", 100);
            assert!(result.is_none());
            assert_eq!(txn.size.load(atomic::Ordering::Relaxed), 5);
            let root = txn.root.read();
            assert_eq!(root.prefix.read().as_str(), "");

            let edge_0 = root.get_edge(b'0');
            assert!(edge_0.is_some());
            let (_, child_node) = edge_0.unwrap();
            assert_eq!(
                *child_node,
                Node {
                    prefix: RwLock::new("0".to_string()),
                    leaf: RwLock::new(None),
                    edges: vec![
                        Edge {
                            label: b'0',
                            node: Arc::new(Node {
                                prefix: RwLock::new("0".to_string()),
                                leaf: RwLock::new(None),
                                edges: vec![
                                    Edge {
                                        label: b'1',
                                        node: Arc::new(Node {
                                            prefix: RwLock::new("1".to_string()),
                                            leaf: RwLock::new(Some(Arc::new(LeafNode {
                                                key: "001".to_string(),
                                                value: 1,
                                            }))),
                                            ..Default::default()
                                        }),
                                    },
                                    Edge {
                                        label: b'2',
                                        node: Arc::new(Node {
                                            prefix: RwLock::new("2".to_string()),
                                            leaf: RwLock::new(Some(Arc::new(LeafNode {
                                                key: "002".to_string(),
                                                value: 2,
                                            }))),
                                            ..Default::default()
                                        }),
                                    },
                                    Edge {
                                        label: b'3',
                                        node: Arc::new(Node {
                                            prefix: RwLock::new("3".to_string()),
                                            leaf: RwLock::new(Some(Arc::new(LeafNode {
                                                key: "003".to_string(),
                                                value: 3,
                                            }))),
                                            ..Default::default()
                                        }),
                                    }
                                ]
                                .into(),
                            }),
                        },
                        Edge {
                            label: b'1',
                            node: Arc::new(Node {
                                prefix: RwLock::new("10".to_string()),
                                leaf: RwLock::new(Some(Arc::new(LeafNode {
                                    key: "010".to_string(),
                                    value: 10,
                                }))),
                                ..Default::default()
                            }),
                        },
                    ]
                    .into(),
                }
            );

            let edge_1 = root.get_edge(b'1');
            assert!(edge_1.is_some());
            let (_, child_node) = edge_1.unwrap();
            assert_eq!(
                *child_node,
                Node {
                    prefix: RwLock::new("100".to_string()),
                    leaf: RwLock::new(Some(Arc::new(LeafNode {
                        key: "100".to_string(),
                        value: 100,
                    }))),
                    ..Default::default()
                }
            );
        }

        {
            // update existing child node value
            let result = txn.insert("002", 20);
            assert!(result.is_some());
            assert_eq!(result.unwrap(), 2);
            assert_eq!(txn.size.load(atomic::Ordering::Relaxed), 5);
            let root = txn.root.read();
            assert_eq!(root.prefix.read().as_str(), "");

            let edge_0 = root.get_edge(b'0');
            assert!(edge_0.is_some());
            let (_, child_node) = edge_0.unwrap();
            assert_eq!(
                *child_node,
                Node {
                    prefix: RwLock::new("0".to_string()),
                    leaf: RwLock::new(None),
                    edges: vec![
                        Edge {
                            label: b'0',
                            node: Arc::new(Node {
                                prefix: RwLock::new("0".to_string()),
                                leaf: RwLock::new(None),
                                edges: vec![
                                    Edge {
                                        label: b'1',
                                        node: Arc::new(Node {
                                            prefix: RwLock::new("1".to_string()),
                                            leaf: RwLock::new(Some(Arc::new(LeafNode {
                                                key: "001".to_string(),
                                                value: 1,
                                            }))),
                                            ..Default::default()
                                        }),
                                    },
                                    Edge {
                                        label: b'2',
                                        node: Arc::new(Node {
                                            prefix: RwLock::new("2".to_string()),
                                            leaf: RwLock::new(Some(Arc::new(LeafNode {
                                                key: "002".to_string(),
                                                value: 20,
                                            }))),
                                            ..Default::default()
                                        }),
                                    },
                                    Edge {
                                        label: b'3',
                                        node: Arc::new(Node {
                                            prefix: RwLock::new("3".to_string()),
                                            leaf: RwLock::new(Some(Arc::new(LeafNode {
                                                key: "003".to_string(),
                                                value: 3,
                                            }))),
                                            ..Default::default()
                                        }),
                                    }
                                ]
                                .into(),
                            }),
                        },
                        Edge {
                            label: b'1',
                            node: Arc::new(Node {
                                prefix: RwLock::new("10".to_string()),
                                leaf: RwLock::new(Some(Arc::new(LeafNode {
                                    key: "010".to_string(),
                                    value: 10,
                                }))),
                                ..Default::default()
                            }),
                        },
                    ]
                    .into(),
                }
            );
        }

        {
            // update top level node value
            let result = txn.insert("100", 200);
            assert!(result.is_some());
            assert_eq!(result.unwrap(), 100);
            assert_eq!(txn.size.load(atomic::Ordering::Relaxed), 5);
            let root = txn.root.read();
            assert_eq!(root.prefix.read().as_str(), "");

            let edge_1 = root.get_edge(b'1');
            assert!(edge_1.is_some());
            let (_, child_node) = edge_1.unwrap();
            assert_eq!(
                *child_node,
                Node {
                    prefix: RwLock::new("100".to_string()),
                    leaf: RwLock::new(Some(Arc::new(LeafNode {
                        key: "100".to_string(),
                        value: 200,
                    }))),
                    ..Default::default()
                }
            );
        }
    }
}
