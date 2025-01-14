use bevy::{
    math::Rect,
    prelude::{Component, GlobalTransform, Line2d},
};

/// The result of a bound check.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    Disjoint,
    ExternallyTangent,
    PartiallyOverlapping,
    InternallyTangent,
    CompletelyOverlapping,
}

/// Storing shape and position infomation, performs Collision Detection with the given `S`.
/// Also, as a component, used as a marker in ECS queries.
pub trait Collision<S> {
    /// Return collision detection result.
    fn detect(&self, obj: S) -> Relation;
}

/// Update the position of the shape during Update and before Collision Detection.
pub trait UpdateCollision {
    /// Set the position of the shape. Used for updating the position of the shape.
    fn update() -> impl FnOnce(&mut Self, &GlobalTransform);
}

/// Object safe trait for storing value in QuadTree
pub trait DynCollision: Collision<Rect> + Collision<Line2d> + Send + Sync {}

impl<T> DynCollision for T where T: Collision<Rect> + Collision<Line2d> + Send + Sync {}

/// Marker trait for `S: DynCollision + Component` and tuple of `S`s
pub trait AsCollision {}

impl<T> AsCollision for T where T: DynCollision + UpdateCollision + Component + Clone {}

macro_rules! impl_as_collision {
    ($($shape: ident),+) => {
        impl<$($shape),+> AsCollision for ($($shape),+,)
        where
            $($shape: DynCollision + UpdateCollision + Component + Clone),+
        {
        }
    };
}

impl_as_collision!(S1);
impl_as_collision!(S1, S2);
impl_as_collision!(S1, S2, S3);
impl_as_collision!(S1, S2, S3, S4);
impl_as_collision!(S1, S2, S3, S4, S5);
impl_as_collision!(S1, S2, S3, S4, S5, S6);
impl_as_collision!(S1, S2, S3, S4, S5, S6, S7);
impl_as_collision!(S1, S2, S3, S4, S5, S6, S7, S8);
impl_as_collision!(S1, S2, S3, S4, S5, S6, S7, S8, S9);
