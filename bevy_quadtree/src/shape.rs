use crate::{Collision, Disassemble, Relation, UpdateCollision};
use bevy::prelude::*;

/// Circle shape implemented [`AsCollision`](crate::AsCollision) trait to be used in the QuadTreePlugin
/// and as a Component in the ECS
///
/// # Panic
/// Do not perform scaling with different x and y values, it will cause the circle to be an ellipse,
/// and the collision detection will be incorrect.
#[allow(missing_docs)]
#[derive(Debug, Component, Clone)]
pub struct CollisionCircle {
    pub center: Vec2,
    pub radius: f32,
}

impl Collision<CollisionRect> for CollisionCircle {
    fn detect(&self, rect: &CollisionRect) -> Relation {
        let i = rect.size() / 2.; // move rect center to origin and get vertex in Quadrant I
        let center = (self.center - rect.center()).abs(); // move circle with rect and symmetrize the circle to Quadrant I
        let ds = [
            (self.center - rect.max).length(),
            (self.center - rect.min).length(),
            (self.center - Vec2::new(rect.min.x, rect.max.y)).length(),
            (self.center - Vec2::new(rect.max.x, rect.min.y)).length(),
        ];
        if ds.iter().all(|&d| d < self.radius) {
            Relation::Contain
        } else if center.x > i.x + self.radius || center.y > i.y + self.radius {
            Relation::Disjoint
        } else if center.x < i.x - self.radius && center.y < i.y - self.radius {
            Relation::Contained
        } else if center.x > i.x && center.y > i.y && (i - center).length() > self.radius {
            Relation::Disjoint
        } else {
            Relation::Overlap
        }
    }
}

impl Collision<CollisionCircle> for CollisionCircle {
    fn detect(&self, circle: &CollisionCircle) -> Relation {
        let d = (self.center - circle.center).length();
        if d + circle.radius < self.radius {
            Relation::Contain
        } else if d + self.radius < circle.radius {
            Relation::Contained
        } else if d > self.radius + circle.radius {
            Relation::Disjoint
        } else {
            Relation::Overlap
        }
    }
}

impl UpdateCollision for CollisionCircle {
    fn update() -> impl FnOnce(&mut Self, &GlobalTransform) {
        |circle, global_transform| {
            circle.center = global_transform.translation().truncate();
            debug_assert_eq!(
                global_transform.scale().x, global_transform.scale().y,
                "Do not perform scaling with different x and y values, it will cause the circle to be an ellipse, and the collision detection will be incorrect."
            );
            circle.radius *= global_transform.scale().x;
        }
    }
}

impl Disassemble for CollisionCircle {
    fn disassemble(&self) -> (Vec<&CollisionRect>, Vec<&CollisionCircle>) {
        (Vec::new(), vec![self])
    }
}

/// Rectagle shape implemented [`AsCollision`](crate::AsCollision) trait to be used in the QuadTreePlugin
/// and as a Component in the ECS
///
/// # Panic
/// Rotation is not supported for CollisionRect
#[derive(Debug, Component, Clone, Deref, DerefMut)]
pub struct CollisionRect(pub Rect);

impl From<Rect> for CollisionRect {
    fn from(rect: Rect) -> Self {
        Self(rect)
    }
}

impl Collision<CollisionRect> for CollisionRect {
    fn detect(&self, rec: &CollisionRect) -> Relation {
        if self.min.x < rec.min.x
            && self.min.y < rec.min.y
            && self.max.x > rec.max.x
            && self.max.y > rec.max.y
        {
            Relation::Contain
        } else if self.max.x < rec.min.x
            || self.min.x > rec.max.x
            || self.max.y < rec.min.y
            || self.min.y > rec.max.y
        {
            Relation::Disjoint
        } else if self.min.x > rec.min.x
            && self.min.y > rec.min.y
            && self.max.x < rec.max.x
            && self.max.y < rec.max.y
        {
            Relation::Contained
        } else {
            Relation::Overlap
        }
    }
}

impl Collision<CollisionCircle> for CollisionRect {
    fn detect(&self, obj: &CollisionCircle) -> Relation {
        match Collision::detect(obj, self) {
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
                "Rotation is not supported for CollisionRect"
            );
            **rect = Rect::from_center_size(
                global_transform.translation().truncate(),
                rect.size() * global_transform.scale().truncate(),
            );
        }
    }
}

impl Disassemble for CollisionRect {
    fn disassemble(&self) -> (Vec<&CollisionRect>, Vec<&CollisionCircle>) {
        (vec![self], Vec::new())
    }
}
