use crate::{DynCollision, Relation};
use bevy::{
    ecs::entity::EntityHashMap,
    math::{Rect, Vec2},
    prelude::Entity,
};
use core::fmt;
use std::sync::{Arc, RwLock};

pub type ArcNode<const N: usize, const K: usize> = Arc<RwLock<Node<N, K>>>;

pub(crate) struct Node<const N: usize, const K: usize = 10> {
    entities: EntityHashMap<Arc<dyn DynCollision>>,
    inlet_boundary: Rect,
    outlet_boundary: Rect,
    parent: Option<ArcNode<N, K>>,
    children: Option<[ArcNode<N, K>; 4]>,
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
            entities: EntityHashMap::default(),
            inlet_boundary: boundary,
            outlet_boundary: Rect::from_center_size(
                boundary.center(),
                boundary.size() * (K as f32 / 10.),
            ),
            parent: None,
            children: None,
        }
    }
}

impl<const N: usize, const K: usize> Node<N, K> {
    fn new_with_parent(boundary: Rect, parent: ArcNode<N, K>) -> Self {
        Self {
            entities: EntityHashMap::default(),
            inlet_boundary: boundary,
            outlet_boundary: Rect::from_center_size(
                boundary.center(),
                boundary.size() * (K as f32 / 10.),
            ),
            parent: Some(parent),
            children: None,
        }
    }

    fn len(&self) -> usize {
        self.entities.len()
    }

    pub(crate) fn update_arc(
        &mut self,
        entity: Entity,
        old: Arc<dyn DynCollision>,
        new: Arc<dyn DynCollision>,
    ) -> ArcNode<N, K> {
        // lock children first
        todo!()
    }

    pub(crate) fn insert_arc(
        this: &ArcNode<N, K>,
        entity: Entity,
        shape: Arc<dyn DynCollision>,
    ) -> ArcNode<N, K> {
        {
            let mut this_w = this.write().unwrap();

            if this_w.children.is_none() {
                if this_w.len() >= N {
                    drop(this_w);
                    Self::divide(this);
                } else {
                    this_w.entities.insert(entity, shape);
                    return Arc::clone(this);
                }
            }
        }

        let this_r = this.read().unwrap();
        for node in this_r.children.as_ref().unwrap().iter() {
            let node_r = node.read().unwrap();
            match shape.detect(node_r.inlet_boundary) {
                Relation::Disjoint | Relation::ExternallyTangent => {}
                Relation::PartiallyOverlap | Relation::InternallyTangented | Relation::Contain => {
                    match &this_r.parent {
                        Some(p) => {
                            return Self::insert_arc(p, entity, shape);
                        }
                        None => {
                            drop(node_r);
                            let mut this_w = this.write().unwrap();
                            this_w.entities.insert(entity, shape);
                            return Arc::clone(this);
                        }
                    }
                }
                Relation::InternallyTangent | Relation::Contained => {
                    drop(node_r);
                    return Self::insert_arc(node, entity, shape);
                }
            }
        }
        unreachable!()
    }

    fn divide(this: &ArcNode<N, K>) {
        let mut this_w = this.write().unwrap();
        debug_assert!(this_w.children.is_none());
        let delta = this_w.inlet_boundary.size() / 2.;
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
        this_w.children = Some(core::array::from_fn(|i| {
            Arc::new(RwLock::new(Node::new_with_parent(
                Rect {
                    min: this_w.inlet_boundary.min + MIN[i] * delta,
                    max: this_w.inlet_boundary.min + MAX[i] * delta,
                },
                Arc::clone(this),
            )))
        }));
        let drain = this_w.entities.drain().collect::<Vec<_>>();
        drop(this_w);
        for (entity, shape) in drain.into_iter() {
            Self::insert_arc(this, entity, shape);
        }
    }

    fn remove(&mut self, entity: Entity) {
        todo!()
    }

    fn query(&self, boundary: Rect) -> Vec<Entity> {
        todo!()
    }
}
