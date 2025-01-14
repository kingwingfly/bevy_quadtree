use crate::{DynCollision, Relation};
use bevy::{
    ecs::entity::EntityHashMap,
    math::{Rect, Vec2},
    prelude::Entity,
};
use core::fmt;

pub(crate) struct Node<const N: usize, const K: usize = 10> {
    entities: Option<EntityHashMap<Box<dyn DynCollision>>>,
    inlet_boundary: Rect,
    outlet_boundary: Rect,
    children: Option<[Box<Node<N, K>>; 4]>,
}

impl<const N: usize, const K: usize> fmt::Debug for Node<N, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("entities", &self.len())
            .field("inlet_boundary", &self.inlet_boundary)
            .field("outlet_boundary", &self.outlet_boundary)
            .field("children", &self.children)
            .finish()
    }
}

impl<const N: usize, const K: usize> From<Rect> for Node<N, K> {
    fn from(boundary: Rect) -> Self {
        Self {
            entities: None,
            inlet_boundary: boundary,
            outlet_boundary: Rect::from_center_size(
                boundary.center(),
                boundary.size() * (K as f32 / 10.),
            ),
            children: None,
        }
    }
}

impl<const N: usize, const K: usize> Node<N, K> {
    fn len(&self) -> usize {
        self.entities.as_ref().map(|m| m.len()).unwrap_or(0)
    }

    pub(crate) fn insert<S>(&mut self, entity: Entity, shape: S)
    where
        S: DynCollision + 'static,
    {
        self.insert_box(entity, Box::new(shape));
    }

    fn insert_box(&mut self, entity: Entity, shape: Box<dyn DynCollision>) {
        match self.entities.as_mut() {
            Some(map) => match map.get_mut(&entity) {
                Some(s) => {
                    *s = shape;
                    return;
                }
                None => {
                    if map.len() < N {
                        map.insert(entity, shape);
                        return;
                    } else {
                        self.divide();
                    }
                }
            },
            None => {
                if self.children.is_none() {
                    self.entities = Some(EntityHashMap::from_iter([(entity, shape)]));
                    return;
                }
            }
        }
        for node in self.children.as_mut().unwrap().iter_mut() {
            match shape.detect(node.inlet_boundary) {
                Relation::Disjoint => todo!(),
                Relation::ExternallyTangent => todo!(),
                Relation::PartiallyOverlapping => todo!(),
                Relation::InternallyTangent => todo!(),
                Relation::CompletelyOverlapping => todo!(),
            }
        }
        unreachable!()
    }

    fn divide(&mut self) {
        let delta = self.inlet_boundary.size() / 2.;
        const MIN: [Vec2; 4] = [
            Vec2::new(1., 1.),
            Vec2::new(0., 1.),
            Vec2::new(0., 0.),
            Vec2::new(1., 0.),
        ];
        const MAX: [Vec2; 4] = [
            Vec2::new(2., 2.),
            Vec2::new(1., 2.),
            Vec2::new(1., 1.),
            Vec2::new(2., 1.),
        ];
        self.children = Some(core::array::from_fn(|i| {
            Box::new(Node::from(Rect {
                min: self.inlet_boundary.min + MIN[i] * delta,
                max: self.inlet_boundary.min + MAX[i] * delta,
            }))
        }));
        let map = self.entities.take().unwrap();
        for (entity, shape) in map.into_iter() {
            self.insert_box(entity, shape);
        }
    }

    fn remove(&mut self, entity: Entity) {
        todo!()
    }

    fn query(&self, boundary: Rect) -> Vec<Entity> {
        todo!()
    }
}
