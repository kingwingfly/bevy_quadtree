use bevy::{math::Rect, prelude::Entity};

#[derive(Debug)]
pub(crate) struct Node<const N: usize, const K: usize = 10> {
    inner: Vec<Entity>,
    boundary: Rect,
    children: [Option<Box<Node<N, K>>>; 4],
}

impl<const N: usize, const K: usize> From<Rect> for Node<N, K> {
    fn from(boundary: Rect) -> Self {
        Self {
            inner: Vec::new(),
            boundary,
            children: [None, None, None, None],
        }
    }
}
