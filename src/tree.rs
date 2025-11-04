#![allow(dead_code)]

mod transaction;

use std::sync::Arc;

use parking_lot::lock_api::RwLock;

use crate::{node::Node, tree::transaction::Txn, utils::NodeValue};

pub struct Tree<T>
where
    T: NodeValue,
{
    root: Arc<Node<T>>,
    size: u32,
}

pub fn new<T>() -> Tree<T>
where
    T: NodeValue,
{
    Tree {
        root: Node::default().into(),
        size: 0,
    }
}

impl<T: NodeValue> Tree<T> {
    pub fn transaction(&self) -> Txn<T> {
        Txn {
            root: RwLock::new(self.root.clone()),
            size: self.size.into(),
            writable: None,
        }
    }
}
