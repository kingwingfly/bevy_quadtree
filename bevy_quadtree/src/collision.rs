use bevy::{
    math::Rect,
    prelude::{Component, Line2d},
};

/// As a shape, performs Collision Detection with a rectangle.
/// Also, as a component, used to store the shape and as a marker in ECS queries.
pub trait Collision<F>: Component {
    /// Return collision detection result.
    fn detect(&self, obj: F) -> RelativePosition;
}

/// The result of a bound check.
#[allow(missing_docs)]
pub enum RelativePosition {
    Disjoint,
    ExternallyTangent,
    PartiallyOverlapping,
    InternallyTangent,
    CompletelyOverlapping,
}

/// Marker trait for `T: Collision<Rect> + Collision<Line2d>` and tuple of `T`s
pub trait AsCollision {}

impl<T> AsCollision for T where T: Collision<Rect> + Collision<Line2d> {}

macro_rules! impl_as_collision {
    ($($shape: ident),+) => {
        impl<$($shape),+> AsCollision for ($($shape),+,)
        where
            $($shape: Collision<Rect> + Collision<Line2d>),+
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
