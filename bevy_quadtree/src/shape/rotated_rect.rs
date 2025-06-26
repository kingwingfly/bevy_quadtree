use crate::{
    CollisionCircle, CollisionRect,
    collision::{Collision, CollisionQuery, DynCollision, Relation, UpdateCollision},
};
use bevy_ecs::prelude::*;
#[cfg(feature = "sprite")]
use bevy_log::warn;
use bevy_math::prelude::*;
#[cfg(feature = "sprite")]
use bevy_sprite::Sprite;
use bevy_transform::components::GlobalTransform;
use core::fmt;
use std::any::type_name;

/// Rotated Rectagle shape to be used in the QuadTreePlugin
/// and as a Component in the ECS.
///
/// Also, implemented [`CollisionQuery`] trait to be used as boundary in the [`QuadTree::query`](crate::QuadTree::query).
#[derive(Component, Clone)]
pub struct CollisionRotatedRect<const ID: usize = 0> {
    pub(crate) init_size: Vec2,
    pub(crate) scale: Vec2,
    pub(crate) isometric: Isometry2d,
}

impl<const ID: usize> fmt::Debug for CollisionRotatedRect<ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: center = ({}, {}); size = ({} x {}) x ({} x {}) = {} x {}; totation = {}",
            type_name::<Self>(),
            self.isometric.translation.x,
            self.isometric.translation.y,
            self.init_size.x,
            self.scale.x,
            self.init_size.y,
            self.scale.y,
            self.init_size.x * self.scale.x,
            self.init_size.y * self.scale.y,
            self.isometric.rotation.as_degrees()
        )
    }
}

impl<const ID: usize> From<&CollisionRotatedRect<ID>> for CollisionRotatedRect<0> {
    /// Convert the shape with `ID` to the shape with `ID = 0`.
    /// Used to eliminate the `ID` in the collision detection.
    fn from(value: &CollisionRotatedRect<ID>) -> Self {
        Self {
            init_size: value.init_size,
            scale: value.scale,
            isometric: value.isometric,
        }
    }
}

impl From<Rect> for CollisionRotatedRect {
    /// Create a new CollisionRotatedRect from a Rect with the default scale and rotation.
    ///
    /// See [`Self::new`] for the version with rotation.
    fn from(rect: Rect) -> Self {
        Self {
            init_size: rect.size(),
            scale: Vec2::ONE,
            isometric: Isometry2d::new(rect.center(), Rot2::IDENTITY),
        }
    }
}

impl CollisionRotatedRect {
    /// Create a new CollisionRotatedRect with `ID = 0`, and given Rect, rotation. See [`Self::new_id`] for the version with `ID`.
    ///
    /// The rotation is relative to the center of the rect.
    /// When updating, the rotation from the GlobalTransform will directly cover the isometric,
    /// instead of based on the initial rotation.
    pub fn new(rect: Rect, rotation: Rot2) -> Self {
        Self {
            init_size: rect.size(),
            scale: Vec2::ONE,
            isometric: Isometry2d::new(rect.center(), rotation),
        }
    }
}

impl<const ID: usize> CollisionRotatedRect<ID> {
    /// Create a new CollisionRotatedRect with the given ID, Rect and rotation.
    pub fn new_id(rect: Rect, rotation: Rot2) -> Self {
        Self {
            init_size: rect.size(),
            scale: Vec2::ONE,
            isometric: Isometry2d::new(rect.center(), rotation),
        }
    }

    /// Set the initial size of the rect, which is used to compute the size with the GlobalTransform's scale.
    pub fn set_init_size(&mut self, size: Vec2) {
        self.init_size = size;
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
        let self_half_size = self.init_size * self.scale / 2.;
        let r_half_size = r_rect.init_size * r_rect.scale / 2.;
        let mut vertex = [
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
        let inv = self.isometric.inverse();
        for v in vertex.iter_mut() {
            *v = inv * r_rect.isometric * *v;
            min_x = min_x.min(v.x);
            max_x = max_x.max(v.x);
            min_y = min_y.min(v.y);
            max_y = max_y.max(v.y);
        }
        if self_half_size.x < min_x
            || -self_half_size.x > max_x
            || self_half_size.y < min_y
            || -self_half_size.y > max_y
        {
            return Relation::Disjoint;
        } else if -self_half_size.x < min_x
            && max_x < self_half_size.x
            && -self_half_size.y < min_y
            && max_y < self_half_size.y
        {
            return Relation::Contain;
        }
        let mut vertex = [
            Vec2::new(self_half_size.x, self_half_size.y),
            Vec2::new(-self_half_size.x, self_half_size.y),
            Vec2::new(-self_half_size.x, -self_half_size.y),
            Vec2::new(self_half_size.x, -self_half_size.y),
        ];
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
        );
        let inv = r_rect.isometric.inverse();
        for v in vertex.iter_mut() {
            *v = inv * self.isometric * *v;
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

impl Collision<CollisionCircle> for CollisionRotatedRect {
    fn detect(&self, circle: &CollisionCircle) -> Relation {
        match Collision::detect(circle, self) {
            Relation::Contain => Relation::Contained,
            Relation::Contained => Relation::Contain,
            r => r,
        }
    }
}

impl<const ID: usize> UpdateCollision<GlobalTransform> for CollisionRotatedRect<ID> {
    fn update() -> impl FnOnce(Mut<Self>, &GlobalTransform) {
        |mut rect, global_transform| {
            debug_assert_eq!(
                global_transform.rotation().to_axis_angle().0.z,
                1.,
                "Rotation for CollisionRotatedRect should around z-axis only"
            );
            rect.scale = global_transform.scale().truncate();
            rect.isometric = Isometry2d::new(
                global_transform.translation().truncate(),
                Rot2::radians(global_transform.rotation().to_axis_angle().1),
            );
        }
    }
}

#[cfg(feature = "sprite")]
impl<const ID: usize> UpdateCollision<Sprite> for CollisionRotatedRect<ID> {
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

impl CollisionQuery for CollisionRotatedRect {
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
    use super::*;
    use std::f32::consts::*;

    #[test]
    fn collision_rotated_rect_detect() {
        let rect = CollisionRotatedRect::new(
            Rect::from_center_size(Vec2::ZERO, Vec2::ONE),
            Rot2::radians(FRAC_PI_4),
        );
        let contain =
            CollisionRotatedRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE / 2.));
        let contained = CollisionRotatedRect::new(
            Rect::from_center_size(Vec2::ZERO, Vec2::ONE * 1.4),
            Rot2::degrees(15.),
        );
        let disjoint = CollisionRotatedRect::from(Rect::from_center_size(Vec2::ONE, Vec2::ONE));
        let overlap =
            CollisionRotatedRect::from(Rect::from_center_size(Vec2::new(0.5, 0.5), Vec2::ONE));
        assert_eq!(rect.detect(&contain), Relation::Contain);
        assert_eq!(rect.detect(&contained), Relation::Contained);
        assert_eq!(rect.detect(&disjoint), Relation::Disjoint);
        assert_eq!(rect.detect(&overlap), Relation::Overlap);
    }

    #[test]
    fn collision_rotated_rect_detect_rect() {
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
    fn collision_rotated_rect_detect_circle() {
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
