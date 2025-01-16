use crate::{DynCollision, Relation};
use bevy::{
    ecs::entity::EntityHashMap,
    log::warn,
    math::{Rect, Vec2},
    prelude::Entity,
};
use core::fmt;
use parking_lot::RwLock;
use std::sync::Arc;

pub type ArcNode<const N: usize, const K: usize> = Arc<RwLock<Node<N, K>>>;

pub(crate) struct Node<const N: usize, const K: usize = 10> {
    entities: EntityHashMap<Box<dyn DynCollision>>,
    inlet_boundary: Rect,
    outlet_boundary: Rect,
    parent: Option<ArcNode<N, K>>,
    pub(crate) children: Option<[ArcNode<N, K>; 4]>,
}

impl<const N: usize, const K: usize> fmt::Debug for Node<N, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("entities", &self.len())
            .field("parent", &self.parent.is_some())
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
                boundary.size() * const { K as f32 / 10. },
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
                boundary.size() * const { K as f32 / 10. },
            ),
            parent: Some(parent),
            children: None,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.entities.len()
    }

    pub(crate) fn update(
        this: &ArcNode<N, K>,
        entity: Entity,
        shape: Box<dyn DynCollision>,
    ) -> Vec<(Entity, ArcNode<N, K>)> {
        let mut changed = vec![];
        Self::update_inner(this, entity, shape, &mut changed);
        changed
    }

    pub(crate) fn insert(
        this: &ArcNode<N, K>,
        entity: Entity,
        shape: Box<dyn DynCollision>,
    ) -> Vec<(Entity, ArcNode<N, K>)> {
        let mut changed = vec![];
        Self::insert_inner(this, entity, shape, &mut changed);
        changed
    }

    fn update_inner(
        this: &ArcNode<N, K>,
        entity: Entity,
        shape: Box<dyn DynCollision>,
        changed: &mut Vec<(Entity, ArcNode<N, K>)>,
    ) {
        let this_r = this.read();
        match shape.detect(this_r.outlet_boundary) {
            Relation::Disjoint | Relation::Contain => {
                if let Some(p) = &this_r.parent {
                    let p = Arc::clone(p);
                    drop(this_r);
                    this.write().entities.remove(&entity);
                    Self::insert_omit_children_inner(&p, entity, shape, changed);
                }
            }
            Relation::Overlap | Relation::Contained => drop(this_r),
        }
    }

    fn insert_inner(
        this: &ArcNode<N, K>,
        entity: Entity,
        shape: Box<dyn DynCollision>,
        changed: &mut Vec<(Entity, ArcNode<N, K>)>,
    ) {
        {
            let this_r = this.read();
            if this_r.children.is_none() {
                if this_r.len() >= N {
                    drop(this_r);
                    Self::divide_inner(this, changed);
                } else {
                    drop(this_r);
                    Self::insert_omit_children_inner(this, entity, shape, changed);
                    return;
                }
            }
        }
        let children = this.read().children.clone().unwrap();
        for node in children.iter() {
            let node_r = node.read();
            match shape.detect(node_r.inlet_boundary) {
                Relation::Disjoint => {}
                Relation::Overlap | Relation::Contain => match &node_r.parent {
                    Some(p) => {
                        Self::insert_omit_children_inner(p, entity, shape, changed);
                        return;
                    }
                    None => {
                        let mut this_w = this.write();
                        this_w.entities.insert(entity, shape);
                        changed.push((entity, Arc::clone(this)));
                        return;
                    }
                },
                Relation::Contained => {
                    drop(node_r);
                    Self::insert_inner(node, entity, shape, changed);
                    return;
                }
            }
        }
        warn!("{:?} out of QuadTree boundary", entity);
    }

    fn insert_omit_children_inner(
        this: &ArcNode<N, K>,
        entity: Entity,
        shape: Box<dyn DynCollision>,
        changed: &mut Vec<(Entity, ArcNode<N, K>)>,
    ) {
        let this_r = this.read();
        match shape.detect(this_r.inlet_boundary) {
            Relation::Overlap | Relation::Disjoint | Relation::Contain => {
                if let Some(p) = &this_r.parent {
                    let p = Arc::clone(p);
                    drop(this_r);
                    Self::insert_inner(&p, entity, shape, changed);
                    return;
                }
            }
            Relation::Contained => {}
        }
        drop(this_r);
        let mut this_w = this.write();
        this_w.entities.insert(entity, shape);
        changed.push((entity, Arc::clone(this)));
    }

    fn divide_inner(this: &ArcNode<N, K>, changed: &mut Vec<(Entity, ArcNode<N, K>)>) {
        let children = {
            let this_r = this.read();
            debug_assert!(
                this_r.children.is_none(),
                "should divide only if not divided yet"
            );
            let delta = this_r.inlet_boundary.size() / 2.;
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
            core::array::from_fn(|i| {
                Arc::new(RwLock::new(Node::new_with_parent(
                    Rect {
                        min: this_r.inlet_boundary.min + MIN[i] * delta,
                        max: this_r.inlet_boundary.min + MAX[i] * delta,
                    },
                    Arc::clone(this),
                )))
            })
        };
        let mut this_w = this.write();
        this_w.children = Some(children);
        let drain = this_w.entities.drain().collect::<Vec<_>>();
        drop(this_w);
        for (entity, shape) in drain.into_iter() {
            Self::insert_inner(this, entity, shape, changed);
        }
    }

    fn remove(&mut self, entity: Entity) {
        todo!()
    }

    fn query(&self, boundary: Rect) -> Vec<Entity> {
        todo!()
    }
}
