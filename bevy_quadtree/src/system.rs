use crate::{collision::DynCollision, tree::QuadTree, UpdateCollision};
use bevy_ecs::prelude::*;
#[cfg(feature = "gizmos")]
use bevy_gizmos::prelude::*;
#[cfg(feature = "gizmos")]
use bevy_transform::components::GlobalTransform;

pub(crate) fn update_collision<S, C>(mut q: Query<(&mut S, &C), Changed<C>>)
where
    S: Component + UpdateCollision<C> + Clone,
    C: Component,
{
    for (s, c) in q.iter_mut() {
        <S as UpdateCollision<C>>::update()(s, c);
    }
}

pub(crate) fn update_quadtree<
    S,
    const N: usize,
    const D: usize,
    const W: usize,
    const H: usize,
    const K: usize,
    const ID: usize,
>(
    tree: Res<QuadTree<N, D, W, H, K, ID>>,
    q: Query<(Entity, &S), Changed<S>>,
    mut r: RemovedComponents<S>,
) where
    QuadTree<N, W, H, K, ID>: Resource,
    S: Component + DynCollision + Clone,
{
    q.par_iter().for_each(|(e, s)| {
        tree.insert(e, s.clone());
    });
    for e in r.read() {
        tree.remove(&e);
    }
}

#[cfg(feature = "gizmos")]
pub(crate) fn show_boundary<
    S,
    const N: usize,
    const D: usize,
    const W: usize,
    const H: usize,
    const K: usize,
    const ID: usize,
>(
    tree: Res<QuadTree<N, D, W, H, K, ID>>,
    q: Query<(Entity, &GlobalTransform), With<S>>,
    mut gizmos: Gizmos,
) where
    QuadTree<N, D, W, H, K, ID>: Resource,
    S: Component + DynCollision + Clone,
{
    use bevy_color::palettes::css::*;

    let mut x = vec![0];

    while let Some(id) = x.pop() {
        let node = &tree.tree[id];
        gizmos.rect_2d(
            node.inlet_boundary.center,
            node.inlet_boundary.size(),
            GREEN,
        );
        gizmos.rounded_rect_2d(
            node.outlet_boundary.center,
            node.outlet_boundary.size(),
            RED,
        );
        if !node.is_leaf() {
            for i in (id << 2) + 1..=(id << 2) + 4 {
                x.push(i);
            }
        }
    }
    for (e, t) in q.iter() {
        let pos = t.translation().truncate();
        if let Some(id) = tree.entities.read().get(&e) {
            let center = tree.tree[*id].inlet_boundary.center;
            gizmos.line_2d(pos, center, BLUE);
        }
    }
}
