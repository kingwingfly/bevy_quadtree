//! Collision Detection

use crate::{shape::CollisionRotatedRect, CollisionCircle, CollisionRect};
use bevy_ecs::prelude::*;

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

/// Erase the `ID` of the shape and store it as a trait object in the `QuadTree`.
#[allow(missing_docs)]
pub trait AsDynCollision {
    fn as_dyn_collision(&self) -> Box<dyn DynCollision>;
}

impl<const D: usize> AsDynCollision for CollisionRect<D> {
    fn as_dyn_collision(&self) -> Box<dyn DynCollision> {
        let this = CollisionRect::from(self);
        Box::new(this)
    }
}

impl<const D: usize> AsDynCollision for CollisionRotatedRect<D> {
    fn as_dyn_collision(&self) -> Box<dyn DynCollision> {
        let this = CollisionRotatedRect::from(self);
        Box::new(this)
    }
}

impl<const D: usize> AsDynCollision for CollisionCircle<D> {
    fn as_dyn_collision(&self) -> Box<dyn DynCollision> {
        let this = CollisionCircle::from(self);
        Box::new(this)
    }
}

/// Update the attributes of the shape during PreUpdate stage and before Collision Detection.
pub trait UpdateCollision<C>
where
    C: Component,
{
    /// Set the attributes of the shape. Used for updating the position of the shape
    /// in `PreUpdate` stage when `GlobalTransform` changed.
    fn update() -> impl FnOnce(Mut<Self>, &C);
}

/// Disassemble the boundary as [`CollisionRect`]s, [`CollisionRotatedRect`]s and [`CollisionCircle`]s as query boundary.
/// All `S: Disassemble` also impl `CollisionQuery`, which can be used as a boundary in [`QuadTree::query`](crate::QuadTree::query).
///
/// Relation::Contain if any of the sub-boundaries completely contains the object.
///
/// Relation::Contained if all of the sub-boundaries are completely contained by the object.
///
/// Relation::Overlap if any of the sub-boundaries overlaps the object
/// or not all of the sub-boundaries are contained by the object.
///
/// Relation::Disjoint otherwise.
pub trait Disassemble {
    /// Disassemble the shape as [`CollisionRect`], [`CollisionRotatedRect`] and [`CollisionCircle`] as query boundaries.
    fn disassemble(
        &self,
    ) -> (
        Vec<&CollisionRect>,
        Vec<&CollisionRotatedRect>,
        Vec<&CollisionCircle>,
    );
}

/// Used for [`QuadTree::query`](crate::QuadTree::query) as a boundary to detect the relation between the boundary and objects from the tree.
///
/// However, implementing user-defined query boundary with [`Disassemble`] trait is recommended, since it's easier.
///
/// For `S: Disassemble`, the default `CollisionQuery` impletation see [`here`](crate::Disassemble).
pub trait CollisionQuery {
    /// Detect the relation between the boundary and objects from the tree.
    fn query(&self, obj: &dyn DynCollision) -> Relation;
}

impl<S> CollisionQuery for S
where
    S: Disassemble,
{
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

impl<T> Disassemble for [T]
where
    T: Disassemble,
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

macro_rules! impl_disassemable {
    ($($i: literal),+) => {
        paste::paste! {
            impl<$([<S $i>]),+> Disassemble for ($([<S $i>]),+,)
            where
                $([<S $i>]: Disassemble,)+
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

impl_disassemable!(0);
impl_disassemable!(0, 1);
impl_disassemable!(0, 1, 2);
impl_disassemable!(0, 1, 2, 3);
impl_disassemable!(0, 1, 2, 3, 4);
impl_disassemable!(0, 1, 2, 3, 4, 5);
impl_disassemable!(0, 1, 2, 3, 4, 5, 6);
impl_disassemable!(0, 1, 2, 3, 4, 5, 6, 7);
impl_disassemable!(0, 1, 2, 3, 4, 5, 6, 7, 8);
