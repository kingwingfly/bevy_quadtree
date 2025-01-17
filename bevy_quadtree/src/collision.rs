use bevy::prelude::{Component, GlobalTransform};
use paste::paste;

use crate::{CollisionCircle, CollisionRect};

/// The result of a bound check.
/// # Example
/// `a.detect(b)`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    /// `a` disjoints `b`
    Disjoint,
    /// `a` overlaps but doesn't completely contain `b`, including ExternallyTangent, InternallyTangent
    Overlap,
    /// `a` completely contains `b`, `b` is in `a` and smaller
    Contain,
    /// `a` is completely contained by `b`, `a` is in `b` and smaller
    Contained,
}

/// Represents the relation between two shapes. Used in [`QuadTree::query()`] only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QRelation {
    /// `a` disjoints `b`
    Disjoint,
    /// `a` overlaps but doesn't completely contain `b`, including ExternallyTangent, InternallyTangent
    Overlap,
    /// `a` completely contains `b`, `b` is in `a` and smaller
    Contain,
    /// `a` is completely contained by `b`, `a` is in `b` and smaller
    Contained,
    /// `a` overlaps or completely contains `b`, `b` is in `a` and smaller if contains.
    ///
    /// This should be **only** used in [`QuadTree::query()`]
    OverlapOrContain,
}

/// Storing shape and position infomation, performs Collision Detection with the given `S`.
/// Also, as a component, used as a marker in ECS queries.
pub trait Collision<S> {
    /// Return collision detection result.
    fn detect(&self, obj: &S) -> Relation;
}

/// Object safe trait for storing value in QuadTree
pub trait DynCollision:
    Collision<CollisionRect> + Collision<CollisionCircle> + Send + Sync
{
}

impl<T> DynCollision for T where
    T: Collision<CollisionRect> + Collision<CollisionCircle> + Send + Sync
{
}

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

/// Update the position of the shape during Update and before Collision Detection.
pub trait UpdateCollision {
    /// Set the position of the shape. Used for updating the position of the shape
    /// in `PreUpdate` stage when `GlobalTransform` changed.
    fn update() -> impl FnOnce(&mut Self, &GlobalTransform);
}

/// Disassemble the boundary as `Rect`s and `CollisionCircle`s as query boundary
/// Pay attention to the default implementation of `Disassemble::detect()` when implementing your own.
pub trait Disassemble {
    /// Disassemble the shape as `Rect` and `CollisionCircle` as query boundary.
    fn disassemble(&self) -> (Vec<&CollisionRect>, Vec<&CollisionCircle>);
    /// Detect the relation between the boundary and the given object.
    /// The default `Disassemble::detect()` impletation:
    ///
    /// Relation::Contain if any of the boundary completely contains the object.
    ///
    /// Relation::Contained if all of the boundary is completely contained by the object.
    ///
    /// Relation::Overlap if any of the boundary overlaps the object
    /// or not all of the boundary is contained by the object.
    ///
    /// Relation::Disjoint otherwise.
    fn detect(&self, obj: &dyn DynCollision) -> Relation {
        let (rects, circles) = self.disassemble();
        let mut relation = Relation::Contain;
        for rect in rects {
            match obj.detect(rect) {
                Relation::Contain => return Relation::Contained,
                Relation::Contained if relation == Relation::Contain => {}
                Relation::Contained | Relation::Overlap => return Relation::Overlap,
                Relation::Disjoint => relation = Relation::Disjoint,
            }
        }
        for circle in circles {
            match obj.detect(circle) {
                Relation::Contain => return Relation::Contained,
                Relation::Contained if relation == Relation::Contain => {}
                Relation::Contained | Relation::Overlap => return Relation::Overlap,
                Relation::Disjoint => relation = Relation::Disjoint,
            }
        }
        relation
    }
}

impl<T> Disassemble for [T]
where
    T: Disassemble,
{
    fn disassemble(&self) -> (Vec<&CollisionRect>, Vec<&CollisionCircle>) {
        let mut rects = vec![];
        let mut circles = vec![];
        for t in self {
            let (r, c) = t.disassemble();
            rects.extend(r);
            circles.extend(c);
        }
        (rects, circles)
    }
}

macro_rules! impl_disassemble {
    ($($i: literal),+) => {
        paste! {
            impl<$([<S $i>]),+> Disassemble for ($([<S $i>]),+,)
            where
                $([<S $i>]: Disassemble,)+
            {
                fn disassemble(&self) -> (Vec<&CollisionRect>, Vec<&CollisionCircle>) {
                    let mut rects = vec![];
                    let mut circles = vec![];
                    $(
                        let (r, c) = self.$i.disassemble();
                        rects.extend(r);
                        circles.extend(c);
                    )+
                    (rects, circles)
                }
            }
        }
    };
}

impl_disassemble!(0);
impl_disassemble!(0, 1);
impl_disassemble!(0, 1, 2);
impl_disassemble!(0, 1, 2, 3);
impl_disassemble!(0, 1, 2, 3, 4);
impl_disassemble!(0, 1, 2, 3, 4, 5);
impl_disassemble!(0, 1, 2, 3, 4, 5, 6);
impl_disassemble!(0, 1, 2, 3, 4, 5, 6, 7);
impl_disassemble!(0, 1, 2, 3, 4, 5, 6, 7, 8);
