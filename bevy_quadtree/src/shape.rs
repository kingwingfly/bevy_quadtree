use crate::{Collision, Relation, UpdateCollision};
use bevy::prelude::*;

/// Circle shape implemented `AsCollision` trait to be used in the quadtree
/// and as a Component in the ECS
#[allow(missing_docs)]
#[derive(Debug, Component, Clone)]
pub struct CollisionCircle {
    pub center: Vec2,
    pub radius: f32,
}

impl Collision<Rect> for CollisionCircle {
    fn detect(&self, _: Rect) -> Relation {
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

/// Rectagle shape implemented `AsCollision` trait to be used in the quadtree
/// and as a Component in the ECS
#[derive(Debug, Component, Clone)]
pub struct CollisionRect {
    /// The minimum point of the rectangle
    pub min: Vec2,
    /// The maximum point of the rectangle
    pub max: Vec2,
}

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
