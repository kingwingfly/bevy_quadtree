#[cfg(test)]
use crate::{BoundCheck, RelativePosition};
use bevy::prelude::*;
#[cfg(not(test))]
use bevy_quadtree::{BoundCheck, RelativePosition};

#[derive(Debug, Component)]
pub struct MyCircle {
    pub center: Vec2,
    pub radius: f32,
}

impl BoundCheck for MyCircle {
    fn check(&self, rect: Rect) -> RelativePosition {
        todo!()
    }
}

#[derive(Debug, Component)]
pub struct MyRect {
    pub min: Vec2,
    pub max: Vec2,
}

impl BoundCheck for MyRect {
    fn check(&self, rect: Rect) -> RelativePosition {
        todo!()
    }
}
