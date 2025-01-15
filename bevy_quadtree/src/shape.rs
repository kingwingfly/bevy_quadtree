use crate::{Collision, Relation, UpdateCollision};
use bevy::prelude::*;

/// Circle shape implemented `AsCollision` trait to be used in the QuadTree
/// and as a Component in the ECS
#[allow(missing_docs)]
#[derive(Debug, Component, Clone)]
pub struct CollisionCircle {
    pub center: Vec2,
    pub radius: f32,
}

impl Collision<Rect> for CollisionCircle {
    fn detect(&self, rect: Rect) -> Relation {
        todo!()
    }
}

impl Collision<Line2d> for CollisionCircle {
    fn detect(&self, _: Line2d) -> Relation {
        todo!()
    }
}

impl UpdateCollision for CollisionCircle {
    fn update() -> impl FnOnce(&mut Self, &GlobalTransform) {
        |_, _| {}
    }
}

/// Rectagle shape implemented `AsCollision` trait to be used in the QuadTree
/// and as a Component in the ECS
#[derive(Debug, Component, Clone, Deref)]
pub struct CollisionRect(pub Rect);

impl Collision<Rect> for CollisionRect {
    fn detect(&self, _: Rect) -> Relation {
        todo!()
    }
}

impl Collision<Line2d> for CollisionRect {
    fn detect(&self, _: Line2d) -> Relation {
        todo!()
    }
}

impl UpdateCollision for CollisionRect {
    fn update() -> impl FnOnce(&mut Self, &GlobalTransform) {
        |_, _| {}
    }
}
