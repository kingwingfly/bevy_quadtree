//! Shapes able to be used in Collision Detection.
//!
//! The `ID` of shapes is used to distinguish the same shape following different transforms, having nothing to do with the `ID` of the [`QuadTree`](crate::QuadTree).
//!
//! The `ID` of the `QuadTree` is used to distinguish different `QuadTree`s in the `World`, which is determined by the `ID` of [`QuadTreePlugin`](crate::QuadTreePlugin).

mod circle;
mod rect;
mod rotated_rect;

pub use circle::CollisionCircle;
pub use rect::CollisionRect;
pub use rotated_rect::CollisionRotatedRect;
