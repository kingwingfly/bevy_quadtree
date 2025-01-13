use crate::tree::QuadTree;
use bevy::prelude::*;

pub(crate) fn update_quadtree<D, B, const N: usize>(
    mut tree: ResMut<QuadTree<D, N>>,
    q: Query<(&GlobalTransform, Entity), With<B>>,
) where
    QuadTree<D, N>: Resource,
    B: Component,
{
}
