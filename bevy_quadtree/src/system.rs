use crate::{tree::QuadTree, DynCollision, UpdateCollision};
use bevy::prelude::*;

pub(crate) fn update_quadtree<S, const N: usize, const W: usize, const H: usize, const K: usize>(
    mut tree: ResMut<QuadTree<N, W, H, K>>,
    mut q: Query<(&GlobalTransform, Entity, &mut S), Changed<GlobalTransform>>,
) where
    QuadTree<N, W, H, K>: Resource,
    S: Component + DynCollision + UpdateCollision + Clone,
{
    for (transform, e, mut s) in q.iter_mut() {
        <S as UpdateCollision>::update()(s.as_mut(), transform);
        tree.insert(e, s.clone());
    }
}
