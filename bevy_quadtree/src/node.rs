use crate::{CollisionRect, Disassemble, DynCollision, QRelation, Relation};
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
    inlet_boundary: CollisionRect,
    outlet_boundary: CollisionRect,
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
                    this.write().entities.remove(&entity);
                    Self::insert_inner(&p, entity, shape, changed, &mut UNLESS_PARENT.to_vec());
                }
            }
            Relation::Disjoint => {
                let quadrant = this_r.quadrant;
                drop(this_r);
                this.write().entities.remove(&entity);
                if let Some(p) = &this.read().parent {
                    Self::insert_inner(p, entity, shape, changed, &mut vec![quadrant]);
                }
            }
            Relation::Overlap | Relation::Contained => match shape.detect(&this_r.inlet_boundary) {
                Relation::Disjoint | Relation::Overlap | Relation::Contain => {}
                Relation::Contained => {
                    drop(this_r);
                    this.write().entities.remove(&entity);
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
        if ALL_CHILDREN.iter().any(|p| !omit.contains(p)) && !omit.contains(&Pos::C) {
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
        if this_r.len() != 0 && this_r.total() != 0 {
            return;
        }
        if let Some(p) = this_r.parent.as_ref() {
            let mut p_w = p.write();
            if p_w
                .children
                .as_ref()
                .unwrap()
                .iter()
                .all(|c| c.read().total() == 0)
            {
                p_w.children = None;
                drop(p_w);
                Self::merge_up(p);
            }
        }
    }

    pub(crate) fn query<S>(this: &ArcNode<N, K>, boundary: &S, relation: QRelation) -> Vec<Entity>
    where
        S: Disassemble,
    {
        let mut res = vec![];
        Node::query_inner(this, boundary, relation, &mut res);
        res
    }

    fn query_inner<S>(
        this: &ArcNode<N, K>,
        boundary: &S,
        relation: QRelation,
        res: &mut Vec<Entity>,
    ) where
        S: Disassemble,
    {
        match relation {
            QRelation::Disjoint => Node::query_disjoint(this, boundary, res),
            QRelation::Overlap => Node::query_overlap(this, boundary, res),
            QRelation::Contain => Node::query_contain(this, boundary, res),
            QRelation::Contained => Node::query_contained(this, boundary, res),
            QRelation::OverlapOrContain => Node::query_overlap_or_contain(this, boundary, res),
        }
    }

    fn query_all(this: &ArcNode<N, K>, res: &mut Vec<Entity>) {
        let this_r = this.read();
        res.extend(this_r.entities.keys().cloned());
        if let Some(children) = &this_r.children {
            for child in children.iter() {
                Self::query_all(child, res);
            }
        }
    }

    fn query_disjoint<S>(this: &ArcNode<N, K>, boundary: &S, res: &mut Vec<Entity>)
    where
        S: Disassemble,
    {
        let this_r = this.read();
        match boundary.detect(&this_r.outlet_boundary) {
            Relation::Disjoint => {
                Self::query_all(this, res);
            }
            Relation::Overlap | Relation::Contained => {
                for (entity, shape) in this_r.entities.iter() {
                    if boundary.detect(shape.as_ref()) == Relation::Disjoint {
                        res.push(*entity);
                    }
                }
                if let Some(children) = &this_r.children {
                    for child in children.iter() {
                        Self::query_disjoint(child, boundary, res);
                    }
                }
            }
            Relation::Contain => {}
        }
    }

    fn query_overlap<S>(this: &ArcNode<N, K>, boundary: &S, res: &mut Vec<Entity>)
    where
        S: Disassemble,
    {
        let this_r = this.read();
        match boundary.detect(&this_r.outlet_boundary) {
            Relation::Disjoint => {}
            Relation::Overlap | Relation::Contained | Relation::Contain => {
                for (entity, shape) in this_r.entities.iter() {
                    if boundary.detect(shape.as_ref()) == Relation::Overlap {
                        res.push(*entity);
                    }
                }
                if let Some(children) = &this_r.children {
                    for child in children.iter() {
                        Self::query_overlap(child, boundary, res);
                    }
                }
            }
        }
    }

    fn query_contain<S>(this: &ArcNode<N, K>, boundary: &S, res: &mut Vec<Entity>)
    where
        S: Disassemble,
    {
        let this_r = this.read();
        match boundary.detect(&this_r.outlet_boundary) {
            Relation::Disjoint => {}
            Relation::Overlap | Relation::Contain | Relation::Contained => {
                for (entity, shape) in this_r.entities.iter() {
                    if boundary.detect(shape.as_ref()) == Relation::Contain {
                        res.push(*entity);
                    }
                }
                if let Some(children) = &this_r.children {
                    for child in children.iter() {
                        Self::query_contain(child, boundary, res);
                    }
                }
            }
        }
    }

    fn query_contained<S>(this: &ArcNode<N, K>, boundary: &S, res: &mut Vec<Entity>)
    where
        S: Disassemble,
    {
        let this_r = this.read();
        match boundary.detect(&this_r.outlet_boundary) {
            Relation::Disjoint | Relation::Contain => {}
            Relation::Overlap | Relation::Contained => {
                for (entity, shape) in this_r.entities.iter() {
                    if boundary.detect(shape.as_ref()) == Relation::Contained {
                        res.push(*entity);
                    }
                }
                if let Some(children) = &this_r.children {
                    for child in children.iter() {
                        Self::query_contained(child, boundary, res);
                    }
                }
            }
        }
    }

    fn query_overlap_or_contain<S>(this: &ArcNode<N, K>, boundary: &S, res: &mut Vec<Entity>)
    where
        S: Disassemble,
    {
        let this_r = this.read();
        match boundary.detect(&this_r.outlet_boundary) {
            Relation::Disjoint => {}
            Relation::Overlap | Relation::Contain | Relation::Contained => {
                for (entity, shape) in this_r.entities.iter() {
                    if matches!(
                        boundary.detect(shape.as_ref()),
                        Relation::Contain | Relation::Overlap
                    ) {
                        res.push(*entity);
                    }
                }
                if let Some(children) = &this_r.children {
                    for child in children.iter() {
                        Self::query_overlap_or_contain(child, boundary, res);
                    }
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
