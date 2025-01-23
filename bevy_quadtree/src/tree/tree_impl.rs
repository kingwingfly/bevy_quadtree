//! QuadTree inner implementation.

use crate::{
    collision::{DynCollision, Relation},
    CollisionRect,
};
use bevy_ecs::entity::{Entity, EntityHashMap};
use bevy_log::warn;
use bevy_math::{Rect, Vec2};
use core::fmt;
use parking_lot::RwLock;
use std::{
    alloc::{alloc, dealloc, Layout},
    any::type_name,
    ops::Index,
    sync::atomic::{AtomicBool, Ordering},
};

use super::query::QueryTree;

pub(crate) type NodeID = usize;

pub(crate) struct Tree<
    const N: usize,
    const D: usize,
    const W: usize,
    const H: usize,
    const K: usize,
> {
    nodes: *mut Node<K>,
}

unsafe impl<const N: usize, const D: usize, const W: usize, const H: usize, const K: usize> Send
    for Tree<N, D, W, H, K>
{
}

unsafe impl<const N: usize, const D: usize, const W: usize, const H: usize, const K: usize> Sync
    for Tree<N, D, W, H, K>
{
}

impl<const N: usize, const D: usize, const W: usize, const H: usize, const K: usize> fmt::Debug
    for Tree<N, D, W, H, K>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("total", &self.total(0))
            .finish()
    }
}

impl<const N: usize, const D: usize, const W: usize, const H: usize, const K: usize> Index<usize>
    for Tree<N, W, H, D, K>
{
    type Output = Node<K>;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.nodes.add(index) }
    }
}

impl<const N: usize, const D: usize, const W: usize, const H: usize, const K: usize> Drop
    for Tree<N, D, W, H, K>
{
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::array::<Node<K>>(Self::MAX_LEN).expect("`D` is too large");
            dealloc(self.nodes as *mut u8, layout);
        }
    }
}

impl<const N: usize, const D: usize, const W: usize, const H: usize, const K: usize>
    Tree<N, D, W, H, K>
{
    const MAX_LEN: usize = (4usize.pow(D as u32) - 1) / 3;

    pub(crate) fn new() -> Self {
        unsafe {
            let layout = Layout::array::<Node<K>>(Self::MAX_LEN).expect("`D` is too large");
            let nodes = alloc(layout) as *mut Node<K>;
            nodes.write(Node::root(Rect::from_center_size(
                Vec2::ZERO,
                Vec2::new(W as f32, H as f32),
            )));
            let mut i = 1;
            while i < Self::MAX_LEN {
                let p = (i - 1) >> 2;
                for c in (*nodes.add(p)).birth() {
                    nodes.add(i).write(c);
                    i += 1;
                }
            }
            Self { nodes }
        }
    }

    pub(crate) fn total(&self, id: NodeID) -> usize {
        self[id].len()
            + if self[id].is_leaf() {
                0
            } else {
                ((id << 2) + 1..=(id << 2) + 4)
                    .map(|i| self.total(i))
                    .sum::<usize>()
            }
    }

    pub(crate) fn update(
        &self,
        id: usize,
        entity: Entity,
        shape: Box<dyn DynCollision>,
        changed: &mut Vec<Change>,
    ) {
        match shape.detect(&self[id].outlet_boundary) {
            Relation::Contain => {
                self.remove(id, &entity);
                if id > 0 {
                    let p = (id - 1) >> 2;
                    if p > 0 {
                        self.insert((p - 1) >> 2, entity, shape, changed, vec![p]);
                        return;
                    }
                }
                warn!("{:?} out of QuadTree boundary", entity);
                self.merge_up(id);
                changed.push(Change::Leave(entity));
            }
            Relation::Disjoint | Relation::Overlap => {
                self.remove(id, &entity);
                if id > 0 {
                    self.insert((id - 1) >> 2, entity, shape, changed, vec![id]);
                }
                self.merge_up(id);
            }
            Relation::Contained => match shape.detect(&self[id].inlet_boundary) {
                Relation::Disjoint | Relation::Overlap | Relation::Contain => {}
                Relation::Contained => {
                    // it may no longer overlap with multiple children
                    self.remove(id, &entity);
                    self.insert(id, entity, shape, changed, vec![]);
                    // never need to merge up
                }
            },
        }
    }

    pub(crate) fn insert(
        &self,
        mut id: usize,
        entity: Entity,
        shape: Box<dyn DynCollision>,
        changed: &mut Vec<Change>,
        mut omit: Vec<NodeID>,
    ) {
        'a: loop {
            if self[id].is_leaf() {
                if self[id].len() < N || (id << 2) + 1 >= Self::MAX_LEN {
                    self[id].insert(entity, shape);
                    changed.push(Change::Move(entity, id));
                    return;
                } else {
                    self.divide(id, changed);
                }
            }
            let children = (id << 2) + 1..=(id << 2) + 4;
            let mut disjoint = false;
            for i in children.clone() {
                if omit.contains(&i) {
                    continue;
                }
                match shape.detect(&self[i].inlet_boundary) {
                    Relation::Disjoint => disjoint = true,
                    Relation::Overlap | Relation::Contain => {
                        omit.extend(children);
                        continue 'a;
                    }
                    Relation::Contained => {
                        id = i;
                        continue 'a;
                    }
                }
            }
            match id {
                0 if disjoint => {
                    warn!("{:?} out of QuadTree boundary", entity);
                    changed.push(Change::Leave(entity));
                    return;
                }
                0 => self[id].insert(entity, shape),
                _ if disjoint => {
                    omit = vec![id];
                    id = (id - 1) >> 2;
                    continue 'a;
                }
                _ => self[id].insert(entity, shape),
            }
            changed.push(Change::Move(entity, id));
            return;
        }
    }

    /// Remove the entity from the node.
    pub(crate) fn remove(&self, id: NodeID, entity: &Entity) {
        self[id].entities.write().remove(entity);
    }

    pub(crate) fn merge_up(&self, id: NodeID) {
        if !self[id].is_leaf() {
            return;
        }
        let mut x = vec![id];
        while let Some(id) = x.pop() {
            if id > 0 {
                let p = (id - 1) >> 2;
                let mut children = (p << 2) + 1..=(p << 2) + 4;
                if children.all(|id| self[id].is_leaf() && self[id].len() == 0) {
                    self[p].leaf.store(true, Ordering::Release);
                    x.push(p);
                }
            }
        }
    }

    fn divide(&self, id: NodeID, changed: &mut Vec<Change>) {
        self[id].leaf.store(false, Ordering::Release);
        let es = self[id].entities.write().drain().collect::<Vec<_>>();
        for (entity, shape) in es {
            self.insert(id, entity, shape, changed, vec![]);
        }
    }

    pub(crate) fn query_tree(&self) -> QueryTree<D, K> {
        QueryTree(self.nodes)
    }
}

pub(crate) struct Node<const K: usize> {
    pub(crate) entities: RwLock<EntityHashMap<Box<dyn DynCollision>>>,
    pub(crate) inlet_boundary: CollisionRect,
    pub(crate) outlet_boundary: CollisionRect,
    leaf: AtomicBool,
}

impl<const K: usize> fmt::Debug for Node<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("entities", &self.len())
            .finish()
    }
}

impl<const K: usize> Node<K> {
    fn root(boundary: Rect) -> Self {
        Self {
            entities: RwLock::new(EntityHashMap::default()),
            inlet_boundary: boundary.into(),
            outlet_boundary: boundary.into(),
            leaf: AtomicBool::new(true),
        }
    }

    fn new(boundary: Rect) -> Self {
        Self {
            entities: RwLock::new(EntityHashMap::default()),
            inlet_boundary: boundary.into(),
            outlet_boundary: Rect::from_center_size(
                boundary.center(),
                boundary.size() * const { K as f32 / 10. },
            )
            .into(),
            leaf: AtomicBool::new(true),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.entities.read().len()
    }

    pub(crate) fn is_leaf(&self) -> bool {
        self.leaf.load(Ordering::Acquire)
    }

    fn birth(&self) -> [Self; 4] {
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
        let delta = self.inlet_boundary.size() / 2.;
        let min = self.inlet_boundary.min();
        core::array::from_fn(|i| {
            let boundary = Rect {
                min: min + MIN[i] * delta,
                max: min + MAX[i] * delta,
            };
            Self::new(boundary)
        })
    }

    fn insert(&self, entity: Entity, shape: Box<dyn DynCollision>) {
        self.entities.write().insert(entity, shape);
    }
}

pub(crate) enum Change {
    Move(Entity, NodeID),
    Leave(Entity),
}
