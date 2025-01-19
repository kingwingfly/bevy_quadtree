//! Collision Detection

use bevy::prelude::{Component, GlobalTransform};
use paste::paste;

use crate::{shape::CollisionRotatedRect, CollisionCircle, CollisionRect};

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

/// Storing shape and position infomation, performs Collision Detection with the given `S`.
/// Also, as a component, used as a marker in ECS queries.
pub trait Collision<S> {
    /// Return collision detection result.
    fn detect(&self, obj: &S) -> Relation;
}

/// Object safe trait for storing value in QuadTree
pub trait DynCollision:
    Collision<CollisionRect>
    + Collision<CollisionRotatedRect>
    + Collision<CollisionCircle>
    + Send
    + Sync
{
}

impl<T> DynCollision for T where
    T: Collision<CollisionRect>
        + Collision<CollisionRotatedRect>
        + Collision<CollisionCircle>
        + Send
        + Sync
{
}

/// Marker trait for [`S: DynCollision + Component`](crate::collision::DynCollision) and tuple of `S`s
pub trait AsCollision {}

impl<T> AsCollision for T where
    T: DynCollision + UpdateCollision<GlobalTransform> + Component + Clone
{
}

macro_rules! impl_as_collision {
    ($($shape: ident),+) => {
        impl<$($shape),+> AsCollision for ($($shape),+,)
        where
            $($shape: DynCollision + UpdateCollision<GlobalTransform> + Component + Clone),+
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

/// Update the attributes of the shape during PreUpdate stage and before Collision Detection.
pub trait UpdateCollision<C>
where
    C: Component,
{
    /// Set the attributes of the shape. Used for updating the position of the shape
    /// in `PreUpdate` stage when `GlobalTransform` changed.
    fn update() -> impl FnOnce(&mut Self, &C);
}

/// Disassemble the boundary as [`CollisionRect`]s, [`CollisionRotatedRect`]s and [`CollisionCircle`]s as query boundary.
///
/// Pay attention to the default implementation of [`CollisionQuery::query`] when implementing your own.
/// However, disassemble the boundary as smaller possible shapes is recommended since it's easier.
pub trait CollisionQuery {
    /// Disassemble the shape as [`CollisionRect`], [`CollisionRotatedRect`] and [`CollisionCircle`] as query boundaries.
    /// `CollisionQuery::disassemble` is called only in `CollisionQuery::query`'s default implementation,
    /// so leave it `unreachable!()` if you have your own implementation of `CollisionQuery`.
    fn disassemble(
        &self,
    ) -> (
        Vec<&CollisionRect>,
        Vec<&CollisionRotatedRect>,
        Vec<&CollisionCircle>,
    );
    /// Detect the relation between the boundary and objects from the tree.
    /// The default `CollisionQuery` impletation:
    ///
    /// Relation::Contain if any of the sub-boundaries completely contains the object.
    ///
    /// Relation::Contained if all of the sub-boundaries are completely contained by the object.
    ///
    /// Relation::Overlap if any of the sub-boundaries overlaps the object
    /// or not all of the sub-boundaries are contained by the object.
    ///
    /// Relation::Disjoint otherwise.
    fn query(&self, obj: &dyn DynCollision) -> Relation {
        let (rects, rotated_rects, circles) = self.disassemble();
        let mut relation = Relation::Contain;
        for rect in rects {
            match obj.detect(rect) {
                Relation::Contain => return Relation::Contained,
                Relation::Contained if relation == Relation::Contain => {}
                Relation::Contained | Relation::Overlap => return Relation::Overlap,
                Relation::Disjoint => relation = Relation::Disjoint,
            }
        }
        for r_rect in rotated_rects {
            match obj.detect(r_rect) {
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

impl<T> CollisionQuery for [T]
where
    T: CollisionQuery,
{
    fn disassemble(
        &self,
    ) -> (
        Vec<&CollisionRect>,
        Vec<&CollisionRotatedRect>,
        Vec<&CollisionCircle>,
    ) {
        let mut rects = vec![];
        let mut rotated_rects = vec![];
        let mut circles = vec![];
        for t in self {
            let (r, rr, c) = t.disassemble();
            rects.extend(r);
            rotated_rects.extend(rr);
            circles.extend(c);
        }
        (rects, rotated_rects, circles)
    }
}

macro_rules! impl_disassemble {
    ($($i: literal),+) => {
        paste! {
            impl<$([<S $i>]),+> CollisionQuery for ($([<S $i>]),+,)
            where
                $([<S $i>]: CollisionQuery,)+
            {
                fn disassemble(
                    &self,
                ) -> (
                    Vec<&CollisionRect>,
                    Vec<&CollisionRotatedRect>,
                    Vec<&CollisionCircle>,
                ) {
                    let mut rects = vec![];
                    let mut rotated_rects = vec![];
                    let mut circles = vec![];
                    $(
                        let (r, rr, c) = self.$i.disassemble();
                        rects.extend(r);
                        rotated_rects.extend(rr);
                        circles.extend(c);
                    )+
                    (rects, rotated_rects, circles)
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
