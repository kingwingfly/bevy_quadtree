//! Query

use crate::{collision::Relation, node::ArcNode, CollisionQuery};
use bevy::ecs::entity::EntityHashSet;

/// `Or` filter used in `QuadTree::query`
pub struct QOr<T>(core::marker::PhantomData<T>);
/// `Not` filter used in `QuadTree::query`
pub struct QNot<T>(core::marker::PhantomData<T>);

/// implemented for [`Disjoint`], [`Overlap`], [`Contain`], [`Contained`], [`QOr`], [`QNot`].
///
/// There is no `QAnd` because all the filters do not overlap, e.g. `QAnd<(Disjoint, Contain)>` is always empty.
#[allow(missing_docs)]
pub trait QRelation {
    fn filter<const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        boundary: &dyn CollisionQuery,
    ) -> EntityHashSet {
        let mut res = EntityHashSet::default();
        Self::filter_inner(node, boundary, &mut res);
        res
    }

    fn filter_inner<const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        boundary: &dyn CollisionQuery,
        res: &mut EntityHashSet,
    );

    fn all<const N: usize, const K: usize>(node: &ArcNode<N, K>, res: &mut EntityHashSet) {
        let node_r = node.read();
        res.extend(node_r.entities.keys().cloned());
        if let Some(children) = &node_r.children {
            for child in children.iter() {
                Self::all(child, res);
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
    fn filter_inner<const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        _: &dyn CollisionQuery,
        res: &mut EntityHashSet,
    ) {
        Self::all(node, res);
    }
}

impl QRelation for Disjoint {
    fn filter_inner<const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        boundary: &dyn CollisionQuery,
        res: &mut EntityHashSet,
    ) {
        let node_r = node.read();
        match boundary.query(&node_r.outlet_boundary) {
            Relation::Disjoint => All::all(node, res),
            Relation::Overlap | Relation::Contained => {
                for (entity, shape) in node_r.entities.iter() {
                    if boundary.query(shape.as_ref()) == Relation::Disjoint {
                        res.insert(*entity);
                    }
                }
                if let Some(children) = &node_r.children {
                    for child in children.iter() {
                        Self::filter_inner(child, boundary, res);
                    }
                }
            }
            Relation::Contain => {}
        }
    }
}
impl QRelation for Overlap {
    fn filter_inner<const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        boundary: &dyn CollisionQuery,
        res: &mut EntityHashSet,
    ) {
        let node_r = node.read();
        match boundary.query(&node_r.outlet_boundary) {
            Relation::Disjoint | Relation::Contain => {}
            Relation::Overlap | Relation::Contained => {
                for (entity, shape) in node_r.entities.iter() {
                    if boundary.query(shape.as_ref()) == Relation::Overlap {
                        res.insert(*entity);
                    }
                }
                if let Some(children) = &node_r.children {
                    for child in children.iter() {
                        Self::filter_inner(child, boundary, res);
                    }
                }
            }
        }
    }
}
impl QRelation for Contain {
    fn filter_inner<const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        boundary: &dyn CollisionQuery,
        res: &mut EntityHashSet,
    ) {
        let node_r = node.read();
        match boundary.query(&node_r.outlet_boundary) {
            Relation::Disjoint => {}
            Relation::Contain => All::all(node, res),
            Relation::Overlap | Relation::Contained => {
                for (entity, shape) in node_r.entities.iter() {
                    if boundary.query(shape.as_ref()) == Relation::Contain {
                        res.insert(*entity);
                    }
                }
                if let Some(children) = &node_r.children {
                    for child in children.iter() {
                        Self::filter_inner(child, boundary, res);
                    }
                }
            }
        }
    }
}
impl QRelation for Contained {
    fn filter_inner<const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        boundary: &dyn CollisionQuery,
        res: &mut EntityHashSet,
    ) {
        let node_r = node.read();
        match boundary.query(&node_r.outlet_boundary) {
            Relation::Disjoint | Relation::Contain | Relation::Overlap => {}
            Relation::Contained => {
                for (entity, shape) in node_r.entities.iter() {
                    if boundary.query(shape.as_ref()) == Relation::Contained {
                        res.insert(*entity);
                    }
                }
                if let Some(children) = &node_r.children {
                    for child in children.iter() {
                        Self::filter_inner(child, boundary, res);
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
            fn filter_inner<const N: usize, const K: usize>(
                node: &ArcNode<N, K>,
                boundary: &dyn CollisionQuery,
                res: &mut EntityHashSet,
            ) {
                $($t::filter_inner(node, boundary, res);)+
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
                fn filter_inner<const N: usize, const K: usize>(
                    node: &ArcNode<N, K>,
                    boundary: &dyn CollisionQuery,
                    res: &mut EntityHashSet,
                ) {
                    let mut tmp = EntityHashSet::default();
                    $r::filter_inner(node, boundary, &mut tmp);
                    All::all(node, res);
                    for entity in tmp.iter() {
                        res.remove(entity);
                    }
                }
            }
        )+
    };
}

impl_not_relation!(Contain, Contained, Overlap, Disjoint);
