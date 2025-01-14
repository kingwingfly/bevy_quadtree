use crate::node::Node;
use bevy::prelude::*;
use core::ops::{Deref, DerefMut};

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

impl<const N: usize, const W: usize, const H: usize, const K: usize> Deref
    for QuadTree<N, W, H, K>
{
    type Target = Node<N, K>;

    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

impl<const N: usize, const W: usize, const H: usize, const K: usize> DerefMut
    for QuadTree<N, W, H, K>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.root
    }
}
