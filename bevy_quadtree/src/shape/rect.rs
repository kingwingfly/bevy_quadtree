use crate::{
    Collision, CollisionCircle, CollisionRotatedRect, Disassemble, DynCollision, Relation,
    UpdateCollision,
};
use bevy::prelude::*;
use std::ops::Deref;

/// Rectagle shape implemented [`AsCollision`](crate::AsCollision) trait to be used in the QuadTreePlugin
/// and as a Component in the ECS.
///
/// Also, implementes [`Disassemble`](crate::Disassemble) trait to be used in the [`QuadTree::query`](crate::QuadTree::query).
///
/// # Panic
/// Rotation is not supported for CollisionRect
#[derive(Debug, Component, Clone)]
pub struct CollisionRect {
    rect: Rect,
    init_size: Vec2,
}

impl Deref for CollisionRect {
    type Target = Rect;

    fn deref(&self) -> &Self::Target {
        &self.rect
    }
}

impl From<Rect> for CollisionRect {
    fn from(rect: Rect) -> Self {
        Self {
            rect,
            init_size: rect.size(),
        }
    }
}

impl Collision<CollisionRect> for CollisionRect {
    fn detect(&self, rect: &CollisionRect) -> Relation {
        if self.min.x < rect.min.x
            && self.min.y < rect.min.y
            && self.max.x > rect.max.x
            && self.max.y > rect.max.y
        {
            Relation::Contain
        } else if self.max.x < rect.min.x
            || self.min.x > rect.max.x
            || self.max.y < rect.min.y
            || self.min.y > rect.max.y
        {
            Relation::Disjoint
        } else if self.min.x > rect.min.x
            && self.min.y > rect.min.y
            && self.max.x < rect.max.x
            && self.max.y < rect.max.y
        {
            Relation::Contained
        } else {
            Relation::Overlap
        }
    }
}

impl Collision<CollisionRotatedRect> for CollisionRect {
    fn detect(&self, r_rect: &CollisionRotatedRect) -> Relation {
        todo!()
    }
}

impl Collision<CollisionCircle> for CollisionRect {
    fn detect(&self, circle: &CollisionCircle) -> Relation {
        match Collision::detect(circle, self) {
            Relation::Contain => Relation::Contained,
            Relation::Contained => Relation::Contain,
            r => r,
        }
    }
}

impl UpdateCollision for CollisionRect {
    fn update() -> impl FnOnce(&mut Self, &GlobalTransform) {
        |rect, global_transform| {
            debug_assert_eq!(
                global_transform.rotation(),
                Quat::IDENTITY,
                "Rotation is not supported for CollisionRect,
                use CollisionRotatedRect and add it to plugin generic params instead."
            );
            rect.rect = Rect::from_center_size(
                global_transform.translation().truncate(),
                rect.init_size * global_transform.scale().truncate(),
            );
        }
    }
}

impl Disassemble for CollisionRect {
    fn disassemble(
        &self,
    ) -> (
        Vec<&CollisionRect>,
        Vec<&CollisionRotatedRect>,
        Vec<&CollisionCircle>,
    ) {
        unreachable!()
    }

    fn detect(&self, obj: &dyn DynCollision) -> Relation {
        let mut relation = Relation::Contain;
        match obj.detect(self) {
            Relation::Contain => return Relation::Contained,
            Relation::Contained if relation == Relation::Contain => {}
            Relation::Contained | Relation::Overlap => return Relation::Overlap,
            Relation::Disjoint => relation = Relation::Disjoint,
        }
        relation
    }
}
