#![allow(dead_code)]

mod transaction;

use crate::{node::Node, utils::NodeValue};

pub struct Tree<T>
where
    T: NodeValue,
{
    root: Node<T>,
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
