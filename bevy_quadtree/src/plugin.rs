use crate::collision::{AsCollision, Collision};
use crate::system::update_quadtree;
use crate::tree::QuadTree;
use bevy::prelude::*;

/// A Bevy plugin for quadtree.
/// # Type Parameters
/// `S`: Shapes implemented [`Collision`], are used to perform Collision Detection with a rectangle,
/// store the shape info and as a marker component in ECS queries. (can be tuple)
///
/// `N`: The max number of objects each node.
///
/// `W`: The width of the root node boundary.
/// `H`: The height of the root node boundary.
/// The boundary's center is (0, 0).
///
/// `K`: For `LooseQuadTree`, K / 10 = loose_boundary / node_boundary. Set K to 10 by default and 20 is founded best.
/// # Example
/// ```no_run
/// # #[path = "test_utils.rs"]
/// # mod test_utils;
/// use bevy::prelude::*;
/// use bevy_quadtree::QuadTreePlugin;
/// use test_utils::{MyCircle, MyRect};
///
/// let _ = App::new()
///    .add_plugins(QuadTreePlugin::<(MyCircle, MyRect), 40, 100, 100, 20>::default())
///    .add_plugins(QuadTreePlugin::<MyCircle, 40, 100, 100>::default());
/// ```
#[derive(Debug)]
pub struct QuadTreePlugin<S, const N: usize, const W: usize, const H: usize, const K: usize = 10>
where
    S: AsCollision,
{
    _marker: std::marker::PhantomData<S>,
}

impl<S, const N: usize, const W: usize, const H: usize, const K: usize> Default
    for QuadTreePlugin<S, N, W, H, K>
where
    S: AsCollision,
{
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S, const N: usize, const W: usize, const H: usize, const K: usize> Plugin
    for QuadTreePlugin<S, N, W, H, K>
where
    S: Collision,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<QuadTree<N, W, H, K>>()
            .add_systems(Update, update_quadtree::<S, N, W, H, K>);
    }
}

macro_rules! impl_plugin {
    ($($shape: ident),+) => {
        impl<$($shape),+, const N: usize, const W: usize, const H: usize, const K: usize> Plugin
            for QuadTreePlugin<($($shape),+,), N, W, H, K>
        where
            $($shape: Collision),+,
            ($($shape),+,): AsCollision,
        {
            fn build(&self, app: &mut App) {
                app.init_resource::<QuadTree<N, W, H, K>>().add_systems(
                    Update,
                    (
                        $(update_quadtree::<$shape, N, W, H, K>),+
                    ),
                );
            }
        }
    };
}

impl_plugin!(S1);
impl_plugin!(S1, S2);
impl_plugin!(S1, S2, S3);
impl_plugin!(S1, S2, S3, S4);
impl_plugin!(S1, S2, S3, S4, S5);
impl_plugin!(S1, S2, S3, S4, S5, S6);
impl_plugin!(S1, S2, S3, S4, S5, S6, S7);
impl_plugin!(S1, S2, S3, S4, S5, S6, S7, S8);
impl_plugin!(S1, S2, S3, S4, S5, S6, S7, S8, S9);
