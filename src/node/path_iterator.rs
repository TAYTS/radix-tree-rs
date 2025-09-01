use std::rc::Rc;

use crate::node::{LeafNode, Node};

pub struct PathIterator<T>
where
    T: Clone + Default,
{
    pub(crate) node: Option<Rc<Node<T>>>,
    pub(crate) path: &'static str,
}

impl<T: Clone + Default> Iterator for PathIterator<T> {
    type Item = (&'static str, T);

    fn next(&mut self) -> Option<Self::Item> {
        let mut leaf: Option<LeafNode<T>> = None;
        while leaf.is_none() && self.node.is_some() {
            if self.node.as_ref().unwrap().leaf.is_some() {
                leaf = self.node.as_ref().unwrap().leaf.clone();
            }
            self.iterate();
        }
        if leaf.is_some() {
            return Some((
                leaf.as_ref().unwrap().key,
                leaf.as_ref().unwrap().value.clone(),
            ));
        }
        None
    }
}

impl<T: Clone + Default> PathIterator<T> {
    fn iterate(&mut self) {
        if self.path.len() == 0 {
            self.node = None;
            return;
        }

        if let Some((_, edge)) = self
            .node
            .as_ref()
            .unwrap()
            .get_edge(self.path.as_bytes()[0])
        {
            if self.path.starts_with(edge.node.prefix) {
                self.path = &self.path[edge.node.prefix.len()..];
            } else {
                self.node = None;
            }
        }
    }
}
