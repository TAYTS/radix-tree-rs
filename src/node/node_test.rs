#![cfg(test)]
mod tests {
    use crate::node::{Edge, LeafNode, Node};

    #[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
    struct TestValue {
        data: String,
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
            leaf: Some(leaf.into()),
            ..Default::default()
        };
        assert!(node.is_leaf(), "should return true for leaf node");

        let node: Node<TestValue> = Node::default();
        assert!(!node.is_leaf(), "should return false for non-leaf node");
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
                leaf: Some(
                    LeafNode {
                        value: TestValue { data: "new".into() },
                        key: "new_key".into(),
                    }
                    .into(),
                ),
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

    #[test]
    fn test_get() {
        let root: Node<TestValue> = Node::default();
        let edge_0 = Edge {
            label: b'0',
            node: Node::<TestValue> {
                prefix: "0".into(),
                ..Default::default()
            }
            .into(),
        };

        let edge_00 = Edge {
            label: b'0',
            node: Node::<TestValue> {
                prefix: "0".into(),
                ..Default::default()
            }
            .into(),
        };

        let edge_001 = Edge {
            label: b'1',
            node: Node {
                prefix: "1".into(),
                leaf: Some(
                    LeafNode {
                        value: TestValue {
                            data: "value_001".into(),
                        },
                        key: "001".into(),
                    }
                    .into(),
                ),
                ..Default::default()
            }
            .into(),
        };

        let edge_002 = Edge {
            label: b'2',
            node: Node {
                prefix: "2".into(),
                leaf: Some(
                    LeafNode {
                        value: TestValue {
                            data: "value_002".into(),
                        },
                        key: "002".into(),
                    }
                    .into(),
                ),
                ..Default::default()
            }
            .into(),
        };

        let edge_003 = Edge {
            label: b'3',
            node: Node {
                prefix: "3".into(),
                leaf: Some(
                    LeafNode {
                        value: TestValue {
                            data: "value_003".into(),
                        },
                        key: "003".into(),
                    }
                    .into(),
                ),
                ..Default::default()
            }
            .into(),
        };

        let edge_010 = Edge {
            label: b'1',
            node: Node {
                prefix: "10".into(),
                leaf: Some(
                    LeafNode {
                        value: TestValue {
                            data: "value_010".into(),
                        },
                        key: "010".into(),
                    }
                    .into(),
                ),
                ..Default::default()
            }
            .into(),
        };

        let edge_100 = Edge {
            label: b'1',
            node: Node {
                prefix: "100".into(),
                leaf: Some(
                    LeafNode {
                        value: TestValue {
                            data: "value_100".into(),
                        },
                        key: "100".into(),
                    }
                    .into(),
                ),
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
}
