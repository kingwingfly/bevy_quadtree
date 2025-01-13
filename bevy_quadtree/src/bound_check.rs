use bevy::{math::Rect, prelude::Component};

/// A shape that can be used to check if a point is inside it.
pub trait BoundCheck: Component {
    /// Returns bound check result if the point is inside the shape.
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

pub trait AsBoundCheck {}

impl<T> AsBoundCheck for T where T: BoundCheck {}

macro_rules! impl_as_bound {
    ($($bound: ident),+) => {
        impl<$($bound),+> AsBoundCheck for ($($bound),+,)
        where
            $($bound: BoundCheck),+
        {
        }
    };
}

impl_as_bound!(B1);
impl_as_bound!(B1, B2);
impl_as_bound!(B1, B2, B3);
impl_as_bound!(B1, B2, B3, B4);
impl_as_bound!(B1, B2, B3, B4, B5);
impl_as_bound!(B1, B2, B3, B4, B5, B6);
impl_as_bound!(B1, B2, B3, B4, B5, B6, B7);
impl_as_bound!(B1, B2, B3, B4, B5, B6, B7, B8);
impl_as_bound!(B1, B2, B3, B4, B5, B6, B7, B8, B9);
