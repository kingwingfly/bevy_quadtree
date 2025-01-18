use crate::{
    collision::{DynCollision, Relation},
    Collision, CollisionCircle, CollisionQuery, CollisionRotatedRect, UpdateCollision,
};
use bevy::prelude::*;
use std::ops::Deref;

/// Rectagle shape implemented [`AsCollision`](crate::collision::AsCollision) trait to be used in the QuadTreePlugin
/// and as a Component in the ECS.
///
/// Also, implemented [`CollisionQuery`] trait to be used as boundary in the [`QuadTree::query`](crate::QuadTree::query).
///
/// # Panic
/// Rotation is not supported for CollisionRect, see [`CollisionRotatedRect`] instead.
#[derive(Debug, Component, Clone)]
pub struct CollisionRect {
    pub(crate) rect: Rect,
    init_size: Vec2,
}

impl Deref for CollisionRect {
    type Target = Rect;

    fn deref(&self) -> &Self::Target {
        &self.rect
    }
}

impl From<Rect> for CollisionRect {
    fn from(rect: Rect) -> Self {
        Self {
            rect,
            init_size: rect.size(),
        }
    }
}

impl CollisionRect {
    /// Create a new CollisionRect from a Rect.
    /// The initial size is set to the size of the rect.
    /// It is used to compute the size with the GlobalTransform's scale.
    ///
    /// The initial center is set to the center of the rect.
    /// It is covered by the GlobalTransform's translation during the update.
    pub fn new(rect: Rect) -> Self {
        rect.into()
    }

    /// Set the initial size of the rect, which is used to compute the size with the GlobalTransform's scale.
    pub fn set_init_size(&mut self, size: Vec2) {
        self.init_size = size;
    }
}

impl Collision<CollisionRect> for CollisionRect {
    fn detect(&self, rect: &CollisionRect) -> Relation {
        if self.min.x < rect.min.x
            && self.min.y < rect.min.y
            && self.max.x > rect.max.x
            && self.max.y > rect.max.y
        {
            Relation::Contain
        } else if self.max.x < rect.min.x
            || self.min.x > rect.max.x
            || self.max.y < rect.min.y
            || self.min.y > rect.max.y
        {
            Relation::Disjoint
        } else if self.min.x > rect.min.x
            && self.min.y > rect.min.y
            && self.max.x < rect.max.x
            && self.max.y < rect.max.y
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
        if self.max.x < min_x || self.min.x > max_x || self.max.y < min_y || self.min.y > max_y {
            return Relation::Disjoint;
        } else if self.min.x < min_x
            && max_x < self.max.x
            && self.min.y < min_y
            && max_y < self.max.y
        {
            return Relation::Contain;
        }
        let mut vetex = [
            self.max,
            Vec2::new(self.min.x, self.max.y),
            self.min,
            Vec2::new(self.max.x, self.min.y),
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

impl UpdateCollision for CollisionRect {
    fn update() -> impl FnOnce(&mut Self, &GlobalTransform) {
        |rect, global_transform| {
            debug_assert_eq!(
                global_transform.rotation(),
                Quat::IDENTITY,
                "Rotation is not supported for CollisionRect,
                use `CollisionRotatedRect` and add it to plugin generic params instead."
            );
            rect.rect = Rect::from_center_size(
                global_transform.translation().truncate(),
                rect.init_size * global_transform.scale().truncate(),
            );
        }
    }
}

impl CollisionQuery for CollisionRect {
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
