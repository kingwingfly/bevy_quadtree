#![allow(private_interfaces)]
//! Query

use std::ops::Index;

use super::{quad_tree::NodeID, tree_impl::Node};
use crate::collision::{CollisionQuery, Relation};
use bevy_ecs::entity::EntityHashSet;

/// A wrapper of root node of the quadtree in order to decrease the number of type parameters.
pub(crate) struct QueryTree<const D: usize, const K: usize>(pub(crate) *const Node<K>);

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
pub trait QRelation<const D: usize> {
    const MAX_LEN: usize = (4usize.pow(D as u32) - 1) / 3;

    // filter the entities that satisfy the relation.
    //
    // Methods related to `QueryTree` are all private, if you need them, please open an issue.
    fn filter<const K: usize>(qt: &QueryTree<D, K>, boundary: &dyn CollisionQuery)
        -> EntityHashSet;

    fn all<const K: usize>(qt: &QueryTree<D, K>, id: NodeID) -> EntityHashSet {
        let mut res = EntityHashSet::default();
        let mut x = vec![id];
        while let Some(id) = x.pop() {
            res.extend(qt[id].entities.read().keys().cloned());
            if !qt[id].is_leaf() && (id << 2) + 4 < <Self as QRelation<D>>::MAX_LEN {
                for i in (id << 2) + 1..=(id << 2) + 4 {
                    x.push(i);
                }
            }
        }
        res
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

impl<const D: usize> QRelation<D> for All {
    fn filter<const K: usize>(qt: &QueryTree<D, K>, _: &dyn CollisionQuery) -> EntityHashSet {
        Self::all(qt, 0)
    }
}

impl<const D: usize> QRelation<D> for Disjoint {
    fn filter<const K: usize>(
        qt: &QueryTree<D, K>,
        boundary: &dyn CollisionQuery,
    ) -> EntityHashSet {
        let mut res = EntityHashSet::default();
        let mut x = vec![0];
        while let Some(id) = x.pop() {
            match boundary.query(&qt[id].outlet_boundary) {
                Relation::Disjoint => res.extend(All::all(qt, id)),
                Relation::Overlap | Relation::Contained => {
                    for (entity, shape) in qt[id].entities.read().iter() {
                        if boundary.query(shape.as_ref()) == Relation::Disjoint {
                            res.insert(*entity);
                        }
                    }
                    if !qt[id].is_leaf() && (id << 2) + 4 < <Self as QRelation<D>>::MAX_LEN {
                        for i in (id << 2) + 1..=(id << 2) + 4 {
                            x.push(i);
                        }
                    }
                }
                Relation::Contain => {}
            }
        }
        res
    }
}
impl<const D: usize> QRelation<D> for Overlap {
    fn filter<const K: usize>(
        qt: &QueryTree<D, K>,
        boundary: &dyn CollisionQuery,
    ) -> EntityHashSet {
        let mut res = EntityHashSet::default();
        let mut x = vec![0];
        while let Some(id) = x.pop() {
            match boundary.query(&qt[id].outlet_boundary) {
                Relation::Disjoint | Relation::Contain => {}
                Relation::Overlap | Relation::Contained => {
                    for (entity, shape) in qt[id].entities.read().iter() {
                        if boundary.query(shape.as_ref()) == Relation::Overlap {
                            res.insert(*entity);
                        }
                    }
                    if !qt[id].is_leaf() && (id << 2) + 4 < <Self as QRelation<D>>::MAX_LEN {
                        for i in (id << 2) + 1..=(id << 2) + 4 {
                            x.push(i);
                        }
                    }
                }
            }
        }
        res
    }
}
impl<const D: usize> QRelation<D> for Contain {
    fn filter<const K: usize>(
        qt: &QueryTree<D, K>,
        boundary: &dyn CollisionQuery,
    ) -> EntityHashSet {
        let mut res = EntityHashSet::default();
        let mut x = vec![0];
        while let Some(id) = x.pop() {
            match boundary.query(&qt[id].outlet_boundary) {
                Relation::Disjoint => {}
                Relation::Contain => res.extend(All::all(qt, id)),
                Relation::Overlap | Relation::Contained => {
                    for (entity, shape) in qt[id].entities.read().iter() {
                        if boundary.query(shape.as_ref()) == Relation::Contain {
                            res.insert(*entity);
                        }
                    }
                    if !qt[id].is_leaf() && (id << 2) + 4 < <Self as QRelation<D>>::MAX_LEN {
                        for i in (id << 2) + 1..=(id << 2) + 4 {
                            x.push(i);
                        }
                    }
                }
            }
        }
        res
    }
}
impl<const D: usize> QRelation<D> for Contained {
    fn filter<const K: usize>(
        qt: &QueryTree<D, K>,
        boundary: &dyn CollisionQuery,
    ) -> EntityHashSet {
        let mut res = EntityHashSet::default();
        let mut x = vec![0];
        while let Some(id) = x.pop() {
            match boundary.query(&qt[id].outlet_boundary) {
                Relation::Disjoint | Relation::Contain | Relation::Overlap => {}
                Relation::Contained => {
                    for (entity, shape) in qt[id].entities.read().iter() {
                        if boundary.query(shape.as_ref()) == Relation::Contained {
                            res.insert(*entity);
                        }
                    }
                    if !qt[id].is_leaf() && (id << 2) + 4 < <Self as QRelation<D>>::MAX_LEN {
                        for i in (id << 2) + 1..=(id << 2) + 4 {
                            x.push(i);
                        }
                    }
                }
            }
        }
        res
    }
}

macro_rules! impl_or_relation {
    ($($t: ident),+) => {
        impl<const D: usize, $($t),+> QRelation<D> for QOr<($($t),+,)>
        where $($t: QRelation<D>),+
        {
            fn filter<const K: usize>(
                qt: &QueryTree<D, K>,
                boundary: &dyn CollisionQuery,
            ) -> EntityHashSet {
                let mut res = EntityHashSet::default();
                $(res.extend($t::filter(qt, boundary));)+
                res
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
            impl<const D: usize> QRelation<D> for QNot<$r> {
                fn filter<const K: usize>(
                    qt: &QueryTree<D, K>,
                    boundary: &dyn CollisionQuery,
                ) -> EntityHashSet {
                    let tmp = $r::filter(qt, boundary);
                    let mut res = All::all(qt, 0);
                    for entity in tmp.iter() {
                        res.remove(entity);
                    }
                    res
                }
            }
        )+
    };
}

impl_not_relation!(Contain, Contained, Overlap, Disjoint);
