//! Query

use crate::{collision::Relation, node::ArcNode, CollisionQuery};
use bevy::ecs::entity::EntityHashSet;

/// `Or` filter used in `QuadTree::query`
pub struct QOr<T>(core::marker::PhantomData<T>);
/// `Not` filter used in `QuadTree::query`
pub struct QNot<T>(core::marker::PhantomData<T>);

/// implemented for [`Contain`], [`Contained`], [`Overlap`], [`Disjoint`], [`QOr`], [`QNot`] and tuple of them.
#[allow(missing_docs)]
pub trait QRelation {
    fn filter<S, const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        boundary: &S,
    ) -> EntityHashSet
    where
        S: CollisionQuery,
    {
        let mut res = EntityHashSet::default();
        Self::filter_inner(node, boundary, &mut res);
        res
    }

    fn filter_inner<S, const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        boundary: &S,
        res: &mut EntityHashSet,
    ) where
        S: CollisionQuery;

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
    fn filter_inner<S, const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        _: &S,
        res: &mut EntityHashSet,
    ) where
        S: CollisionQuery,
    {
        Self::all(node, res);
    }
}

impl QRelation for Disjoint {
    fn filter_inner<S, const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        boundary: &S,
        res: &mut EntityHashSet,
    ) where
        S: CollisionQuery,
    {
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
    fn filter_inner<S, const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        boundary: &S,
        res: &mut EntityHashSet,
    ) where
        S: CollisionQuery,
    {
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
    fn filter_inner<S, const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        boundary: &S,
        res: &mut EntityHashSet,
    ) where
        S: CollisionQuery,
    {
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
    fn filter_inner<S, const N: usize, const K: usize>(
        node: &ArcNode<N, K>,
        boundary: &S,
        res: &mut EntityHashSet,
    ) where
        S: CollisionQuery,
    {
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
            fn filter_inner<S, const N: usize, const K: usize>(
                node: &ArcNode<N, K>,
                boundary: &S,
                res: &mut EntityHashSet,
            ) where
                S: CollisionQuery,
            {
                $($t::filter_inner(node, boundary, res);)+
            }
        }
    };
}

impl_or_relation!(R1);
impl_or_relation!(R1, R2);
impl_or_relation!(R1, R2, R3);
impl_or_relation!(R1, R2, R3, R4);
impl_or_relation!(R1, R2, R3, R4, R5);
impl_or_relation!(R1, R2, R3, R4, R5, R6);
impl_or_relation!(R1, R2, R3, R4, R5, R6, R7);
impl_or_relation!(R1, R2, R3, R4, R5, R6, R7, R8);
impl_or_relation!(R1, R2, R3, R4, R5, R6, R7, R8, R9);

macro_rules! impl_not_relation {
    ($($t: ident), +) => {
        $(
            impl QRelation for QNot<$t> {
                fn filter_inner<S, const N: usize, const K: usize>(
                    node: &ArcNode<N, K>,
                    boundary: &S,
                    res: &mut EntityHashSet,
                ) where
                    S: CollisionQuery,
                {
                    let mut tmp = EntityHashSet::default();
                    $t::filter_inner(node, boundary, &mut tmp);
                    All::all(node, res);
                    res.extend(tmp);
                }
            }
        )+
    };
}

impl_not_relation!(Contain, Contained, Overlap, Disjoint);
