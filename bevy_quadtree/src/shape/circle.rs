use crate::{
    Collision, CollisionRect, CollisionRotatedRect, Disassemble, DynCollision, Relation,
    UpdateCollision,
};
use bevy::prelude::*;

/// Circle shape implemented [`AsCollision`](crate::AsCollision) trait to be used in the QuadTreePlugin
/// and as a Component in the ECS.
///
/// Also, implementes [`Disassemble`](crate::Disassemble) trait to be used in the [`QuadTree::query`](crate::QuadTree::query).
///
/// # Panic
/// Do not perform scaling with different x and y values, it will cause the circle to be an ellipse,
/// and the collision detection will be incorrect.
#[allow(missing_docs)]
#[derive(Debug, Component, Clone)]
pub struct CollisionCircle {
    pub center: Vec2,
    pub radius: f32,
    pub init_radius: f32,
}

impl CollisionCircle {
    #[allow(missing_docs)]
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self {
            center,
            radius,
            init_radius: radius,
        }
    }
}

impl Collision<CollisionRect> for CollisionCircle {
    fn detect(&self, rect: &CollisionRect) -> Relation {
        let i = rect.max - rect.center(); // move rect center to origin and get vertex in Quadrant I
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

impl Collision<CollisionRotatedRect> for CollisionCircle {
    fn detect(&self, obj: &CollisionRotatedRect) -> Relation {
        todo!()
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
            circle.radius = circle.init_radius * global_transform.scale().x;
        }
    }
}

impl Disassemble for CollisionCircle {
    fn disassemble(
        &self,
    ) -> (
        Vec<&CollisionRect>,
        Vec<&CollisionRotatedRect>,
        Vec<&CollisionCircle>,
    ) {
        unreachable!()
    }

    fn detect(&self, obj: &dyn DynCollision) -> Relation {
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
