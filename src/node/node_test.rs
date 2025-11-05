#![cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::lock_api::RwLock;

    use crate::node::{Edge, LeafNode, Node};

    #[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
    struct TestValue {
        data: String,
    }

    #[test]
    fn test_node_equality() {
        let base_node: Node<TestValue> = Node {
            prefix: RwLock::new("prefix".into()),
            leaf: RwLock::new(Some(Arc::new(LeafNode {
                value: TestValue {
                    data: "value".into(),
                },
                key: "key".into(),
            }))),
            edges: vec![Edge {
                label: b'a',
                node: Arc::new(Node {
                    prefix: RwLock::new("a".into()),
                    leaf: RwLock::new(Some(Arc::new(LeafNode {
                        value: TestValue {
                            data: "a_value".into(),
                        },
                        key: "a_key".into(),
                    }))),
                    edges: vec![Edge {
                        label: b'b',
                        node: Arc::new(Node {
                            prefix: RwLock::new("ab".into()),
                            ..Default::default()
                        }),
                    }]
                    .into(),
                }),
            }]
            .into(),
        };

        {
            let node_eq: Node<TestValue> = Node {
                prefix: RwLock::new("prefix".into()),
                leaf: RwLock::new(Some(Arc::new(LeafNode {
                    value: TestValue {
                        data: "value".into(),
                    },
                    key: "key".into(),
                }))),
                edges: vec![Edge {
                    label: b'a',
                    node: Arc::new(Node {
                        prefix: RwLock::new("a".into()),
                        leaf: RwLock::new(Some(Arc::new(LeafNode {
                            value: TestValue {
                                data: "a_value".into(),
                            },
                            key: "a_key".into(),
                        }))),
                        edges: vec![Edge {
                            label: b'b',
                            node: Arc::new(Node {
                                prefix: RwLock::new("ab".into()),
                                ..Default::default()
                            }),
                        }]
                        .into(),
                    }),
                }]
                .into(),
            };

            assert_eq!(
                base_node, node_eq,
                "Nodes with identical data should be equal"
            );
        }

        {
            let node_prefix_diff = base_node.clone();
            {
                let mut write_guard = node_prefix_diff.prefix.write();
                *write_guard = "diff".into();
            }

            assert_ne!(
                base_node, node_prefix_diff,
                "Nodes with different prefixes should not be equal"
            );
        }

        {
            let node_leaf_diff_key = base_node.clone();

            {
                let mut write_guard = node_leaf_diff_key.leaf.write();
                *write_guard = Some(Arc::new(LeafNode {
                    value: TestValue {
                        data: "value".into(),
                    },
                    key: "key1".into(),
                }));
            }

            assert_ne!(
                base_node, node_leaf_diff_key,
                "Nodes with different leaf keys should not be equal"
            );
        }

        {
            let node_leaf_diff_value = base_node.clone();
            {
                let mut write_guard = node_leaf_diff_value.leaf.write();
                *write_guard = Some(Arc::new(LeafNode {
                    value: TestValue {
                        data: "value1".into(),
                    },
                    key: "key".into(),
                }));
            }

            assert_ne!(
                base_node, node_leaf_diff_value,
                "Nodes with different leaf values should not be equal"
            );
        }

        {
            let node_missing_leaf = base_node.clone();
            {
                let mut write_guard = node_missing_leaf.leaf.write();
                *write_guard = None;
            }

            assert_ne!(
                base_node, node_missing_leaf,
                "Nodes with one missing leaf should not be equal"
            );
        }

        {
            let node_with_different_edge_node_prefix = base_node.clone();
            {
                let mut write_guard = node_with_different_edge_node_prefix.edges.0.write();
                write_guard[0] = Edge {
                    label: b'a',
                    node: Arc::new(Node {
                        prefix: RwLock::new("different".into()),
                        ..Default::default()
                    }),
                };
            }

            assert_ne!(
                base_node, node_with_different_edge_node_prefix,
                "Nodes with different edge node prefixes should not be equal"
            );
        }

        {
            let node_with_different_edge_label = base_node.clone();
            {
                let mut write_guard = node_with_different_edge_label.edges.0.write();
                write_guard[0] = Edge {
                    label: b'b',
                    node: write_guard[0].node.clone(),
                };
            }

            assert_ne!(
                base_node, node_with_different_edge_label,
                "Nodes with different edge labels should not be equal"
            );
        }

        {
            let node_with_missing_edge = base_node.clone();
            {
                let mut write_guard = node_with_missing_edge.edges.0.write();
                write_guard.clear();
            }

            assert_ne!(
                base_node, node_with_missing_edge,
                "Nodes with missing edges should not be equal"
            );
        }

        {
            let node_with_additional_edge = base_node.clone();
            {
                let mut write_guard = node_with_additional_edge.edges.0.write();
                write_guard.push(Edge {
                    label: b'c',
                    node: Arc::new(Node {
                        prefix: RwLock::new("c".into()),
                        ..Default::default()
                    }),
                });
            }
            assert_ne!(
                base_node, node_with_additional_edge,
                "Nodes with additional edges should not be equal"
            );
        }

        {
            let node_with_different_edge_node_prefix = base_node.clone();
            {
                let mut write_guard = node_with_different_edge_node_prefix.edges.0.write();
                let edge = &mut write_guard[0];
                *edge = Edge {
                    label: b'a',
                    node: Arc::new(Node {
                        prefix: RwLock::new("different".into()),
                        leaf: RwLock::new(Some(Arc::new(LeafNode {
                            value: TestValue {
                                data: "a_value".into(),
                            },
                            key: "a_key".into(),
                        }))),
                        edges: vec![Edge {
                            label: b'b',
                            node: Arc::new(Node {
                                prefix: RwLock::new("ab".into()),
                                ..Default::default()
                            }),
                        }]
                        .into(),
                    }),
                };
            }

            assert_ne!(
                base_node, node_with_different_edge_node_prefix,
                "Nodes with different edge node prefixes should not be equal"
            );
        }

        {
            let node_with_different_edge_node_leaf_key = base_node.clone();
            {
                let mut write_guard = node_with_different_edge_node_leaf_key.edges.0.write();
                let edge = &mut write_guard[0];
                *edge = Edge {
                    label: b'a',
                    node: Arc::new(Node {
                        prefix: RwLock::new("a".into()),
                        leaf: RwLock::new(Some(Arc::new(LeafNode {
                            value: TestValue {
                                data: "a_value".into(),
                            },
                            key: "different_key".into(),
                        }))),
                        edges: vec![Edge {
                            label: b'b',
                            node: Arc::new(Node {
                                prefix: RwLock::new("ab".into()),
                                ..Default::default()
                            }),
                        }]
                        .into(),
                    }),
                };
            }

            assert_ne!(
                base_node, node_with_different_edge_node_leaf_key,
                "Nodes with different edge node leaf keys should not be equal"
            );
        }

        {
            let node_with_different_edge_node_leaf_value = base_node.clone();
            {
                let mut write_guard = node_with_different_edge_node_leaf_value.edges.0.write();
                let edge = &mut write_guard[0];
                *edge = Edge {
                    label: b'a',
                    node: Arc::new(Node {
                        prefix: RwLock::new("a".into()),
                        leaf: RwLock::new(Some(Arc::new(LeafNode {
                            value: TestValue {
                                data: "different".into(),
                            },
                            key: "a_key".into(),
                        }))),
                        edges: vec![Edge {
                            label: b'b',
                            node: Arc::new(Node {
                                prefix: RwLock::new("ab".into()),
                                ..Default::default()
                            }),
                        }]
                        .into(),
                    }),
                };
            }

            assert_ne!(
                base_node, node_with_different_edge_node_leaf_value,
                "Nodes with different edge node leaf values should not be equal"
            );
        }

        {
            let node_with_missing_edge_node_leaf = base_node.clone();
            {
                let mut write_guard = node_with_missing_edge_node_leaf.edges.0.write();
                let edge = &mut write_guard[0];
                *edge = Edge {
                    label: b'a',
                    node: Arc::new(Node {
                        prefix: RwLock::new("a".into()),
                        leaf: RwLock::new(None),
                        edges: vec![Edge {
                            label: b'b',
                            node: Arc::new(Node {
                                prefix: RwLock::new("ab".into()),
                                ..Default::default()
                            }),
                        }]
                        .into(),
                    }),
                };
            }

            assert_ne!(
                base_node, node_with_missing_edge_node_leaf,
                "Nodes with missing edge node leaves should not be equal"
            );
        }

        {
            let node_with_additional_edge_node_edge = base_node.clone();
            {
                let mut write_guard = node_with_additional_edge_node_edge.edges.0.write();
                let edge = &mut write_guard[0];
                *edge = Edge {
                    label: b'a',
                    node: Arc::new(Node {
                        prefix: RwLock::new("a".into()),
                        leaf: RwLock::new(Some(Arc::new(LeafNode {
                            value: TestValue {
                                data: "a_value".into(),
                            },
                            key: "a_key".into(),
                        }))),
                        edges: vec![
                            Edge {
                                label: b'b',
                                node: Arc::new(Node {
                                    prefix: RwLock::new("ab".into()),
                                    ..Default::default()
                                }),
                            },
                            Edge {
                                label: b'c',
                                node: Arc::new(Node {
                                    prefix: RwLock::new("ac".into()),
                                    ..Default::default()
                                }),
                            },
                        ]
                        .into(),
                    }),
                };
            }

            assert_ne!(
                base_node, node_with_additional_edge_node_edge,
                "Nodes with additional edge node edges should not be equal"
            );
        }

        {
            let node_with_missing_edge_node_edge = base_node.clone();
            {
                let mut write_guard = node_with_missing_edge_node_edge.edges.0.write();
                let edge = &mut write_guard[0];
                *edge = Edge {
                    label: b'a',
                    node: Arc::new(Node {
                        prefix: RwLock::new("a".into()),
                        leaf: RwLock::new(Some(Arc::new(LeafNode {
                            value: TestValue {
                                data: "a_value".into(),
                            },
                            key: "a_key".into(),
                        }))),
                        edges: vec![].into(),
                    }),
                };
            }
        }
    }

    #[test]
    fn new() {
        {
            let leaf_node = LeafNode {
                value: TestValue {
                    data: "test".into(),
                },
                key: "key".into(),
            };

            let node = Node::new("prefix", Some(leaf_node.clone()));
            assert_eq!(node.prefix.read().as_str(), "prefix");
            assert!(node.leaf.read().is_some());
            let stored_leaf = node.leaf.read();
            let stored_leaf = stored_leaf.as_ref().unwrap();
            assert_eq!(stored_leaf.key, leaf_node.key);
            assert_eq!(stored_leaf.value, leaf_node.value);
        }

        {
            let node_no_leaf: Node<TestValue> = Node::new("no_leaf", None);
            assert_eq!(node_no_leaf.prefix.read().as_str(), "no_leaf");
            assert!(node_no_leaf.leaf.read().is_none());
        }

        {
            let blank_node: Node<TestValue> = Node::new("", None);
            assert_eq!(blank_node.prefix.read().as_str(), "");
            assert!(blank_node.leaf.read().is_none());
        }
    }

    #[test]
    fn check_is_leaf() {
        let leaf = LeafNode {
            value: TestValue {
                data: "test".into(),
            },
            key: "key".into(),
        };

        let node = Node {
            leaf: RwLock::new(Some(Arc::new(leaf))),
            ..Default::default()
        };
        assert!(node.is_leaf(), "should return true for leaf node");

        let node: Node<TestValue> = Node::default();
        assert!(!node.is_leaf(), "should return false for non-leaf node");
    }

    #[test]
    fn test_replace_prefix() {
        let node: Node<TestValue> = Node::new("old_prefix", None);
        assert_eq!(node.prefix.read().as_str(), "old_prefix");

        node.replace_prefix("new_prefix");
        assert_eq!(node.prefix.read().as_str(), "new_prefix");

        node.replace_prefix("");
        assert_eq!(node.prefix.read().as_str(), "");
    }

    #[test]
    fn test_replace_leaf() {
        let node: Node<TestValue> = Node::default();

        // replace with a new leaf
        let new_leaf = LeafNode {
            value: TestValue {
                data: "new_data".into(),
            },
            key: "new_key".into(),
        };
        node.replace_leaf(Some(new_leaf.clone()));

        {
            let stored_leaf = node.leaf.read();
            assert!(
                stored_leaf.is_some(),
                "leaf should be present after replacement"
            );
            let stored_leaf = stored_leaf.as_ref().unwrap();
            assert_eq!(stored_leaf.key, new_leaf.key);
            assert_eq!(stored_leaf.value, new_leaf.value);
        }

        // replace with None (remove leaf)
        node.replace_leaf(None);

        {
            let stored_leaf = node.leaf.read();
            assert!(stored_leaf.is_none(), "leaf should be None after removal");
        }
    }

    #[test]
    fn test_add_edge() {
        {
            // insert into Node with no edges
            let node: Node<TestValue> = Node::default();
            let edge: Edge<TestValue> = Edge {
                label: b'a',
                node: Node::default().into(),
            };

            node.add_edge(edge.clone());

            let edges = node.edges.0.read();
            assert_eq!(edges.len(), 1);
            assert_eq!(edges[0], edge);
        }

        {
            // insert into Node with existing edges
            let node: Node<TestValue> = Node::default();
            let edge_b = Edge {
                label: b'b',
                node: Node::default().into(),
            };
            let edge_d = Edge {
                label: b'd',
                node: Node::default().into(),
            };
            node.add_edge(edge_b.clone());
            node.add_edge(edge_d.clone());

            // insert edge that should go in the middle
            let edge_c = Edge {
                label: b'c',
                node: Node::default().into(),
            };
            node.add_edge(edge_c.clone());

            let edges = node.edges.0.read();
            let edges = edges.as_slice();
            assert_eq!(edges.len(), 3);
            assert_eq!(
                edges,
                [edge_b, edge_c, edge_d],
                "edges should be in sorted order"
            );
        }
    }

    #[test]
    fn test_replace_edge() {
        let node: Node<TestValue> = Node::default();
        let edge_a = Edge {
            label: b'a',
            node: Node::default().into(),
        };
        let edge_b = Edge {
            label: b'b',
            node: Node::default().into(),
        };
        node.add_edge(edge_a.clone());
        node.add_edge(edge_b.clone());

        // replace edge 'a'
        let new_edge_a = Edge {
            label: b'a',
            node: Node {
                leaf: RwLock::new(Some(Arc::new(LeafNode {
                    value: TestValue { data: "new".into() },
                    key: "new_key".into(),
                }))),
                ..Default::default()
            }
            .into(),
        };
        node.replace_edge(new_edge_a.clone());

        let edges = node.edges.0.read();
        let edges = edges.as_slice();
        assert_eq!(edges.len(), 2);
        assert_eq!(edges[0], new_edge_a, "edge 'a' should be replaced");
        assert_eq!(edges[1], edge_b, "edge 'b' should remain unchanged");
    }

    #[test]
    fn test_replace_edge_at() {
        let node: Node<TestValue> = Node::default();
        let edge_a = Edge {
            label: b'a',
            node: Node::default().into(),
        };
        let edge_b = Edge {
            label: b'b',
            node: Node::default().into(),
        };
        node.add_edge(edge_a.clone());
        node.add_edge(edge_b.clone());

        // replace edge at index 1 (edge 'b')
        let new_edge_b = Edge {
            label: b'b',
            node: Node {
                leaf: RwLock::new(Some(Arc::new(LeafNode {
                    value: TestValue {
                        data: "new_b".into(),
                    },
                    key: "new_key_b".into(),
                }))),
                ..Default::default()
            }
            .into(),
        };
        node.replace_edge_at(1, new_edge_b.clone());

        {
            let edges = node.edges.0.read();
            let edges = edges.as_slice();
            assert_eq!(edges.len(), 2);
            assert_eq!(edges[0], edge_a, "edge 'a' should remain unchanged");
            assert_eq!(edges[1], new_edge_b, "edge 'b' should be replaced");
        }
    }

    #[test]
    #[should_panic(expected = "replace edge at invalid index or label mismatch")]
    fn test_replace_edge_invalid_index() {
        let node: Node<TestValue> = Node::default();
        let edge_a = Edge {
            label: b'a',
            node: Node::default().into(),
        };
        let edge_b = Edge {
            label: b'b',
            node: Node::default().into(),
        };
        node.add_edge(edge_a);
        node.add_edge(edge_b);

        // attempt to replace edge at invalid index
        node.replace_edge_at(
            5,
            Edge {
                label: b'c',
                node: Node::default().into(),
            },
        );
    }

    #[test]
    #[should_panic(expected = "replace edge at invalid index or label mismatch")]
    fn test_replace_edge_with_invalid_label() {
        let node: Node<TestValue> = Node::default();
        let edge_a = Edge {
            label: b'a',
            node: Node::default().into(),
        };
        let edge_b = Edge {
            label: b'b',
            node: Node::default().into(),
        };
        node.add_edge(edge_a);
        node.add_edge(edge_b);

        // attempt to replace edge with invalid label
        node.replace_edge_at(
            1,
            Edge {
                label: b'c',
                node: Node::default().into(),
            },
        );
    }

    #[test]
    #[should_panic(expected = "replace missing edge")]
    fn test_replace_missing_edge() {
        let node: Node<TestValue> = Node::default();
        let edge_a = Edge {
            label: b'a',
            node: Node::default().into(),
        };
        node.add_edge(edge_a);

        // attempt to replace non-existent edge 'b'
        let edge_b = Edge {
            label: b'b',
            node: Node::default().into(),
        };
        node.replace_edge(edge_b);
    }

    #[test]
    fn test_get_edge() {
        let node: Node<TestValue> = Node::default();
        let edge_a = Edge {
            label: b'a',
            node: Node::default().into(),
        };
        let edge_b = Edge {
            label: b'b',
            node: Node::default().into(),
        };
        node.add_edge(edge_a.clone());
        node.add_edge(edge_b.clone());

        // get existing edge 'a'
        let result = node.get_edge(b'a');
        assert!(result.is_some(), "should find edge 'a'");
        let (idx, found_node) = result.unwrap();
        assert_eq!(idx, 0, "edge 'a' should be at index 0");
        assert_eq!(
            *found_node, *edge_a.node,
            "found node for edge 'a' should match"
        );

        // get existing edge 'b'
        let result = node.get_edge(b'b');
        assert!(result.is_some(), "should find edge 'b'");
        let (idx, found_node) = result.unwrap();
        assert_eq!(idx, 1, "edge 'b' should be at index 1");
        assert_eq!(
            *found_node, *edge_b.node,
            "found node for edge 'b' should match"
        );

        // get non-existent edge 'c'
        let result = node.get_edge(b'c');
        assert!(result.is_none(), "should not find edge 'c'");
    }

    #[test]
    fn test_get_edge_at() {
        let node: Node<TestValue> = Node::default();
        let edge_a = Edge {
            label: b'a',
            node: Node::default().into(),
        };
        let edge_b = Edge {
            label: b'b',
            node: Node::default().into(),
        };
        node.add_edge(edge_a.clone());
        node.add_edge(edge_b.clone());

        {
            // get edge at index 0
            let result = node.get_edge_at(0);
            assert!(result.is_some(), "should find edge at index 0");
            let found_node = result.unwrap();
            assert_eq!(
                *found_node, *edge_a.node,
                "found node at index 0 should match edge 'a'"
            );
        }

        {
            // get edge at index 1
            let result = node.get_edge_at(1);
            assert!(result.is_some(), "should find edge at index 1");
            let found_node = result.unwrap();
            assert_eq!(
                *found_node, *edge_b.node,
                "found node at index 1 should match edge 'b'"
            );
        }

        {
            // get edge at invalid index 2
            let result = node.get_edge_at(2);
            assert!(result.is_none(), "should not find edge at index 2");
        }
    }

    #[test]
    fn test_get_lower_bound_edge() {
        let node: Node<TestValue> = Node::default();
        let edge_a = Edge {
            label: b'a',
            node: Node::default().into(),
        };
        let edge_c = Edge {
            label: b'c',
            node: Node::default().into(),
        };
        node.add_edge(edge_a.clone());
        node.add_edge(edge_c.clone());

        // get lower bound edge for 'b' (should return edge 'c')
        let result = node.get_lower_bound_edge(b'b');
        assert!(result.is_some(), "should find lower bound edge for 'b'");
        let (idx, found_node) = result.unwrap();
        assert_eq!(idx, 1, "lower bound edge for 'b' should be at index 1");
        assert_eq!(
            *found_node, *edge_c.node,
            "found node for lower bound edge should match edge 'c'"
        );

        // get lower bound edge for 'c' (should return edge 'c')
        let result = node.get_lower_bound_edge(b'c');
        assert!(result.is_some(), "should find lower bound edge for 'c'");
        let (idx, found_node) = result.unwrap();
        assert_eq!(idx, 1, "lower bound edge for 'c' should be at index 1");
        assert_eq!(
            *found_node, *edge_c.node,
            "found node for lower bound edge should match edge 'c'"
        );

        // get lower bound edge for 'd' (should return None)
        let result = node.get_lower_bound_edge(b'd');
        assert!(result.is_none(), "should not find lower bound edge for 'd'");
    }

    #[test]
    fn test_delete_edge() {
        let node: Node<TestValue> = Node::default();
        let edge_a = Edge {
            label: b'a',
            node: Node::default().into(),
        };
        let edge_b = Edge {
            label: b'b',
            node: Node::default().into(),
        };
        node.add_edge(edge_a.clone());
        node.add_edge(edge_b.clone());

        {
            // delete non-existent edge 'c' (should do nothing)
            node.delete_edge(b'c');
            let edges = node.edges.0.read();
            let edges = edges.as_slice();
            assert_eq!(edges.len(), 2, "both edges should remain");
            assert_eq!(edges[0], edge_a, "edge 'a' should remain");
            assert_eq!(edges[1], edge_b, "edge 'b' should remain");
        }

        {
            // delete edge 'a'
            node.delete_edge(b'a');
            let edges = node.edges.0.read();
            let edges = edges.as_slice();
            assert_eq!(edges.len(), 1);
            assert_eq!(edges[0], edge_b, "only edge 'b' should remain");
        }

        {
            // delete edge 'b'
            node.delete_edge(b'b');
            let edges = node.edges.0.read();
            let edges = edges.as_slice();
            assert_eq!(edges.len(), 0, "no edges should remain");
        }
    }

    fn get_test_tree() -> Node<TestValue> {
        let root: Node<TestValue> = Node::default();
        let edge_0 = Edge {
            label: b'0',
            node: Node::<TestValue> {
                prefix: RwLock::new("0".into()),
                ..Default::default()
            }
            .into(),
        };

        let edge_00 = Edge {
            label: b'0',
            node: Node::<TestValue> {
                prefix: RwLock::new("0".into()),
                ..Default::default()
            }
            .into(),
        };

        let edge_001 = Edge {
            label: b'1',
            node: Node {
                prefix: RwLock::new("1".into()),
                leaf: RwLock::new(Some(Arc::new(LeafNode {
                    value: TestValue {
                        data: "value_001".into(),
                    },
                    key: "001".into(),
                }))),
                ..Default::default()
            }
            .into(),
        };

        let edge_002 = Edge {
            label: b'2',
            node: Node {
                prefix: RwLock::new("2".into()),
                leaf: RwLock::new(Some(Arc::new(LeafNode {
                    value: TestValue {
                        data: "value_002".into(),
                    },
                    key: "002".into(),
                }))),
                ..Default::default()
            }
            .into(),
        };

        let edge_003 = Edge {
            label: b'3',
            node: Node {
                prefix: RwLock::new("3".into()),
                leaf: RwLock::new(Some(Arc::new(LeafNode {
                    value: TestValue {
                        data: "value_003".into(),
                    },
                    key: "003".into(),
                }))),
                ..Default::default()
            }
            .into(),
        };

        let edge_010 = Edge {
            label: b'1',
            node: Node {
                prefix: RwLock::new("10".into()),
                leaf: RwLock::new(Some(Arc::new(LeafNode {
                    value: TestValue {
                        data: "value_010".into(),
                    },
                    key: "010".into(),
                }))),
                ..Default::default()
            }
            .into(),
        };

        let edge_100 = Edge {
            label: b'1',
            node: Node {
                prefix: RwLock::new("100".into()),
                leaf: RwLock::new(Some(Arc::new(LeafNode {
                    value: TestValue {
                        data: "value_100".into(),
                    },
                    key: "100".into(),
                }))),
                ..Default::default()
            }
            .into(),
        };

        edge_00.node.add_edge(edge_001);
        edge_00.node.add_edge(edge_002);
        edge_00.node.add_edge(edge_003);
        edge_0.node.add_edge(edge_010);
        edge_0.node.add_edge(edge_00);
        root.add_edge(edge_0);
        root.add_edge(edge_100);

        root
    }

    #[test]
    fn test_get() {
        let root = get_test_tree();

        {
            let result = root.get("001");
            assert_eq!(
                result,
                Some(TestValue {
                    data: "value_001".into()
                })
            );
        }

        {
            let result = root.get("100");
            assert_eq!(
                result,
                Some(TestValue {
                    data: "value_100".into()
                })
            );
        }

        {
            let result = root.get("002");
            assert_eq!(
                result,
                Some(TestValue {
                    data: "value_002".into()
                })
            );
        }

        {
            let result = root.get("003");
            assert_eq!(
                result,
                Some(TestValue {
                    data: "value_003".into()
                })
            );
        }

        {
            let result = root.get("010");
            assert_eq!(
                result,
                Some(TestValue {
                    data: "value_010".into()
                })
            );
        }

        {
            let result = root.get("01");
            assert_eq!(result, None);
        }

        {
            let result = root.get("00");
            assert_eq!(result, None);
        }

        {
            let result = root.get("0");
            assert_eq!(result, None);
        }
    }

    #[test]
    fn test_longest_prefix() {
        let root = get_test_tree();

        {
            let result = root.longest_prefix("00123");
            assert_eq!(
                result,
                Some((
                    "001".into(),
                    TestValue {
                        data: "value_001".into()
                    }
                ))
            );
        }

        {
            let result = root.longest_prefix("003");
            assert_eq!(
                result,
                Some((
                    "003".into(),
                    TestValue {
                        data: "value_003".into()
                    }
                ))
            );
        }

        {
            let result = root.longest_prefix("10099");
            assert_eq!(
                result,
                Some((
                    "100".into(),
                    TestValue {
                        data: "value_100".into()
                    }
                ))
            );
        }

        {
            let result = root.longest_prefix("002abc");
            assert_eq!(
                result,
                Some((
                    "002".into(),
                    TestValue {
                        data: "value_002".into()
                    }
                ))
            );
        }

        {
            let result = root.longest_prefix("010abc");
            assert_eq!(
                result,
                Some((
                    "010".into(),
                    TestValue {
                        data: "value_010".into()
                    }
                ))
            );
        }

        {
            let result = root.longest_prefix("011abc");
            assert_eq!(result, None);
        }

        {
            let result = root.longest_prefix("0");
            assert_eq!(result, None);
        }

        {
            let result = root.longest_prefix("2");
            assert_eq!(result, None);
        }
    }

    #[test]
    fn test_minimum() {
        let root = get_test_tree();

        let result = root.minimum();
        assert_eq!(
            result,
            Some((
                "001".into(),
                TestValue {
                    data: "value_001".into()
                }
            ))
        );
    }

    #[test]
    fn test_maximum() {
        let root = get_test_tree();

        let result = root.maximum();
        assert_eq!(
            result,
            Some((
                "100".into(),
                TestValue {
                    data: "value_100".into()
                }
            ))
        );
    }

    #[test]
    fn test_is_empty() {
        {
            // non-empty node
            let root = get_test_tree();
            assert!(!root.empty_edge(), "node with edges should not be empty");
        }

        {
            // empty node
            let root = Node::<TestValue>::default();
            assert!(root.empty_edge(), "node with no edges should be empty");
        }
    }

    #[test]
    fn test_first_edge() {
        {
            // get first edge correctly
            let root = get_test_tree();

            let first_edge_node = root.first_edge();
            assert!(first_edge_node.is_some());
            let first_edge_node = first_edge_node.unwrap();
            assert_eq!(first_edge_node.prefix.read().as_str(), "0");
        }

        {
            // empty node returns None
            let root = Node::<TestValue>::default();
            let first_edge_node = root.first_edge();
            assert!(first_edge_node.is_none());
        }
    }

    #[test]
    fn test_last_edge() {
        {
            // get last edge correctly
            let root = get_test_tree();

            let last_edge_node = root.last_edge();
            assert!(last_edge_node.is_some());
            let last_edge_node = last_edge_node.unwrap();
            assert_eq!(last_edge_node.prefix.read().as_str(), "100");
        }

        {
            // empty node returns None
            let root = Node::<TestValue>::default();
            let last_edge_node = root.last_edge();
            assert!(last_edge_node.is_none());
        }
    }
}
