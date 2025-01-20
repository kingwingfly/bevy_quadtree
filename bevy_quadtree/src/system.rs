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
    const W: usize,
    const H: usize,
    const K: usize,
    const ID: usize,
>(
    tree: Res<QuadTree<N, W, H, K, ID>>,
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
    const W: usize,
    const H: usize,
    const K: usize,
    const ID: usize,
>(
    tree: Res<QuadTree<N, W, H, K, ID>>,
    q: Query<(Entity, &GlobalTransform), With<S>>,
    mut gizmos: Gizmos,
) where
    QuadTree<N, W, H, K, ID>: Resource,
    S: Component + DynCollision + Clone,
{
    use crate::node::ArcNode;
    use bevy_color::palettes::css::*;

    fn draw<const N: usize, const K: usize>(gizmos: &mut Gizmos, node: &ArcNode<N, K>) {
        let node = node.read();
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
        if let Some(children) = &node.children {
            for child in children.iter() {
                draw(gizmos, child);
            }
        }
    }
    draw(&mut gizmos, &tree.root);
    for (e, t) in q.iter() {
        let pos = t.translation().truncate();
        if let Some(belong) = tree.entities.read().get(&e) {
            let center = belong.read().inlet_boundary.center;
            gizmos.line_2d(pos, center, BLUE);
        }
    }
}
