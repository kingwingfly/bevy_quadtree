use crate::node::Node;
use bevy::prelude::*;

#[derive(Debug, Resource)]
pub(crate) struct QuadTree<const N: usize, const W: usize, const H: usize, const K: usize = 10> {
    root: Node<N, K>,
}

impl<const N: usize, const W: usize, const H: usize, const K: usize> Default
    for QuadTree<N, W, H, K>
{
    fn default() -> Self {
        Self {
            root: Node::from(Rect::from_center_size(
                Vec2::ZERO,
                Vec2::new(W as f32, H as f32),
            )),
        }
    }
}
