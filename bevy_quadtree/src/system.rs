use crate::{tree::QuadTree, Collision};
use bevy::prelude::*;

pub(crate) fn update_quadtree<S, const N: usize, const W: usize, const H: usize, const K: usize>(
    mut tree: ResMut<QuadTree<N, W, H, K>>,
    q: Query<(&GlobalTransform, Entity, &S), Changed<GlobalTransform>>,
) where
    QuadTree<N, W, H, K>: Resource,
    S: Collision,
{
    for (transform, e, b) in q.iter() {}
}
