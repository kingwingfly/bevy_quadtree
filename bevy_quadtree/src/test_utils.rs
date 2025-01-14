#![allow(dead_code)]

#[cfg(test)]
use crate::{Collision, RelativePosition};
use bevy::prelude::*;
#[cfg(not(test))]
use bevy_quadtree::{Collision, RelativePosition};

#[derive(Debug, Component)]
pub struct MyCircle {
    pub center: Vec2,
    pub radius: f32,
}

impl Collision<Rect> for MyCircle {
    fn detect(&self, _: Rect) -> RelativePosition {
        todo!()
    }
}

impl Collision<Line2d> for MyCircle {
    fn detect(&self, _: Line2d) -> RelativePosition {
        todo!()
    }
}

#[derive(Debug, Component)]
pub struct MyRect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Collision<Rect> for MyRect {
    fn detect(&self, _: Rect) -> RelativePosition {
        todo!()
    }
}

impl Collision<Line2d> for MyRect {
    fn detect(&self, _: Line2d) -> RelativePosition {
        todo!()
    }
}
