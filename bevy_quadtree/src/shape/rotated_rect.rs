use crate::{
    collision::{DynCollision, Relation},
    Collision, CollisionCircle, CollisionQuery, CollisionRect, UpdateCollision,
};
use bevy::prelude::*;

/// Rotated Rectagle shape to be used in the QuadTreePlugin
/// and as a Component in the ECS.
///
/// Also, implemented [`CollisionQuery`] trait to be used as boundary in the [`QuadTree::query`](crate::QuadTree::query).
#[derive(Debug, Component, Clone)]
pub struct CollisionRotatedRect {
    pub(crate) init_size: Vec2,
    pub(crate) scale: Vec2,
    pub(crate) isometric: Isometry2d,
}

impl From<Rect> for CollisionRotatedRect {
    /// Create a new CollisionRotatedRect from a Rect with the default scale and rotation.
    fn from(rect: Rect) -> Self {
        Self {
            init_size: rect.size(),
            scale: Vec2::ONE,
            isometric: Isometry2d::new(rect.center(), Rot2::IDENTITY),
        }
    }
}

impl CollisionRotatedRect {
    /// Create a new CollisionRotatedRect with the given Rect and rotation.
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
        let inv = self.isometric.inverse();
        for v in vetex.iter_mut() {
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
        let mut vetex = [
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
        for v in vetex.iter_mut() {
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

impl UpdateCollision<GlobalTransform> for CollisionRotatedRect {
    fn update() -> impl FnOnce(&mut Self, &GlobalTransform) {
        |rect, global_transform| {
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

impl CollisionQuery for CollisionRotatedRect {
    fn disassemble(
        &self,
    ) -> (
        Vec<&CollisionRect>,
        Vec<&CollisionRotatedRect>,
        Vec<&CollisionCircle>,
    ) {
        unreachable!()
    }

    fn query(&self, obj: &dyn DynCollision) -> Relation {
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
