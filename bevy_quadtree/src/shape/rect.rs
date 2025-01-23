use bevy_ecs::prelude::*;
#[cfg(feature = "sprite")]
use bevy_log::warn;
use bevy_math::prelude::*;
#[cfg(feature = "sprite")]
use bevy_sprite::Sprite;
use bevy_transform::components::GlobalTransform;
use core::fmt;
use std::any::type_name;

use crate::{
    collision::{Collision, CollisionQuery, DynCollision, Relation, UpdateCollision},
    CollisionCircle, CollisionRotatedRect,
};

/// Rectagle shape to be used in the QuadTreePlugin
/// and as a Component in the ECS.
///
/// Also, implemented [`CollisionQuery`] trait to be used as boundary in the [`QuadTree::query`](crate::QuadTree::query).
///
/// # Panic
/// Rotation is not supported for CollisionRect, see [`CollisionRotatedRect`] instead.
#[derive(Component, Clone)]
pub struct CollisionRect<const ID: usize = 0> {
    pub(crate) center: Vec2,
    pub(crate) scale: Vec2,
    init_size: Vec2,
}

impl<const ID: usize> fmt::Debug for CollisionRect<ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: center = ({}, {}); size = ({} x {}) x ({} x {}) = {} x {}",
            type_name::<Self>(),
            self.center.x,
            self.center.y,
            self.init_size.x,
            self.scale.x,
            self.init_size.y,
            self.scale.y,
            self.init_size.x * self.scale.x,
            self.init_size.y * self.scale.y
        )
    }
}

impl<const ID: usize> From<&CollisionRect<ID>> for CollisionRect<0> {
    /// Convert the shape with `ID` to the shape with `ID = 0`.
    /// Used to eliminate the `ID` in the collision detection.
    fn from(value: &CollisionRect<ID>) -> Self {
        Self {
            center: value.center,
            scale: value.scale,
            init_size: value.init_size,
        }
    }
}

impl From<Rect> for CollisionRect {
    /// Create a new CollisionRect from a Rect with `ID = 0`.
    ///
    /// See [`Self::new`] for details.
    fn from(rect: Rect) -> Self {
        Self {
            center: rect.center(),
            scale: Vec2::ONE,
            init_size: rect.size(),
        }
    }
}

impl CollisionRect {
    /// Create a new CollisionRect from a Rect with `ID = 0`.
    ///
    /// The initial size is set to the size of the rect.
    /// It is used to compute the size with the GlobalTransform's scale.
    ///
    /// The initial center is set to the center of the rect.
    /// It is covered by the GlobalTransform's translation during the update.
    pub fn new(rect: Rect) -> Self {
        rect.into()
    }
}

impl<const ID: usize> CollisionRect<ID> {
    /// Create a new CollisionRect from a Rect with given `ID`.
    pub fn new_id(rect: Rect) -> Self {
        Self {
            center: rect.center(),
            scale: Vec2::ONE,
            init_size: rect.size(),
        }
    }

    pub(crate) fn size(&self) -> Vec2 {
        self.init_size * self.scale
    }

    pub(crate) fn min(&self) -> Vec2 {
        self.center - self.size() / 2.
    }

    pub(crate) fn max(&self) -> Vec2 {
        self.center + self.size() / 2.
    }

    /// Set the initial size of the rect, which is used to compute the size with the GlobalTransform's scale.
    pub fn set_init_size(&mut self, size: Vec2) {
        self.init_size = size;
    }
}

impl Collision<CollisionRect> for CollisionRect {
    fn detect(&self, rect: &CollisionRect) -> Relation {
        let self_min = self.min();
        let self_max = self.max();
        let rect_min = rect.min();
        let rect_max = rect.max();
        if self_min.x < rect_min.x
            && self_min.y < rect_min.y
            && self_max.x > rect_max.x
            && self_max.y > rect_max.y
        {
            Relation::Contain
        } else if self_max.x < rect_min.x
            || self_min.x > rect_max.x
            || self_max.y < rect_min.y
            || self_min.y > rect_max.y
        {
            Relation::Disjoint
        } else if self_min.x > rect_min.x
            && self_min.y > rect_min.y
            && self_max.x < rect_max.x
            && self_max.y < rect_max.y
        {
            Relation::Contained
        } else {
            Relation::Overlap
        }
    }
}

impl Collision<CollisionRotatedRect> for CollisionRect {
    fn detect(&self, r_rect: &CollisionRotatedRect) -> Relation {
        let r_half_size = r_rect.init_size * r_rect.scale / 2.;
        let mut vetex = [
            Vec2::new(r_half_size.x, r_half_size.y),
            Vec2::new(-r_half_size.x, r_half_size.y),
            Vec2::new(-r_half_size.x, -r_half_size.y),
            Vec2::new(r_half_size.x, -r_half_size.y),
        ];
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
        );
        for v in vetex.iter_mut() {
            *v = r_rect.isometric * *v;
            min_x = min_x.min(v.x);
            max_x = max_x.max(v.x);
            min_y = min_y.min(v.y);
            max_y = max_y.max(v.y);
        }
        let self_min = self.min();
        let self_max = self.max();
        if self_max.x < min_x || self_min.x > max_x || self_max.y < min_y || self_min.y > max_y {
            return Relation::Disjoint;
        } else if self_min.x < min_x
            && max_x < self_max.x
            && self_min.y < min_y
            && max_y < self_max.y
        {
            return Relation::Contain;
        }
        let mut vetex = [
            self_max,
            Vec2::new(self_min.x, self_max.y),
            self_min,
            Vec2::new(self_max.x, self_min.y),
        ];
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
        );
        let inv = r_rect.isometric.inverse();
        for v in vetex.iter_mut() {
            *v = inv * *v;
            min_x = min_x.min(v.x);
            max_x = max_x.max(v.x);
            min_y = min_y.min(v.y);
            max_y = max_y.max(v.y);
        }
        if r_half_size.x < min_x
            || -r_half_size.x > max_x
            || r_half_size.y < min_y
            || -r_half_size.y > max_y
        {
            return Relation::Disjoint;
        } else if -r_half_size.x < min_x
            && max_x < r_half_size.x
            && -r_half_size.y < min_y
            && max_y < r_half_size.y
        {
            return Relation::Contained;
        }
        Relation::Overlap
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

impl<const ID: usize> UpdateCollision<GlobalTransform> for CollisionRect<ID> {
    fn update() -> impl FnOnce(Mut<Self>, &GlobalTransform) {
        |mut rect, global_transform| {
            debug_assert_eq!(
                global_transform.rotation(),
                Quat::IDENTITY,
                "Rotation is not supported for CollisionRect,
                use `CollisionRotatedRect` and add it to plugin generic params instead."
            );
            rect.scale = global_transform.scale().truncate();
            rect.center = global_transform.translation().truncate();
        }
    }
}

#[cfg(feature = "sprite")]
impl<const ID: usize> UpdateCollision<Sprite> for CollisionRect<ID> {
    fn update() -> impl FnOnce(Mut<Self>, &Sprite) {
        |mut rect, sprite| {
            if let Some(size) = sprite.custom_size {
                if size != rect.init_size {
                    rect.init_size = size;
                }
            } else {
                warn!("Tracking sprite with no custom size");
            }
        }
    }
}

impl CollisionQuery for CollisionRect {
    fn query(&self, obj: &dyn DynCollision) -> Relation {
        match obj.detect(self) {
            Relation::Contain => Relation::Contained,
            Relation::Contained => Relation::Contain,
            r => r,
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_math::Rot2;

    use super::*;
    use std::f32::consts::*;

    #[test]
    fn collision_rect_detect() {
        let rect = CollisionRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE));
        let contain = CollisionRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE / 2.));
        let contained = CollisionRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE * 2.));
        let disjoint = CollisionRect::from(Rect::from_center_size(Vec2::new(2., 2.), Vec2::ONE));
        let overlap = CollisionRect::from(Rect::from_center_size(Vec2::new(0.5, 0.5), Vec2::ONE));
        assert_eq!(rect.detect(&contain), Relation::Contain);
        assert_eq!(rect.detect(&contained), Relation::Contained);
        assert_eq!(rect.detect(&disjoint), Relation::Disjoint);
        assert_eq!(rect.detect(&overlap), Relation::Overlap);
    }

    #[test]
    fn collision_rect_detect_circle() {
        let rect = CollisionRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE));
        let contain = CollisionCircle::new(Vec2::ZERO, 0.4);
        let contained = CollisionCircle::new(Vec2::ZERO, 2.);
        let disjoint = CollisionCircle::new(Vec2::new(2., 2.), 1.);
        let overlap = CollisionCircle::new(Vec2::new(0.5, 0.5), 1.);
        assert_eq!(rect.detect(&contain), Relation::Contain);
        assert_eq!(rect.detect(&contained), Relation::Contained);
        assert_eq!(rect.detect(&disjoint), Relation::Disjoint);
        assert_eq!(rect.detect(&overlap), Relation::Overlap);
    }

    #[test]
    fn collision_rect_detect_rotated_rect() {
        let rect = CollisionRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE));
        let contain = CollisionRotatedRect::new(
            Rect::from_center_size(Vec2::ZERO, Vec2::ONE / 2.),
            Rot2::radians(FRAC_PI_4),
        );
        let contained = CollisionRotatedRect::new(
            Rect::from_center_size(Vec2::ZERO, Vec2::ONE * 3.),
            Rot2::radians(FRAC_PI_4),
        );
        let disjoint = CollisionRotatedRect::new(
            Rect::from_center_size(Vec2::new(2., 2.), Vec2::ONE),
            Rot2::radians(FRAC_PI_4),
        );
        let overlap = CollisionRotatedRect::new(
            Rect::from_center_size(Vec2::new(0.5, 0.5), Vec2::ONE),
            Rot2::radians(FRAC_PI_4),
        );
        assert_eq!(rect.detect(&contain), Relation::Contain);
        assert_eq!(rect.detect(&contained), Relation::Contained);
        assert_eq!(rect.detect(&disjoint), Relation::Disjoint);
        assert_eq!(rect.detect(&overlap), Relation::Overlap);
    }
}
