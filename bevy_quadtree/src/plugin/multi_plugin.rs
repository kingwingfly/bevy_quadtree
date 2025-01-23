use super::TrackingPair;
use crate::QuadTree;
use bevy_app::prelude::*;

/// A Bevy plugin for multiple quadtrees.
///
/// # Example
/// ```no_run
/// # #[cfg(feature = "sprite")]
/// # {
/// # use bevy_app::prelude::*;
/// # use bevy_transform::prelude::*;
/// # use bevy_sprite::Sprite;
/// # use bevy_quadtree::{CollisionCircle, CollisionRect, MultiQuadTreePlugin, QTConfig};
/// App::new()
///     .add_plugins(MultiQuadTreePlugin::<(
///         QTConfig::<(
///                 // CollisionRect with ID 0, follows GlobalTransform and Sprite
///                 (CollisionRect<0>, (GlobalTransform, Sprite)),
///                 // CollisionRect with ID 1, follows GlobalTransform only
///                 (CollisionRect<1>, GlobalTransform),
///                 // CollisionCircle with default ID 0, follows GlobalTransform
///                 (CollisionCircle, GlobalTransform),
///             ),
///             // at most 40 objects in each node, 4 levels, 100x100 boundary, center at (0, 0)
///             // 2.0 = outlet_boundary / inlet_boundary for LooseQuadTree,
///             // the ID of this quadtree is 0.
///             // query by `Res<QuadTree<0>>`
///             40, 4, 100, 100, 0, 0, 20, 0>,
///         QTConfig::<(
///                 // CollisionRect with ID 1, follows GlobalTransform and Sprite
///                 (CollisionRect<1>, (GlobalTransform, Sprite)),
///                 // CollisionCircle with ID 1, follows GlobalTransform only
///                 (CollisionCircle<1>, GlobalTransform),
///             ),
///             // The same attribute as the previous one, but the ID is 1.
///             // query by `Res<QuadTree<1>>`
///             40, 4, 100, 100, 0, 0, 20, 1>,
///         )>::default());
/// # }
/// ```
///
/// # Panic
/// 1. duplicated quadtree added. (Debug check only)
///
/// e.g. (QTConfig<..., 40, 4, 100, 100, 0, 0, 20, 0>, QTConfig<..., 40, 4, 100, 100, 0, 0, 20, 0>)
/// they both insert the same `QuadTree` into the world.
///
/// 2. duplicated shapes in the same quadtree. (Debug check only)
///
/// See [`QuadTreePlugin`](crate::QuadTreePlugin) for more information.
///
/// 3. invalid const parameters. (Debug check only)
///
/// N, D, W, H should > 0. K should >= 10.
#[derive(Debug)]
pub struct MultiQuadTreePlugin<C>
where
    C: AsQuadTreePluginConfig,
{
    _marker: std::marker::PhantomData<C>,
}

impl<C> Default for MultiQuadTreePlugin<C>
where
    C: AsQuadTreePluginConfig,
{
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C> Plugin for MultiQuadTreePlugin<C>
where
    C: AsQuadTreePluginConfig,
{
    fn build(&self, app: &mut App) {
        C::add_quadtree(app);
    }
}

/// Type alias for [`QuadTreePluginConfig`]
pub type QTConfig<
    P,
    const ID: usize,
    const N: usize,
    const D: usize,
    const W: usize,
    const H: usize,
    const X: usize = 0,
    const Y: usize = 0,
    const K: usize = 20,
> = QuadTreePluginConfig<P, ID, N, D, W, H, X, Y, K>;

/// QuadTreePluginConfig. See [`AsQuadTreePluginConfig`] for more information.
pub struct QuadTreePluginConfig<
    P,
    const ID: usize,
    const N: usize,
    const D: usize,
    const W: usize,
    const H: usize,
    const X: usize = 0,
    const Y: usize = 0,
    const K: usize = 20,
> where
    P: TrackingPair,
{
    _marker: std::marker::PhantomData<P>,
}

/// Marker trait for quadtree plugin config
pub trait AsQuadTreePluginConfig: Send + Sync + 'static {
    /// add quadtree resource, systems to app
    fn add_quadtree(app: &mut App);
    #[cfg(debug_assertions)]
    /// return the tree type id for duplicate checking
    fn tree_id() -> std::any::TypeId;
}

impl<
        P,
        const ID: usize,
        const N: usize,
        const D: usize,
        const W: usize,
        const H: usize,
        const X: usize,
        const Y: usize,
        const K: usize,
    > AsQuadTreePluginConfig for QuadTreePluginConfig<P, ID, N, D, W, H, X, Y, K>
where
    P: TrackingPair,
{
    fn add_quadtree(app: &mut App) {
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

    fn tree_id() -> std::any::TypeId {
        std::any::TypeId::of::<QuadTree<ID>>()
    }
}

macro_rules! impl_plugin_config {
    ($($i: literal),+) => {
        paste::paste! {
            impl<$([<C $i>]),+> AsQuadTreePluginConfig for ($([<C $i>]),+,)
            where
                $([<C $i>]: AsQuadTreePluginConfig),+
            {
                fn add_quadtree(app: &mut App) {
                    #[cfg(debug_assertions)]
                    {
                        let mut set = std::collections::HashMap::new();
                        $(
                            if let Some(dup) = set.insert([<C $i>]::tree_id(), std::any::type_name::<[<C $i>]>()) {
                                panic!("Duplicated quadtrees added:\n<{}>\n<{}>\nThey have the same ID, assign different IDs for them.", dup, std::any::type_name::<[<C $i>]>());
                            }
                        );+
                    }
                    $([<C $i>]::add_quadtree(app);)+
                }

                #[cfg(debug_assertions)]
                fn tree_id() -> std::any::TypeId {
                    unreachable!("only C and (C1, C2, ...) are allowed")
                }
            }
        }
    };
}

impl_plugin_config!(0);
impl_plugin_config!(0, 1);
impl_plugin_config!(0, 1, 2);
impl_plugin_config!(0, 1, 2, 3);
impl_plugin_config!(0, 1, 2, 3, 4);
impl_plugin_config!(0, 1, 2, 3, 4, 5);
impl_plugin_config!(0, 1, 2, 3, 4, 5, 6);
impl_plugin_config!(0, 1, 2, 3, 4, 5, 6, 7);
impl_plugin_config!(0, 1, 2, 3, 4, 5, 6, 7, 8);

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use crate::{CollisionCircle, CollisionRect, CollisionRotatedRect};
    use bevy_transform::prelude::*;

    #[test]
    #[should_panic(expected = "Duplicated quadtrees added")]
    fn duplicate_quadtree() {
        App::new().add_plugins(MultiQuadTreePlugin::<(
            QTConfig<
                (
                    (CollisionCircle<0>, GlobalTransform),
                    (CollisionCircle<1>, GlobalTransform),
                ),
                0,
                40,
                4,
                100,
                100,
                0,
                0,
                20,
            >,
            QTConfig<
                (
                    (CollisionCircle<2>, GlobalTransform),
                    (CollisionCircle<3>, GlobalTransform),
                ),
                0,
                40,
                4,
                100,
                100,
                0,
                0,
                20,
            >,
        )>::default());
    }

    #[test]
    fn multi_plugin_test1() {
        App::new().add_plugins(MultiQuadTreePlugin::<
            QTConfig<
                (
                    (CollisionCircle<0>, GlobalTransform),
                    (CollisionCircle<1>, GlobalTransform),
                ),
                0,
                40,
                4,
                100,
                100,
                0,
                0,
                20,
            >,
        >::default());
    }

    #[test]
    fn multi_plugin_test2() {
        App::new().add_plugins(MultiQuadTreePlugin::<(
            QTConfig<
                (
                    (CollisionCircle<0>, GlobalTransform),
                    (CollisionCircle<1>, GlobalTransform),
                ),
                0,
                40,
                4,
                100,
                100,
                0,
                0,
                20,
            >,
            QTConfig<
                (
                    (CollisionCircle<0>, GlobalTransform),
                    (CollisionCircle<1>, GlobalTransform),
                ),
                1,
                40,
                4,
                100,
                100,
                0,
                0,
                20,
            >,
        )>::default());
    }
}
