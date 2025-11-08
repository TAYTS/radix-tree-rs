#[cfg(test)]
mod tests {
    use crate::{node::Node, tree::Tree};

    #[test]
    fn test_new_tree() {
        let tree = Tree::<bool>::new();

        assert_eq!(
            tree,
            Tree::<bool> {
                root: Node::default().into(),
                size: 0,
            }
        );
    }
}
