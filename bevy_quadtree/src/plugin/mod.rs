//! QuadTree Plugin
//!

#[cfg(feature = "multi-quadtree")]
mod multi_plugin;

use crate::collision::{AsDynCollision, UpdateCollision};
use crate::system::{update_collision, update_quadtree};
use crate::tree::QuadTree;
use bevy_app::prelude::*;
use bevy_ecs::component::Mutable;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleConfigs;
use bevy_ecs::system::ScheduleSystem;
#[cfg(feature = "sprite")]
use bevy_sprite::Sprite;
use bevy_transform::prelude::*;
#[cfg(feature = "multi-quadtree")]
pub use multi_plugin::{
    AsQuadTreePluginConfig, MultiQuadTreePlugin, QTConfig, QuadTreePluginConfig,
};

/// A Bevy plugin for quadtree.
/// # Type Parameters
/// `P`: `(S, C)` pair (or tuple of `(S, C)`s) where `S` (collision shape) can updated by `C` (component).
///
/// `S: Component + AsDynCollision + UpdateCollision<C> + Clone`,
/// such as [`CollisionCircle, CollisionRect, CollisionRotatedRect`](crate::shape).
/// Being used to perform Collision Detection,
/// storing the shape and position info, also serving as a marker component in ECS queries.
/// (Do not need to include those only used in the [`QuadTree::query`](crate::QuadTree::query),
/// please include those need to be updated.)
///
/// `C: Component`, the components the shapes will be updated according to.
/// Only `GlobalTransform`, `Sprite`(need feature `sprite`) or tuples of them for now.
///
/// `N`: The max number of objects each node.
/// `D`: The max depth of the tree.
///
/// `W`: The width of the root node boundary.
/// `H`: The height of the root node boundary.
/// The boundary's center is (`X`, `Y`).
///
/// `K`: For `LooseQuadTree`, K / 10 = outlet_boundary / inlet_boundary. K is set to 20 by default.
/// K should >= 10. Only if the object move and is **no longer completely contained** by the outlet_boundary will it be inserted again.
///
/// `ID`: Set a lucky number as you like for the quadtree.
/// If you want multiple quadtrees, see [`MultiQuadTreePlugin`].
///
/// # Example
/// ```no_run
/// # #[cfg(feature = "sprite")]
/// # {
/// # use bevy_app::prelude::*;
/// # use bevy_transform::prelude::*;
/// # use bevy_sprite::Sprite;
/// use bevy_quadtree::{CollisionCircle, CollisionRect, CollisionRotatedRect, QuadTreePlugin};
///
/// App::new()
///    .add_plugins(QuadTreePlugin::<(
///             (CollisionCircle, GlobalTransform),
///             (CollisionRotatedRect, Sprite),
///             (CollisionRect, (GlobalTransform, Sprite)),
///         ),
///         40, 4, 100, 100, 0, 0, 20>::default());
/// # }
/// ```
///
/// # Panic
/// 1. duplicated shapes in `P`. (Debug check only)
///
/// e.g. `P = ((CollisionRect, GlobalTransform), (CollisionRect, Sprite))` will lead a debug_assertion failure.
/// Try `P = (CollisionRect, (GlobalTransform, Sprite))` or `P = ((CollisionRect<0>, GlobalTransform), (CollisionRect<1>, Sprite))` in concret context.
///
/// 2. invalid const parameters. (Debug check only)
///
/// N, D, W, H should > 0. K should >= 10.
#[derive(Debug)]
pub struct QuadTreePlugin<
    P,
    const N: usize,
    const D: usize,
    const W: usize,
    const H: usize,
    const X: usize = 0,
    const Y: usize = 0,
    const K: usize = 20,
    const ID: usize = 0,
> where
    P: TrackingPair,
{
    _marker: std::marker::PhantomData<P>,
}

impl<
    P,
    const D: usize,
    const N: usize,
    const W: usize,
    const H: usize,
    const X: usize,
    const Y: usize,
    const K: usize,
    const ID: usize,
> Default for QuadTreePlugin<P, N, D, W, H, X, Y, K, ID>
where
    P: TrackingPair,
{
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<
    P,
    const N: usize,
    const D: usize,
    const W: usize,
    const H: usize,
    const X: usize,
    const Y: usize,
    const K: usize,
    const ID: usize,
> Plugin for QuadTreePlugin<P, N, D, W, H, X, Y, K, ID>
where
    P: TrackingPair,
{
    fn build(&self, app: &mut App) {
        assert!(N > 0, "N should > 0");
        assert!(D > 0, "D should > 0");
        assert!(W > 0, "W should > 0");
        assert!(H > 0, "H should > 0");
        assert!(K >= 10, "K should >= 10");
        app.insert_resource(QuadTree::<ID>::new(N, D, W, H, X, Y, K))
            .add_systems(PreUpdate, P::update_collision())
            .add_systems(Update, P::update_quadtree::<ID>());
        #[cfg(feature = "gizmos")]
        app.add_systems(PostUpdate, P::show_boundary::<ID>());
    }
}

/// `(S, C)` pair where `S` is collision shape and `C` is the component used to update `S`.
/// Also implemented for tuple of `(S, C)` pairs.
pub trait TrackingPair: Send + Sync + 'static {
    /// return the system to update collision
    fn update_collision() -> ScheduleConfigs<ScheduleSystem>;
    /// return the system to update quadtree
    fn update_quadtree<const ID: usize>() -> ScheduleConfigs<ScheduleSystem>;
    /// return the system to show box
    #[cfg(feature = "gizmos")]
    fn show_boundary<const ID: usize>() -> ScheduleConfigs<ScheduleSystem>;
    /// return the shape id, to ensure no duplicate shape updating system added
    #[cfg(debug_assertions)]
    fn shape_id() -> std::any::TypeId;
}

macro_rules! impl_tracking_pair {
    ($c: ty) => {
        impl<S> TrackingPair for (S, $c)
        where
            S: Component<Mutability = Mutable> + AsDynCollision + UpdateCollision<$c> + Clone,
        {
            fn update_collision() -> ScheduleConfigs<ScheduleSystem> {
                update_collision::<S, $c>.ambiguous_with_all()
            }
            fn update_quadtree<const ID: usize>() -> ScheduleConfigs<ScheduleSystem> {
                update_quadtree::<S, ID>.ambiguous_with_all()
            }
            #[cfg(feature = "gizmos")]
            fn show_boundary<const ID: usize>() -> ScheduleConfigs<ScheduleSystem> {
                use crate::system::show_boundary;
                show_boundary::<S, ID>.ambiguous_with_all()
            }
            #[cfg(debug_assertions)]
            fn shape_id() -> std::any::TypeId {
                std::any::TypeId::of::<S>()
            }
        }
    };
}

impl_tracking_pair!(GlobalTransform);
#[cfg(feature = "sprite")]
impl_tracking_pair!(Sprite);

macro_rules! impl_tracking_pair_tuple {
    ($($c: ty),+) => {
        impl<S> TrackingPair for (S, ($($c),+,))
        where
            S: Component<Mutability = Mutable> + AsDynCollision + $(UpdateCollision<$c>+)+ Clone,
        {
            fn update_collision() -> ScheduleConfigs<ScheduleSystem> {
                // update_collision has Mut<S>, so chain them
                ($(update_collision::<S, $c>),+,).chain()
            }
            fn update_quadtree<const ID: usize>(
            ) -> ScheduleConfigs<ScheduleSystem> {
                update_quadtree::<S, ID>.ambiguous_with_all()
            }
            #[cfg(feature = "gizmos")]
            fn show_boundary<const ID: usize>(
            ) -> ScheduleConfigs<ScheduleSystem> {
                use crate::system::show_boundary;
                show_boundary::<S, ID>.ambiguous_with_all()
            }
            #[cfg(debug_assertions)]
            fn shape_id() -> std::any::TypeId {
                std::any::TypeId::of::<S>()
            }
        }
    };
}

impl_tracking_pair_tuple!(GlobalTransform);
#[cfg(feature = "sprite")]
impl_tracking_pair_tuple!(Sprite);
#[cfg(feature = "sprite")]
impl_tracking_pair_tuple!(Sprite, GlobalTransform);
#[cfg(feature = "sprite")]
impl_tracking_pair_tuple!(GlobalTransform, Sprite);

macro_rules! impl_tracking_pairs {
    ($($i: literal),+) => {
        paste::paste! {
            impl<$([<P $i>]),+> TrackingPair for ($([<P $i>]),+,)
            where
                $([<P $i>]: TrackingPair),+
            {
                fn update_collision() -> ScheduleConfigs<ScheduleSystem> {
                    // no duplicate shape in `P`s, so ambiguous_with_all
                    ($([<P $i>]::update_collision()),+,).ambiguous_with_all()
                }
                fn update_quadtree<const ID: usize>(
                ) -> ScheduleConfigs<ScheduleSystem> {
                    #[cfg(debug_assertions)]
                    {
                        let mut set = std::collections::HashMap::new();
                        $(
                            if let Some(dup) = set.insert([<P $i>]::shape_id(), std::any::type_name::<[<P $i>]>()) {
                                panic!("Duplicated quadtree updating system added:\n<{}>\n<{}>\nThey have the same collision shape, merge them into one or use `ID` type parameter of shape.", dup, std::any::type_name::<[<P $i>]>());
                            }
                        );+
                    }
                    ($([<P $i>]::update_quadtree::<ID>()),+,).ambiguous_with_all()
                }
                #[cfg(feature = "gizmos")]
                fn show_boundary<const ID: usize>(
                ) -> ScheduleConfigs<ScheduleSystem> {
                    #[cfg(debug_assertions)]
                    {
                        let mut set = std::collections::HashMap::new();
                        $(
                            if let Some(dup) = set.insert([<P $i>]::shape_id(), std::any::type_name::<[<P $i>]>()) {
                                panic!("Duplicated gizmos box updating system added:\n<{}>\n<{}>\nThey have the same collision shape, merge them into one or use `ID` type parameter of shape.", dup, std::any::type_name::<[<P $i>]>());
                            }
                        );+
                    }
                    ($([<P $i>]::show_boundary::<ID>()),+,).ambiguous_with_all()
                }
                #[cfg(debug_assertions)]
                fn shape_id() -> std::any::TypeId {
                    unreachable!("only (S, C) and ((S1, C), (S2, C), ...) are allowed")
                }
            }
        }
    };
}

impl_tracking_pairs!(0);
impl_tracking_pairs!(0, 1);
impl_tracking_pairs!(0, 1, 2);
impl_tracking_pairs!(0, 1, 2, 3);
impl_tracking_pairs!(0, 1, 2, 3, 4);
impl_tracking_pairs!(0, 1, 2, 3, 4, 5);
impl_tracking_pairs!(0, 1, 2, 3, 4, 5, 6);
impl_tracking_pairs!(0, 1, 2, 3, 4, 5, 6, 7);
impl_tracking_pairs!(0, 1, 2, 3, 4, 5, 6, 7, 8);

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use crate::{CollisionCircle, CollisionRect, CollisionRotatedRect};

    #[test]
    #[should_panic(expected = "Duplicated quadtree updating system added")]
    fn duplicate_shape1() {
        App::new().add_plugins(QuadTreePlugin::<
            (
                (CollisionCircle, GlobalTransform),
                (CollisionCircle, GlobalTransform),
            ),
            40,
            4,
            100,
            100,
            0,
            0,
            20,
        >::default());
    }

    #[cfg(feature = "sprite")]
    #[test]
    #[should_panic(expected = "Duplicated quadtree updating system added")]
    fn duplicate_shape2() {
        App::new().add_plugins(QuadTreePlugin::<
            ((CollisionRect, GlobalTransform), (CollisionRect, Sprite)),
            40,
            4,
            100,
            100,
            0,
            0,
            20,
        >::default());
    }

    #[cfg(feature = "sprite")]
    #[test]
    #[should_panic(expected = "Duplicated quadtree updating system added")]
    fn duplicate_shape3() {
        App::new().add_plugins(QuadTreePlugin::<
            (
                (CollisionCircle, GlobalTransform),
                (CollisionRect, Sprite),
                (CollisionRotatedRect, Sprite),
                (CollisionRotatedRect, (GlobalTransform, Sprite)),
            ),
            40,
            4,
            100,
            100,
            0,
            0,
            20,
        >::default());
    }

    #[cfg(all(feature = "sprite", feature = "gizmos"))]
    #[test]
    fn plugin_test() {
        App::new().add_plugins(QuadTreePlugin::<
            (
                (CollisionCircle, GlobalTransform),
                (CollisionRotatedRect, Sprite),
                (CollisionRect, (GlobalTransform, Sprite)),
            ),
            40,
            4,
            100,
            100,
            0,
            0,
            20,
        >::default());
    }

    #[test]
    fn plugin_test_with_id() {
        App::new().add_plugins(QuadTreePlugin::<
            (
                (CollisionCircle<0>, GlobalTransform),
                (CollisionCircle<1>, GlobalTransform),
            ),
            40,
            4,
            100,
            100,
            0,
            0,
            20,
        >::default());
    }

    #[cfg(feature = "sprite")]
    #[test]
    fn plugin_test_tuple() {
        App::new().add_plugins(QuadTreePlugin::<
            (
                (CollisionRect<0>, (GlobalTransform, Sprite)),
                (CollisionRect<1>, GlobalTransform),
            ),
            40,
            4,
            100,
            100,
            0,
            0,
            20,
        >::default());
    }
}
