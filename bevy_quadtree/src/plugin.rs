//! QuadTree Plugin

use crate::collision::AsCollision;
use crate::collision::DynCollision;
use crate::system::{update_collision, update_quadtree};
use crate::tree::QuadTree;
use crate::UpdateCollision;
use bevy::prelude::*;

/// A Bevy plugin for quadtree.
/// # Type Parameters
/// `S`: Shapes implemented [`AsCollision`], are used to perform Collision Detection,
/// storing the shape and position info, also serving as a marker component in ECS queries (can be tuple).
/// Adding the shapes which you want to include into [`QuadTree`](crate::QuadTree) and auto-upgrade.
/// (Do not need to include those only used in the [`QuadTree::query`](crate::QuadTree::query))
///
/// `C`: A component serves as the source of tranform used to update `S` or shapes in `S`.
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
/// let mut app = App::new();
/// app
///    .add_plugins(QuadTreePlugin::<(CollisionCircle, CollisionRotatedRect, CollisionRect), GlobalTransform, 40, 100, 100, 20>::default());
/// #[cfg(feature = "sprite")]
/// app
///    .add_plugins(QuadTreePlugin::<CollisionRect, Sprite, 40, 100, 100>::default());
/// ```
#[derive(Debug)]
pub struct QuadTreePlugin<S, C, const N: usize, const W: usize, const H: usize, const K: usize = 10>
where
    S: AsCollision<C>,
    C: Component,
{
    _marker: std::marker::PhantomData<(S, C)>,
}

impl<S, C, const N: usize, const W: usize, const H: usize, const K: usize> Default
    for QuadTreePlugin<S, C, N, W, H, K>
where
    S: AsCollision<C>,
    C: Component,
{
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S, C, const N: usize, const W: usize, const H: usize, const K: usize> Plugin
    for QuadTreePlugin<S, C, N, W, H, K>
where
    S: DynCollision + UpdateCollision<C> + Component + Clone,
    C: Component,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<QuadTree<N, W, H, K>>()
            .add_systems(PreUpdate, update_collision::<S, C>)
            .add_systems(Update, update_quadtree::<S, N, W, H, K>);
        #[cfg(feature = "gizmos")]
        {
            use crate::system::show_box;
            app.add_systems(Update, show_box::<S, N, W, H, K>);
        }
    }
}

macro_rules! impl_plugin {
    ($($shape: ident),+) => {
        impl<$($shape),+, C, const N: usize, const W: usize, const H: usize, const K: usize> Plugin
            for QuadTreePlugin<($($shape),+,), C, N, W, H, K>
        where
            $($shape: DynCollision + UpdateCollision<C> + Component + Clone),+,
            C: Component,
            ($($shape),+,): AsCollision<C>,
        {
            fn build(&self, app: &mut App) {
                app.init_resource::<QuadTree<N, W, H, K>>()
                    .add_systems(
                        PreUpdate,
                        (
                            $(update_collision::<$shape, C>),+
                        ),
                    )
                    .add_systems(
                        Update,
                        (
                            $(update_quadtree::<$shape, N, W, H, K>),+
                        ),
                );
                #[cfg(feature = "gizmos")]
                {
                    use crate::system::show_box;
                    app.add_systems(
                        Update,
                        (
                            $(show_box::<$shape, N, W, H, K>),+
                        )
                    );
                }
            }
        }
    };
}

impl_plugin!(S0);
impl_plugin!(S0, S1);
impl_plugin!(S0, S1, S2);
impl_plugin!(S0, S1, S2, S3);
impl_plugin!(S0, S1, S2, S3, S4);
impl_plugin!(S0, S1, S2, S3, S4, S5);
impl_plugin!(S0, S1, S2, S3, S4, S5, S6);
impl_plugin!(S0, S1, S2, S3, S4, S5, S6, S7);
impl_plugin!(S0, S1, S2, S3, S4, S5, S6, S7, S8);
