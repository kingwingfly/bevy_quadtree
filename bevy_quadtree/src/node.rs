//! Node in QuadTree

use crate::{
    collision::{DynCollision, Relation},
    CollisionRect,
};
use bevy::{
    ecs::entity::EntityHashMap,
    log::warn,
    math::{Rect, Vec2},
    prelude::Entity,
};
use core::fmt;
use parking_lot::RwLock;
use std::sync::{Arc, RwLockReadGuard};

/// type alias for `Arc<RwLock<Node<N, K>>>`
pub type ArcNode<const N: usize, const K: usize> = Arc<RwLock<Node<N, K>>>;

/// Node in `QuadTree`
pub struct Node<const N: usize, const K: usize = 10> {
    pub(crate) entities: EntityHashMap<Box<dyn DynCollision>>,
    pub(crate) inlet_boundary: CollisionRect,
    pub(crate) outlet_boundary: CollisionRect,
    parent: Option<ArcNode<N, K>>,
    pub(crate) children: Option<[ArcNode<N, K>; 4]>,
    quadrant: Pos,
}

impl<const N: usize, const K: usize> fmt::Debug for Node<N, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("entities", &self.total())
            .field("quadrant", &self.quadrant)
            .field("parent", &self.parent.is_some())
            .field("inlet_boundary", &self.inlet_boundary)
            .field("outlet_boundary", &self.outlet_boundary)
            .field("children", &self.children)
            .finish()
    }
}

impl<const N: usize, const K: usize> Node<N, K> {
    #[allow(unused)]
    fn from(boundary: Rect, quadrant: Pos) -> Self {
        Self {
            entities: EntityHashMap::default(),
            inlet_boundary: boundary.into(),
            outlet_boundary: Rect::from_center_size(
                boundary.center(),
                boundary.size() * const { K as f32 / 10. },
            )
            .into(),
            parent: None,
            children: None,
            quadrant,
        }
    }

    pub(crate) fn root(boundary: Rect) -> Self {
        Self {
            entities: EntityHashMap::default(),
            inlet_boundary: boundary.into(),
            outlet_boundary: boundary.into(),
            parent: None,
            children: None,
            quadrant: Pos::O,
        }
    }

    fn new_with_parent(boundary: Rect, parent: ArcNode<N, K>, quadrant: Pos) -> Self {
        Self {
            entities: EntityHashMap::default(),
            inlet_boundary: boundary.into(),
            outlet_boundary: Rect::from_center_size(
                boundary.center(),
                boundary.size() * const { K as f32 / 10. },
            )
            .into(),
            parent: Some(parent),
            children: None,
            quadrant,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.entities.len()
    }

    pub(crate) fn total(&self) -> usize {
        self.entities.len()
            + self
                .children
                .as_ref()
                .map_or(0, |c| c.iter().map(|n| n.read().total()).sum())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
            && self
                .children
                .as_ref()
                .map_or(true, |c| c.iter().all(|n| n.read().is_empty()))
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
        Self::insert_inner(this, entity, shape, &mut changed, &mut vec![]);
        changed
    }

    fn update_inner(
        this: &ArcNode<N, K>,
        entity: Entity,
        shape: Box<dyn DynCollision>,
        changed: &mut Vec<(Entity, ArcNode<N, K>)>,
    ) {
        let this_r = this.read();
        match shape.detect(&this_r.outlet_boundary) {
            Relation::Contain => {
                if let Some(p) = &this_r.parent {
                    let p = Arc::clone(p);
                    drop(this_r);
                    Node::remove(this, &entity);
                    Self::insert_inner(&p, entity, shape, changed, &mut UNLESS_PARENT.to_vec());
                }
            }
            Relation::Disjoint | Relation::Overlap => {
                let quadrant = this_r.quadrant;
                drop(this_r);
                Node::remove(this, &entity);
                if let Some(p) = &this.read().parent {
                    Self::insert_inner(p, entity, shape, changed, &mut vec![quadrant]);
                }
            }
            Relation::Contained => match shape.detect(&this_r.inlet_boundary) {
                Relation::Disjoint | Relation::Overlap | Relation::Contain => {}
                Relation::Contained => {
                    drop(this_r);
                    Node::remove(this, &entity);
                    Self::insert_inner(this, entity, shape, changed, &mut vec![]);
                }
            },
        }
    }

    fn insert_inner(
        this: &ArcNode<N, K>,
        entity: Entity,
        shape: Box<dyn DynCollision>,
        changed: &mut Vec<(Entity, ArcNode<N, K>)>,
        omit: &mut Vec<Pos>,
    ) {
        if !omit.contains(&Pos::C) && ALL_CHILDREN.iter().any(|p| !omit.contains(p)) {
            {
                let this_r = this.read();
                if this_r.children.is_none() {
                    if this_r.total() >= N {
                        drop(this_r);
                        Self::divide_inner(this, changed);
                    } else {
                        drop(this_r);
                        omit.extend(ALL_CHILDREN);
                        Self::insert_inner(this, entity, shape, changed, omit);
                        return;
                    }
                }
            }
            let this_r = this.read();
            let children = this_r.children.as_ref().unwrap();
            for (i, node) in children.iter().enumerate() {
                if omit.contains(&i.into()) {
                    continue;
                }
                let node_r = node.read();
                match shape.detect(&node_r.inlet_boundary) {
                    Relation::Disjoint => {}
                    Relation::Overlap | Relation::Contain => {
                        drop(node_r);
                        drop(this_r);
                        omit.extend(ALL_CHILDREN);
                        Self::insert_inner(this, entity, shape, changed, omit);
                        return;
                    }
                    Relation::Contained => {
                        drop(node_r);
                        Self::insert_inner(node, entity, shape, changed, &mut vec![Pos::P]);
                        return;
                    }
                }
            }
            let this_r = this.read();
            let quadrant = this_r.quadrant;
            let p = this_r.parent.clone();
            drop(this_r);
            match p {
                Some(p) => Self::insert_inner(&p, entity, shape, changed, &mut vec![quadrant]),
                None => warn!("{:?} out of QuadTree boundary", entity),
            }
        } else {
            if !omit.contains(&Pos::P) {
                let this_r = this.read();
                if let Some(p) = this_r.parent.clone() {
                    match shape.detect(&this_r.inlet_boundary) {
                        Relation::Overlap | Relation::Disjoint => {
                            let quadrant = this_r.quadrant;
                            drop(this_r);
                            Self::insert_inner(&p, entity, shape, changed, &mut vec![quadrant]);
                            return;
                        }
                        Relation::Contain => {
                            drop(this_r);
                            Self::insert_inner(
                                &p,
                                entity,
                                shape,
                                changed,
                                &mut UNLESS_PARENT.to_vec(),
                            );
                            return;
                        }
                        Relation::Contained => {}
                    }
                }
            }
            let mut this_w = this.write();
            this_w.entities.insert(entity, shape);
            changed.push((entity, Arc::clone(this)));
        }
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
                    i.into(),
                )))
            })
        };
        let mut this_w = this.write();
        this_w.children = Some(children);
        let drain = this_w.entities.drain().collect::<Vec<_>>();
        drop(this_w);
        for (entity, shape) in drain.into_iter() {
            Self::insert_inner(this, entity, shape, changed, &mut vec![Pos::P]);
        }
    }

    pub(crate) fn remove(this: &ArcNode<N, K>, entity: &Entity) {
        {
            let mut this_w = this.write();
            this_w.entities.remove(entity);
        }
        Self::merge_up(this);
    }

    fn merge_up(this: &ArcNode<N, K>) {
        let this_r = this.read();
        if !this_r.is_empty() {
            return;
        }
        if let Some(p) = this_r.parent.as_ref() {
            let mut p_w = p.write();
            if let Some(children) = p_w.children.as_ref() {
                if core::array::from_fn::<_, 4, _>(|i| children[i].read())
                    .iter()
                    .all(|c| c.is_empty())
                {
                    p_w.children = None;
                    drop(p_w);
                    Self::merge_up(p);
                }
            }
        }
    }
}

/// Position of the node in the QuadTree
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pos {
    I = 0,
    II,
    III,
    IV,
    /// Current
    C,
    /// Parent
    P,
    /// Root
    O,
}

const ALL_CHILDREN: [Pos; 4] = [Pos::I, Pos::II, Pos::III, Pos::IV];
const UNLESS_PARENT: [Pos; 5] = [Pos::I, Pos::II, Pos::III, Pos::IV, Pos::C];

impl From<usize> for Pos {
    fn from(pos: usize) -> Self {
        match pos {
            0 => Self::I,
            1 => Self::II,
            2 => Self::III,
            3 => Self::IV,
            _ => unimplemented!(),
        }
    }
}
