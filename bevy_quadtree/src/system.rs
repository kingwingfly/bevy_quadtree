use crate::{collision::DynCollision, tree::QuadTree, UpdateCollision};
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
    q: Query<(Entity, &S), Changed<S>>,
    mut r: RemovedComponents<S>,
) where
    QuadTree<N, W, H, K>: Resource,
    S: Component + DynCollision + Clone,
{
    q.par_iter().for_each(|(e, s)| {
        tree.insert(e, s.clone());
    });
    for e in r.read() {
        tree.remove(&e);
    }
}
