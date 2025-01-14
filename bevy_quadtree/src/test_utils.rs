#![allow(dead_code)]

#[cfg(test)]
use crate::{Collision, Relation, UpdateCollision};
use bevy::prelude::*;
#[cfg(not(test))]
use bevy_quadtree::{Collision, Relation, UpdateCollision};

#[derive(Debug, Component, Clone)]
pub struct MyCircle {
    pub center: Vec2,
    pub radius: f32,
}

impl Collision<Rect> for MyCircle {
    fn detect(&self, _: Rect) -> Relation {
        todo!()
    }
}

impl Collision<Line2d> for MyCircle {
    fn detect(&self, _: Line2d) -> Relation {
        todo!()
    }
}

impl UpdateCollision for MyCircle {
    fn update() -> impl FnOnce(&mut Self, &GlobalTransform) {
        |_, _| {}
    }
}

#[derive(Debug, Component, Clone)]
pub struct MyRect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Collision<Rect> for MyRect {
    fn detect(&self, _: Rect) -> Relation {
        todo!()
    }
}

impl Collision<Line2d> for MyRect {
    fn detect(&self, _: Line2d) -> Relation {
        todo!()
    }
}

impl UpdateCollision for MyRect {
    fn update() -> impl FnOnce(&mut Self, &GlobalTransform) {
        |_, _| {}
    }
}
