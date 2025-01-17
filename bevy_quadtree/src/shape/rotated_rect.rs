use crate::{
    Collision, CollisionCircle, CollisionRect, Disassemble, DynCollision, Relation, UpdateCollision,
};
use bevy::prelude::*;
use std::ops::Deref;

/// Rotated Rectagle shape implemented [`AsCollision`](crate::AsCollision) trait to be used in the QuadTreePlugin
/// and as a Component in the ECS.
///
/// Also, implementes [`Disassemble`](crate::Disassemble) trait to be used in the [`QuadTree::query`](crate::QuadTree::query).
#[derive(Debug, Component, Clone)]
pub struct CollisionRotatedRect {
    rect: Rect,
    init_size: Vec2,
    rotation: Quat,
}

impl Deref for CollisionRotatedRect {
    type Target = Rect;

    fn deref(&self) -> &Self::Target {
        &self.rect
    }
}

impl From<Rect> for CollisionRotatedRect {
    /// Create a new CollisionRotatedRect from a Rect with the default rotation.
    fn from(rect: Rect) -> Self {
        Self {
            rect,
            init_size: rect.size(),
            rotation: Quat::IDENTITY,
        }
    }
}

impl CollisionRotatedRect {
    fn new(rect: Rect, rotation: Quat) -> Self {
        Self {
            rect,
            init_size: rect.size(),
            rotation,
        }
    }
}

impl Collision<CollisionRect> for CollisionRotatedRect {
    fn detect(&self, rect: &CollisionRect) -> Relation {
        match Collision::detect(rect, self) {
            Relation::Contain => Relation::Contained,
            Relation::Contained => Relation::Contain,
            r => r,
        }
    }
}

impl Collision<CollisionRotatedRect> for CollisionRotatedRect {
    fn detect(&self, r_rect: &CollisionRotatedRect) -> Relation {
        todo!()
    }
}

impl Collision<CollisionCircle> for CollisionRotatedRect {
    fn detect(&self, circle: &CollisionCircle) -> Relation {
        match Collision::detect(circle, self) {
            Relation::Contain => Relation::Contained,
            Relation::Contained => Relation::Contain,
            r => r,
        }
    }
}

impl UpdateCollision for CollisionRotatedRect {
    fn update() -> impl FnOnce(&mut Self, &GlobalTransform) {
        |rect, global_transform| {
            rect.rect = Rect::from_center_size(
                global_transform.translation().truncate(),
                rect.init_size * global_transform.scale().truncate(),
            );
            rect.rotation = global_transform.rotation();
        }
    }
}

impl Disassemble for CollisionRotatedRect {
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
