use crate::{tree::QuadTree, DynCollision, UpdateCollision};
use bevy::prelude::*;

pub(crate) fn update_collision<S>(
    mut q: Query<(&GlobalTransform, &mut S), Changed<GlobalTransform>>,
) where
    S: Component + UpdateCollision + Clone,
{
    for (transform, mut s) in q.iter_mut() {
        <S as UpdateCollision>::update()(s.as_mut(), transform);
    }
}

pub(crate) fn update_quadtree<S, const N: usize, const W: usize, const H: usize, const K: usize>(
    tree: Res<QuadTree<N, W, H, K>>,
    mut q: Query<(Entity, &S), Changed<GlobalTransform>>,
) where
    QuadTree<N, W, H, K>: Resource,
    S: Component + DynCollision + Clone,
{
    for (e, s) in q.iter_mut() {
        tree.insert(e, s.clone());
    }
}
