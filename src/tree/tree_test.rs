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

    #[test]
    fn test_transaction() {
        let mut tree = Tree::<bool>::new();

        {
            // insert new key into transaction should not affect original tree
            let mut txn = tree.start_transaction();
            let insert_keys = vec!["001", "002", "010", "100"];
            for key in insert_keys.iter() {
                txn.insert(key, true);
            }

            assert_eq!(txn.len(), 4, "txn size should be 4 after insertions");
            assert_eq!(tree.len(), 0, "original tree size should still be 0");

            for key in insert_keys.iter() {
                let result = txn.get(key);
                assert_eq!(result, Some(true), "key '{key}' should exist in txn");
            }

            for key in insert_keys.iter() {
                let result = tree.get(key);
                assert_eq!(result, None, "key '{key}' should not exist in tree");
            }

            tree = txn.commit();
            assert_eq!(tree.len(), 4, "tree size should be 4 after commit");

            for key in insert_keys.iter() {
                let result = tree.get(key);
                assert_eq!(
                    result,
                    Some(true),
                    "key '{key}' should exist in tree after commit"
                );
            }
        }

        {
            // delete key in transaction should not affect original tree
            let mut txn = tree.start_transaction();
            let check_keys = vec!["002", "010", "100"];

            let result = txn.delete("001");
            assert_eq!(result, Some(true), "deleted value should be Some(true)");

            let result = txn.get("001");
            assert_eq!(result, None, "key '001' should be deleted in txn");

            let result = tree.get("001");
            assert_eq!(result, Some(true), "key '001' should still exist in tree");

            for key in check_keys.iter() {
                let result = txn.get(key);
                assert_eq!(result, Some(true), "key '{key}' should exist in txn");
            }

            for key in check_keys.iter() {
                let result = tree.get(key);
                assert_eq!(result, Some(true), "key '{key}' should still exist in tree");
            }

            assert_eq!(txn.len(), 3);
            assert_eq!(tree.len(), 4);

            tree = txn.commit();
            assert_eq!(tree.len(), 3);
            for key in check_keys.iter() {
                let result = tree.get(key);
                assert_eq!(result, Some(true), "key '{key}' should still exist in tree");
            }
        }

        {
            // delete prefix in transaction should not affect original tree
            let mut txn = tree.start_transaction();

            let deleted_keys = vec!["002", "010"];
            let check_key = "100";

            let result = txn.delete_prefix("0");
            assert!(result, "should delete keys with prefix '0'");

            assert_eq!(txn.len(), 1);
            assert_eq!(tree.len(), 3);

            for key in deleted_keys.iter() {
                let result = txn.get(key);
                assert_eq!(result, None, "key '{key}' should be deleted in txn");
            }

            for key in deleted_keys.iter() {
                let result = tree.get(key);
                assert_eq!(result, Some(true), "key '{key}' should still exist in tree");
            }

            assert_eq!(
                txn.get(check_key),
                Some(true),
                "key '{check_key}' should exist in txn"
            );
            assert_eq!(
                tree.get(check_key),
                Some(true),
                "key '{check_key}' should still exist in tree"
            );

            tree = txn.commit();
            assert_eq!(tree.len(), 1);

            for key in deleted_keys.iter() {
                let result = tree.get(key);
                assert_eq!(
                    result, None,
                    "key '{key}' should be deleted in tree after commit"
                );
            }
            assert_eq!(
                tree.get(check_key),
                Some(true),
                "key '{check_key}' should still exist in tree after commit"
            );
        }
    }

    #[test]
    fn test_tree_operations() {
        let tree = Tree::<bool>::new();

        let (tree, old_value) = tree.insert("001", true);
        assert_eq!(old_value, None, "old value should be None on first insert");
        assert_eq!(tree.len(), 1, "tree size should be 1 after insertion");
        assert_eq!(tree.get("001"), Some(true), "key '001' should exist");

        let (tree, old_value) = tree.insert("001", false);
        assert_eq!(
            old_value,
            Some(true),
            "old value should be Some(true) on update"
        );
        assert_eq!(tree.len(), 1, "tree size should still be 1 after update");
        assert_eq!(
            tree.get("001"),
            Some(false),
            "key '001' should be updated to false"
        );

        let (tree, old_value) = tree.delete("001");
        assert_eq!(
            old_value,
            Some(false),
            "old value should be Some(false) on delete"
        );
        assert_eq!(tree.len(), 0, "tree size should be 0 after deletion");
        assert_eq!(
            tree.get("001"),
            None,
            "key '001' should not exist after deletion"
        );

        let (tree, _) = tree.insert("001", true);
        let (tree, _) = tree.insert("002", true);
        let (tree, _) = tree.insert("003", true);
        let (tree, _) = tree.insert("010", true);
        let (tree, _) = tree.insert("100", true);

        let (tree, has_deleted) = tree.delete_prefix("00");
        assert!(has_deleted, "should delete keys with prefix '00'");
        assert_eq!(tree.len(), 2, "tree size should be 2 after prefix deletion");
    }
}
