use bevy::{math::Rect, prelude::Component};

/// As a shape, performs Collision Detection with a rectangle.
/// Also, as a component, used to store the shape and as a marker in ECS queries.
pub trait Collision: Component {
    /// Returns collision detection result.
    fn check(&self, rect: Rect) -> RelativePosition;
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

pub trait AsCollision {}

impl<T> AsCollision for T where T: Collision {}

macro_rules! impl_as_collision {
    ($($bound: ident),+) => {
        impl<$($bound),+> AsCollision for ($($bound),+,)
        where
            $($bound: Collision),+
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
