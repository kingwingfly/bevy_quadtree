//! QuadTree Plugin

use crate::collision::DynCollision;
use crate::system::{update_collision, update_quadtree};
use crate::tree::QuadTree;
use crate::UpdateCollision;
use bevy::ecs::schedule::SystemConfigs;
use bevy::prelude::*;

/// A Bevy plugin for quadtree.
/// # Type Parameters
/// `P`: `(S, C)` pair (or tuple of `(S, C)`s) where `S` (collision shape) can updated by `C` (component).
///
/// `S: Component + DynCollision + UpdateCollision<C> + Clone`,
/// such as [`CollisionCircle, CollisionRect, CollisionRotatedRect`](crate::shape).
/// are used to perform Collision Detection,
/// storing the shape and position info, also serving as a marker component in ECS queries.
/// Add the shapes which you wanna include into [`QuadTree`] and auto-upgrade.
/// (Do not need to include those only used in the [`QuadTree::query`](crate::QuadTree::query))
///
/// `C: Component`, such as `GlobalTransform`.
///
/// `N`: The max number of objects each node.
///
/// `W`: The width of the root node boundary.
/// `H`: The height of the root node boundary.
/// The boundary's center is (0, 0).
///
/// `K`: For `LooseQuadTree`, K / 10 = outlet_boundary / inlet_boundary. Set K to 10 by default and 20 is founded best.
/// K should >= 10. Only if the object move and is **no longer completely contained** by the outlet_boundary will it be inserted again.
///
/// # Example
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_quadtree::{CollisionCircle, CollisionRect, CollisionRotatedRect, QuadTreePlugin};
///
/// #[cfg(feature = "sprite")]
/// App::new()
///    .add_plugins(QuadTreePlugin::<(
///             (CollisionCircle, GlobalTransform),
///             (CollisionRotatedRect, GlobalTransform),
///             (CollisionRect, Sprite),
///         ),
///         40, 100, 100, 20>::default());
/// ```
#[derive(Debug)]
pub struct QuadTreePlugin<P, const N: usize, const W: usize, const H: usize, const K: usize = 10>
where
    P: TrackingPair,
{
    _marker: std::marker::PhantomData<P>,
}

impl<P, const N: usize, const W: usize, const H: usize, const K: usize> Default
    for QuadTreePlugin<P, N, W, H, K>
where
    P: TrackingPair,
{
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<P, const N: usize, const W: usize, const H: usize, const K: usize> Plugin
    for QuadTreePlugin<P, N, W, H, K>
where
    P: TrackingPair,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<QuadTree<N, W, H, K>>()
            .add_systems(PreUpdate, P::update_collision())
            .add_systems(Update, P::update_quadtree::<N, W, H, K>());
        #[cfg(feature = "gizmos")]
        {
            app.add_systems(PostUpdate, P::show_box::<N, W, H, K>());
        }
    }
}

/// `(S, C)` pair where `S` is collision shape and `C` is the component used to update `S`.
/// Also implemented for tuple of `(S, C)` pairs.
pub trait TrackingPair: Send + Sync + 'static {
    /// return the system to update collision
    fn update_collision() -> SystemConfigs;
    /// return the system to update quadtree
    fn update_quadtree<const N: usize, const W: usize, const H: usize, const K: usize>(
    ) -> SystemConfigs;
    /// return the system to show box
    #[cfg(feature = "gizmos")]
    fn show_box<const N: usize, const W: usize, const H: usize, const K: usize>() -> SystemConfigs;
}

impl<S, C> TrackingPair for (S, C)
where
    S: Component + DynCollision + UpdateCollision<C> + Clone,
    C: Component,
{
    fn update_collision() -> SystemConfigs {
        (update_collision::<S, C>,).ambiguous_with_all()
    }

    fn update_quadtree<const N: usize, const W: usize, const H: usize, const K: usize>(
    ) -> SystemConfigs {
        (update_quadtree::<S, N, W, H, K>).ambiguous_with_all()
    }
    #[cfg(feature = "gizmos")]
    fn show_box<const N: usize, const W: usize, const H: usize, const K: usize>() -> SystemConfigs {
        use crate::system::show_box;
        (show_box::<S, N, W, H, K>,).ambiguous_with_all()
    }
}

macro_rules! impl_tracking_pair {
    ($($i: literal),+) => {
        paste::paste! {
            impl<$([<S $i>]),+, $([<C $i>]),+> TrackingPair for ($(([<S $i>], [<C $i>])),+,)
            where
                $([<S $i>]: Component + DynCollision + UpdateCollision<[<C $i>]> + Clone),+,
                $([<C $i>]: Component),+
            {
                fn update_collision() -> SystemConfigs {
                    ($(update_collision::<[<S $i>], [<C $i>]>),+,).chain()
                }
                fn update_quadtree<const N: usize, const W: usize, const H: usize, const K: usize>(
                ) -> SystemConfigs {
                    ($(update_quadtree::<[<S $i>], N, W, H, K>),+,).chain()
                }
                #[cfg(feature = "gizmos")]
                fn show_box<const N: usize, const W: usize, const H: usize, const K: usize>(
                ) -> SystemConfigs {
                    use crate::system::show_box;
                    ($(show_box::<[<S $i>], N, W, H, K>),+,).chain()
                }
            }
        }
    };
}

impl_tracking_pair!(0);
impl_tracking_pair!(0, 1);
impl_tracking_pair!(0, 1, 2);
impl_tracking_pair!(0, 1, 2, 3);
impl_tracking_pair!(0, 1, 2, 3, 4);
impl_tracking_pair!(0, 1, 2, 3, 4, 5);
impl_tracking_pair!(0, 1, 2, 3, 4, 5, 6);
impl_tracking_pair!(0, 1, 2, 3, 4, 5, 6, 7);
impl_tracking_pair!(0, 1, 2, 3, 4, 5, 6, 7, 8);
