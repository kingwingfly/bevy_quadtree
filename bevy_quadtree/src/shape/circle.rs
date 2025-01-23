use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_transform::components::GlobalTransform;
use core::fmt;
use std::any::type_name;

use crate::{
    collision::{Collision, CollisionQuery, DynCollision, Relation, UpdateCollision},
    CollisionRect, CollisionRotatedRect,
};

/// Circle shape to be used in the QuadTreePlugin
/// and as a Component in the ECS.
///
/// Also, implemented [`CollisionQuery`] trait to be used as boundary in the [`QuadTree::query`](crate::QuadTree::query).
///
/// # Panic
/// Do not perform scaling with different x and y values, it will cause the circle to be an ellipse,
/// and the collision detection will be incorrect.
#[derive(Component, Clone)]
pub struct CollisionCircle<const ID: usize = 0> {
    pub(crate) center: Vec2,
    scale: f32,
    init_radius: f32,
}

impl<const ID: usize> fmt::Debug for CollisionCircle<ID> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: center = ({}, {}); r = {} x {} = {}",
            type_name::<Self>(),
            self.center.x,
            self.center.y,
            self.init_radius,
            self.scale,
            self.init_radius * self.scale
        )
    }
}

impl<const ID: usize> From<&CollisionCircle<ID>> for CollisionCircle<0> {
    /// Convert the shape with `ID` to the shape with `ID = 0`.
    /// Used to eliminate the `ID` in the collision detection.
    fn from(value: &CollisionCircle<ID>) -> Self {
        Self {
            center: value.center,
            scale: value.scale,
            init_radius: value.init_radius,
        }
    }
}

impl CollisionCircle {
    /// Create a new circle with `ID = 0`. See [`Self::new_id`] for the version with `ID`.
    ///
    /// The initial radius is used to compute the size with the GlobalTransform's scale.
    ///
    /// The initial center is covered by the GlobalTransform's translation during the update.
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self {
            center,
            scale: 1.,
            init_radius: radius,
        }
    }
}

impl<const ID: usize> CollisionCircle<ID> {
    /// Create a new circle with the given `ID`.
    pub fn new_id(center: Vec2, radius: f32) -> Self {
        Self {
            center,
            scale: 1.,
            init_radius: radius,
        }
    }

    fn radius(&self) -> f32 {
        self.init_radius * self.scale
    }

    /// Set the initial radius of the circle, which is used to compute the radius with the GlobalTransform's scale.
    pub fn set_init_radius(&mut self, radius: f32) {
        self.init_radius = radius;
    }
}

impl Collision<CollisionRect> for CollisionCircle {
    fn detect(&self, rect: &CollisionRect) -> Relation {
        let rect_max = rect.max();
        let rect_min = rect.min();
        let i = rect_max - rect.center; // move rect center to origin and get vertex in Quadrant I
        let center = (self.center - rect.center).abs(); // move circle with rect and symmetrize the circle to Quadrant I
        let ds = [
            (self.center - rect_max).length(),
            (self.center - Vec2::new(rect_min.x, rect_max.y)).length(),
            (self.center - rect_min).length(),
            (self.center - Vec2::new(rect_max.x, rect_min.y)).length(),
        ];
        let radius = self.radius();
        if ds.iter().all(|&d| d < radius) {
            Relation::Contain
        } else if center.x > i.x + radius || center.y > i.y + radius {
            Relation::Disjoint
        } else if center.x < i.x - radius && center.y < i.y - radius {
            Relation::Contained
        } else if center.x > i.x && center.y > i.y && (i - center).length() > radius {
            Relation::Disjoint
        } else {
            Relation::Overlap
        }
    }
}

impl Collision<CollisionRotatedRect> for CollisionCircle {
    fn detect(&self, r_rect: &CollisionRotatedRect) -> Relation {
        let r_rect_size = r_rect.init_size * r_rect.scale;
        let i = r_rect_size / 2.; // Move rect center to origin, rotate back and get vertex in Quadrant I
        let center = (r_rect.isometric.inverse() * self.center).abs(); // Move circle with rect, rotate around origin and symmetrize the circle to Quadrant I
        let ds = [
            (center - i).length(),
            (center - Vec2::new(-i.x, i.y)).length(),
            (center - Vec2::new(-i.x, -i.y)).length(),
            (center - Vec2::new(i.x, -i.y)).length(),
        ];
        let radius = self.radius();
        if ds.iter().all(|&d| d < radius) {
            Relation::Contain
        } else if center.x > i.x + radius || center.y > i.y + radius {
            Relation::Disjoint
        } else if center.x < i.x - radius && center.y < i.y - radius {
            Relation::Contained
        } else if center.x > i.x && center.y > i.y && ds[0] > radius {
            Relation::Disjoint
        } else {
            Relation::Overlap
        }
    }
}

impl Collision<CollisionCircle> for CollisionCircle {
    fn detect(&self, circle: &CollisionCircle) -> Relation {
        let d = (self.center - circle.center).length();
        let self_r = self.radius();
        let circle_r = circle.radius();
        if d + circle_r < self_r {
            Relation::Contain
        } else if d + self_r < circle_r {
            Relation::Contained
        } else if d > self_r + circle_r {
            Relation::Disjoint
        } else {
            Relation::Overlap
        }
    }
}

impl<const ID: usize> UpdateCollision<GlobalTransform> for CollisionCircle<ID> {
    fn update() -> impl FnOnce(Mut<Self>, &GlobalTransform) {
        |mut circle, global_transform| {
            circle.center = global_transform.translation().truncate();
            debug_assert_eq!(
                global_transform.scale().x, global_transform.scale().y,
                "Do not perform scaling with different x and y values,
                it will cause the circle to be an ellipse, and the collision detection will be incorrect."
            );
            circle.scale = global_transform.scale().x;
        }
    }
}

impl CollisionQuery for CollisionCircle {
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
    use bevy_math::{Rect, Rot2};

    use super::*;
    use std::f32::consts::*;

    #[test]
    fn collision_circle_detect() {
        let circle = CollisionCircle::new(Vec2::ZERO, 1.);
        let contain = CollisionCircle::new(Vec2::ZERO, 0.5);
        let contained = CollisionCircle::new(Vec2::ZERO, 2.);
        let disjoint = CollisionCircle::new(Vec2::new(2., 2.), 1.);
        let overlap = CollisionCircle::new(Vec2::new(0.5, 0.5), 1.);
        assert_eq!(circle.detect(&contain), Relation::Contain);
        assert_eq!(circle.detect(&contained), Relation::Contained);
        assert_eq!(circle.detect(&disjoint), Relation::Disjoint);
        assert_eq!(circle.detect(&overlap), Relation::Overlap);
    }

    #[test]
    fn collision_circle_detect_rect() {
        let circle = CollisionCircle::new(Vec2::ZERO, 1.);
        let contain = CollisionRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE / 2.));
        let contained = CollisionRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE * 3.));
        let disjoint = CollisionRect::from(Rect::from_center_size(Vec2::new(2., 2.), Vec2::ONE));
        let overlap = CollisionRect::from(Rect::from_center_size(Vec2::new(0.5, 0.5), Vec2::ONE));
        assert_eq!(circle.detect(&contain), Relation::Contain);
        assert_eq!(circle.detect(&contained), Relation::Contained);
        assert_eq!(circle.detect(&disjoint), Relation::Disjoint);
        assert_eq!(circle.detect(&overlap), Relation::Overlap);
    }

    #[test]
    fn collision_circal_detect_rotated_rect() {
        let circle = CollisionCircle::new(Vec2::ZERO, 1.);
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
        assert_eq!(circle.detect(&contain), Relation::Contain);
        assert_eq!(circle.detect(&contained), Relation::Contained);
        assert_eq!(circle.detect(&disjoint), Relation::Disjoint);
        assert_eq!(circle.detect(&overlap), Relation::Overlap);
        let circle = CollisionCircle::new(Vec2::ZERO, 1.);
        let contain = CollisionRotatedRect::new(
            Rect::from_center_size(Vec2::new(0.9, 0.), Vec2::new(0.2, 0.001)),
            Rot2::radians(FRAC_PI_2),
        );
        assert_eq!(circle.detect(&contain), Relation::Contain);
    }
}
