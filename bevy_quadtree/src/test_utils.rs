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

impl Collision for MyCircle {
    fn check(&self, _: Rect) -> RelativePosition {
        todo!()
    }
}

#[derive(Debug, Component)]
pub struct MyRect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Collision for MyRect {
    fn check(&self, _: Rect) -> RelativePosition {
        todo!()
    }
}
