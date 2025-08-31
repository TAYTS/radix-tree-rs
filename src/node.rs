#![allow(dead_code)]

use std::{collections::VecDeque, rc::Rc};

#[derive(Debug, Clone)]
pub struct Edge<T> {
    label: u8,
    node: Rc<Node<T>>,
}

pub type Edges<T> = Vec<Edge<T>>;

impl<T> PartialOrd for Edge<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.label.cmp(&other.label))
    }
}

impl<T> PartialEq for Edge<T> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

#[derive(Debug, Clone)]
pub struct Node<T> {
    // TODO: add on change callback or channel
    prefix: &'static str,
    leaf: Option<LeafNode<T>>,
    edges: Edges<T>,
}

#[derive(Debug, Clone)]
pub struct LeafNode<T> {
    value: T,
    key: &'static str,
}

impl<T: Clone> Node<T> {
    fn is_leaf(&self) -> bool {
        self.leaf.is_some()
    }

    fn add_edge(&mut self, edge: Edge<T>) {
        let insert_idx = self
            .edges
            .binary_search_by(|e| e.label.cmp(&edge.label))
            .unwrap_or_else(|idx| idx);
        self.edges.insert(insert_idx, edge);
    }

    fn replace_edge(&mut self, edge: Edge<T>) {
        let edge_idx = self
            .edges
            .binary_search_by(|e| e.label.cmp(&edge.label))
            .unwrap_or_else(|idx| idx);
        if edge_idx < self.edges.len() || self.edges[edge_idx].label == edge.label {
            self.edges[edge_idx] = edge;
        } else {
            panic!("replace missing edge");
        }
    }

    fn get_edge(&self, label: u8) -> Option<(usize, &Edge<T>)> {
        let edge_idx = self
            .edges
            .binary_search_by(|e| e.label.cmp(&label))
            .unwrap_or_else(|idx| idx);
        if edge_idx < self.edges.len() && self.edges[edge_idx].label == label {
            Some((edge_idx, &self.edges[edge_idx]))
        } else {
            None
        }
    }

    fn get_lower_bound_edge(&self, label: u8) -> Option<(usize, &Edge<T>)> {
        let edge_idx = self
            .edges
            .binary_search_by(|e| e.label.cmp(&label))
            .unwrap_or_else(|idx| idx);
        if edge_idx < self.edges.len() {
            Some((edge_idx, &self.edges[edge_idx]))
        } else {
            None
        }
    }

    fn delete_edge(&mut self, label: u8) {
        let edge_idx = self
            .edges
            .binary_search_by(|e| e.label.cmp(&label))
            .unwrap_or_else(|idx| idx);
        if edge_idx < self.edges.len() && self.edges[edge_idx].label == label {
            self.edges.remove(edge_idx);
        }
    }

    pub fn get(&self, label: &str) -> Option<T> {
        let mut search = label;
        loop {
            if search.is_empty() {
                if self.is_leaf() && self.leaf.is_some() {
                    return Some(self.leaf.as_ref().unwrap().value.clone());
                }
                break;
            }

            if self.get_edge(search.as_bytes()[0]).is_none() {
                break;
            }

            if search.starts_with(self.prefix) {
                search = &search[self.prefix.len()..];
            } else {
                break;
            }
        }
        None
    }

    pub fn longest_prefix(&self, key: &str) -> Option<(&'static str, T)> {
        let mut last: Option<&LeafNode<T>> = None;
        let mut search = key;
        loop {
            if self.is_leaf() {
                last = self.leaf.as_ref();
            }

            if search.is_empty() {
                break;
            }

            if self.get_edge(search.as_bytes()[0]).is_none() {
                break;
            }

            if search.starts_with(self.prefix) {
                search = &search[self.prefix.len()..];
            } else {
                break;
            }
        }
        if last.is_some() {
            return Some((last.unwrap().key, last.unwrap().value.clone()));
        }
        None
    }

    pub fn minimum(&self) -> Option<(&'static str, T)> {
        let mut current = self;
        loop {
            if current.is_leaf() {
                return Some((
                    current.leaf.as_ref().unwrap().key,
                    current.leaf.as_ref().unwrap().value.clone(),
                ));
            }
            if !current.edges.is_empty() {
                current = &current.edges.first().unwrap().node;
            } else {
                break;
            }
        }
        None
    }

    pub fn maximum(&self) -> Option<(&'static str, T)> {
        let mut current = self;
        loop {
            if !current.edges.is_empty() {
                current = &current.edges.last().unwrap().node;
                continue;
            }
            if current.is_leaf() {
                return Some((
                    current.leaf.as_ref().unwrap().key,
                    current.leaf.as_ref().unwrap().value.clone(),
                ));
            } else {
                break;
            }
        }
        None
    }

    pub fn into_node_iterator(self) -> NodeIterator<T> {
        NodeIterator {
            node: Some(Rc::new(self)),
            stack: Vec::new(),
        }
    }
}

pub struct NodeIterator<T> {
    node: Option<Rc<Node<T>>>,
    stack: Vec<VecDeque<Edge<T>>>,
}

impl<T: Clone> Iterator for NodeIterator<T> {
    type Item = (&'static str, T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() && self.node.is_some() {
            self.stack.push(VecDeque::from([Edge {
                label: 0,
                node: self.node.as_ref().unwrap().clone(),
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
