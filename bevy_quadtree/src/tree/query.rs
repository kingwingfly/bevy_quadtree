//! Query

use std::ops::Index;

use super::{quad_tree::NodeID, tree_impl::Node};
use crate::{collision::Relation, CollisionQuery};
use bevy_ecs::entity::EntityHashSet;

pub struct QueryTree<const D: usize, const K: usize>(pub(crate) *const Node<K>);

impl<const D: usize, const K: usize> Index<usize> for QueryTree<D, K> {
    type Output = Node<K>;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.0.add(index) }
    }
}

/// `Or` filter used in `QuadTree::query`
pub struct QOr<T>(core::marker::PhantomData<T>);
/// `Not` filter used in `QuadTree::query`
pub struct QNot<T>(core::marker::PhantomData<T>);

/// implemented for [`Disjoint`], [`Overlap`], [`Contain`], [`Contained`], [`QOr`], [`QNot`].
///
/// There is no `QAnd` because all the filters do not overlap, e.g. `QAnd<(Disjoint, Contain)>` is always empty.
#[allow(missing_docs)]
pub trait QRelation {
    fn filter<const D: usize, const K: usize>(
        qt: &QueryTree<D, K>,
        boundary: &dyn CollisionQuery,
    ) -> EntityHashSet {
        let mut res = EntityHashSet::default();
        Self::filter_inner(qt, 0, boundary, &mut res);
        res
    }

    fn filter_inner<const D: usize, const K: usize>(
        qt: &QueryTree<D, K>,
        id: NodeID,
        boundary: &dyn CollisionQuery,
        res: &mut EntityHashSet,
    );

    fn all<const D: usize, const K: usize>(
        qt: &QueryTree<D, K>,
        id: NodeID,
        res: &mut EntityHashSet,
    ) {
        let mut x = vec![id];
        while let Some(id) = x.pop() {
            res.extend(qt[id].entities.read().keys().cloned());
            for i in (id << 2) + 1..=(id << 2) + 4 {
                if !qt[i].is_leaf() {
                    x.push(i);
                } else {
                    res.extend(qt[i].entities.read().keys().cloned());
                }
            }
        }
    }
}

/// boundary disjoints shape
pub struct Disjoint;
/// boundary overlaps shape, including ExternallyTangent, InternallyTangent
pub struct Overlap;
/// boundary contains shape
pub struct Contain;
/// boundary is contained by shape
pub struct Contained;
/// all
pub struct All;

impl QRelation for All {
    fn filter_inner<const D: usize, const K: usize>(
        qt: &QueryTree<D, K>,
        _: NodeID,
        _: &dyn CollisionQuery,
        res: &mut EntityHashSet,
    ) {
        Self::all(qt, 0, res);
    }
}

impl QRelation for Disjoint {
    fn filter_inner<const D: usize, const K: usize>(
        qt: &QueryTree<D, K>,
        id: NodeID,
        boundary: &dyn CollisionQuery,
        res: &mut EntityHashSet,
    ) {
        match boundary.query(&qt[id].outlet_boundary) {
            Relation::Disjoint => All::all(qt, id, res),
            Relation::Overlap | Relation::Contained => {
                for (entity, shape) in qt[id].entities.read().iter() {
                    if boundary.query(shape.as_ref()) == Relation::Disjoint {
                        res.insert(*entity);
                    }
                }
                if !qt[id].is_leaf() {
                    for i in (id << 2) + 1..=(id << 2) + 4 {
                        Self::filter_inner(qt, i, boundary, res);
                    }
                }
            }
            Relation::Contain => {}
        }
    }
}
impl QRelation for Overlap {
    fn filter_inner<const D: usize, const K: usize>(
        qt: &QueryTree<D, K>,
        id: NodeID,
        boundary: &dyn CollisionQuery,
        res: &mut EntityHashSet,
    ) {
        match boundary.query(&qt[id].outlet_boundary) {
            Relation::Disjoint | Relation::Contain => {}
            Relation::Overlap | Relation::Contained => {
                for (entity, shape) in qt[id].entities.read().iter() {
                    if boundary.query(shape.as_ref()) == Relation::Overlap {
                        res.insert(*entity);
                    }
                }
                if !qt[id].is_leaf() {
                    for i in (id << 2) + 1..=(id << 2) + 4 {
                        Self::filter_inner(qt, i, boundary, res);
                    }
                }
            }
        }
    }
}
impl QRelation for Contain {
    fn filter_inner<const D: usize, const K: usize>(
        qt: &QueryTree<D, K>,
        id: NodeID,
        boundary: &dyn CollisionQuery,
        res: &mut EntityHashSet,
    ) {
        match boundary.query(&qt[id].outlet_boundary) {
            Relation::Disjoint => {}
            Relation::Contain => All::all(qt, id, res),
            Relation::Overlap | Relation::Contained => {
                for (entity, shape) in qt[id].entities.read().iter() {
                    if boundary.query(shape.as_ref()) == Relation::Contain {
                        res.insert(*entity);
                    }
                }
                if !qt[id].is_leaf() {
                    for i in (id << 2) + 1..=(id << 2) + 4 {
                        Self::filter_inner(qt, i, boundary, res);
                    }
                }
            }
        }
    }
}
impl QRelation for Contained {
    fn filter_inner<const D: usize, const K: usize>(
        qt: &QueryTree<D, K>,
        id: NodeID,
        boundary: &dyn CollisionQuery,
        res: &mut EntityHashSet,
    ) {
        match boundary.query(&qt[id].outlet_boundary) {
            Relation::Disjoint | Relation::Contain | Relation::Overlap => {}
            Relation::Contained => {
                for (entity, shape) in qt[id].entities.read().iter() {
                    if boundary.query(shape.as_ref()) == Relation::Contained {
                        res.insert(*entity);
                    }
                }
                if !qt[id].is_leaf() {
                    for i in (id << 2) + 1..=(id << 2) + 4 {
                        Self::filter_inner(qt, i, boundary, res);
                    }
                }
            }
        }
    }
}

macro_rules! impl_or_relation {
    ($($t: ident),+) => {
        impl<$($t),+> QRelation for QOr<($($t),+,)>
        where $($t: QRelation),+
        {
            fn filter_inner<const D: usize, const K: usize>(
                qt: &QueryTree<D, K>,
                id: NodeID,
                boundary: &dyn CollisionQuery,
                res: &mut EntityHashSet,
            ) {
                $($t::filter_inner(qt, id, boundary, res);)+
            }
        }
    };
}

impl_or_relation!(R0);
impl_or_relation!(R0, R1);
impl_or_relation!(R0, R1, R2);
impl_or_relation!(R0, R1, R2, R3);

macro_rules! impl_not_relation {
    ($($r: ident), +) => {
        $(
            impl QRelation for QNot<$r> {
                fn filter_inner<const D: usize, const K: usize>(
                    qt: &QueryTree<D, K>,
                    id: NodeID,
                    boundary: &dyn CollisionQuery,
                    res: &mut EntityHashSet,
                ) {
                    let mut tmp = EntityHashSet::default();
                    $r::filter_inner(qt, id, boundary, &mut tmp);
                    All::all(qt, 0, res);
                    for entity in tmp.iter() {
                        res.remove(entity);
                    }
                }
            }
        )+
    };
}

impl_not_relation!(Contain, Contained, Overlap, Disjoint);
