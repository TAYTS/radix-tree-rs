use std::{collections::VecDeque, rc::Rc};

use crate::node::{Edge, Node};

pub(crate) struct NodeIterator<T>
where
    T: Default + Clone,
{
    pub(crate) node: Option<Rc<Node<T>>>,
    pub(crate) stack: Vec<VecDeque<Edge<T>>>,
}

impl<T: Clone + Default> Iterator for NodeIterator<T> {
    type Item = (&'static str, T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() && self.node.is_some() {
            self.stack.push(VecDeque::from([Edge {
                node: self.node.as_ref().unwrap().clone(),
                ..Default::default()
            }]));
        }

        while !self.stack.is_empty() {
            let last_edges = self.stack.last_mut().unwrap();
            let elem = last_edges.pop_front().unwrap().node;

            // note: remove from stack if the edges are empty
            if last_edges.is_empty() {
                self.stack.pop();
            }

            if !elem.edges.is_empty() {
                self.stack.push(VecDeque::from_iter(elem.edges.clone()));
            }

            if elem.leaf.is_some() {
                return Some((
                    elem.leaf.as_ref().unwrap().key,
                    elem.leaf.as_ref().unwrap().value.clone(),
                ));
            }
        }
        None
    }
}
