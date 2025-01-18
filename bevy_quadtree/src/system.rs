use crate::{collision::DynCollision, tree::QuadTree, UpdateCollision};
use bevy::prelude::*;

#[derive(Debug, Component, Deref, DerefMut)]
pub(crate) struct UpdateCD<const CD: usize>(Timer);

impl<const CD: usize> Default for UpdateCD<CD> {
    fn default() -> Self {
        Self(Timer::from_seconds(CD as f32 / 1000., TimerMode::Once))
    }
}

pub(crate) fn update_collision<S, const CD: usize>(
    time: Res<Time>,
    mut q: Query<(&GlobalTransform, &mut S), Changed<GlobalTransform>>,
    mut cd: Local<UpdateCD<CD>>,
) where
    S: Component + UpdateCollision + Clone,
{
    if !cd.tick(time.delta()).just_finished() {
        return;
    }
    cd.reset();
    for (transform, mut s) in q.iter_mut() {
        <S as UpdateCollision>::update()(s.as_mut(), transform);
    }
}

pub(crate) fn update_quadtree<
    S,
    const N: usize,
    const W: usize,
    const H: usize,
    const K: usize,
    const CD: usize,
>(
    time: Res<Time>,
    tree: Res<QuadTree<N, W, H, K>>,
    q: Query<(Entity, &S), Changed<S>>,
    mut r: RemovedComponents<S>,
    mut cd: Local<UpdateCD<CD>>,
) where
    QuadTree<N, W, H, K>: Resource,
    S: Component + DynCollision + Clone,
{
    if !cd.tick(time.delta()).just_finished() {
        return;
    }
    cd.reset();
    q.par_iter().for_each(|(e, s)| {
        tree.insert(e, s.clone());
    });
    for e in r.read() {
        tree.remove(&e);
    }
}
