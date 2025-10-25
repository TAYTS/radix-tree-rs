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
                node: Node::default(),
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
                node: Node::default(),
            };
            let edge_d = Edge {
                label: b'd',
                node: Node::default(),
            };
            node.add_edge(edge_b.clone());
            node.add_edge(edge_d.clone());

            // insert edge that should go in the middle
            let edge_c = Edge {
                label: b'c',
                node: Node::default(),
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
            node: Node::default(),
        };
        let edge_b = Edge {
            label: b'b',
            node: Node::default(),
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
            },
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
            node: Node::default(),
        };
        node.add_edge(edge_a);

        // attempt to replace non-existent edge 'b'
        let edge_b = Edge {
            label: b'b',
            node: Node::default(),
        };
        node.replace_edge(edge_b);
    }
}
