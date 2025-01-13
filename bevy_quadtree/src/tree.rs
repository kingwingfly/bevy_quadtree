use crate::node::Node;
use bevy::prelude::*;

#[derive(Debug, Resource)]
pub(crate) struct QuadTree<D, const N: usize> {
    root: Node<D, N>,
}

impl<D, const N: usize> Default for QuadTree<D, N> {
    fn default() -> Self {
        Self {
            root: Node::default(),
        }
    }
}
