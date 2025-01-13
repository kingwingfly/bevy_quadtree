use crate::bound_check::{AsBoundCheck, BoundCheck};
use crate::system::update_quadtree;
use crate::tree::QuadTree;
use bevy::prelude::*;
use std::any::Any;

/// A Bevy plugin for quadtree.
///
/// D: the data type to store in the quadtree. e.g. Entity
/// B: BoundCheck
///
/// # Example
/// ```no_run
/// # #[path = "test_utils.rs"]
/// # mod test_utils;
/// use bevy::prelude::*;
/// use bevy_quadtree::QuadTreePlugin;
/// use test_utils::{MyCircle, MyRect};
///
/// let _ = App::new().
///    add_plugins(QuadTreePlugin::<Entity, (MyCircle, MyRect), 4>::default());
/// ```
#[derive(Debug)]
pub struct QuadTreePlugin<D, B, const N: usize>
where
    D: Send + Sync + Any,
    B: AsBoundCheck,
{
    _marker: std::marker::PhantomData<(D, B)>,
}

impl<D, B, const N: usize> Default for QuadTreePlugin<D, B, N>
where
    D: Send + Sync + Any,
    B: AsBoundCheck,
{
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<D, B, const N: usize> Plugin for QuadTreePlugin<D, B, N>
where
    D: Send + Sync + Any,
    B: BoundCheck,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<QuadTree<D, N>>()
            .add_systems(Update, update_quadtree::<D, B, N>);
    }
}

macro_rules! impl_plugin {
    ($($bound: ident),+) => {
        impl<D, $($bound),+, const N: usize> Plugin for QuadTreePlugin<D, ($($bound),+,), N>
        where
            D: Send + Sync + Any,
            $($bound: BoundCheck),+,
            ($($bound),+,): AsBoundCheck,
        {
            fn build(&self, app: &mut App) {
                app.init_resource::<QuadTree<D, N>>().add_systems(
                    Update,
                    (
                        $(update_quadtree::<D, $bound, N>),+
                    ),
                );
            }
        }
    };
}

impl_plugin!(B1);
impl_plugin!(B1, B2);
impl_plugin!(B1, B2, B3);
impl_plugin!(B1, B2, B3, B4);
impl_plugin!(B1, B2, B3, B4, B5);
impl_plugin!(B1, B2, B3, B4, B5, B6);
impl_plugin!(B1, B2, B3, B4, B5, B6, B7);
impl_plugin!(B1, B2, B3, B4, B5, B6, B7, B8);
impl_plugin!(B1, B2, B3, B4, B5, B6, B7, B8, B9);
