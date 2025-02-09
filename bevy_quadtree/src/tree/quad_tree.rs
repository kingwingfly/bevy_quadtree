//! QuadTree

pub(crate) use super::tree_impl::{NodeID, Tree};

use super::{tree_impl::Change, QRelation};
use crate::collision::{AsDynCollision, CollisionQuery};
use bevy_ecs::{
    entity::{EntityHashMap, EntityHashSet},
    prelude::*,
};
use core::fmt;
use parking_lot::RwLock;
use std::any::type_name;

/// The QuadTree used as `Resource` in this plugin.
/// The root node boundary's center is (0, 0).
#[derive(Resource)]
pub struct QuadTree<const ID: usize = 0> {
    pub(crate) tree: Tree,
    pub(crate) entities: RwLock<EntityHashMap<NodeID>>,
}

impl<const ID: usize> fmt::Debug for QuadTree<ID> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("tree", &self.tree)
            .field("len", &self.len())
            .finish()
    }
}

impl<const ID: usize> QuadTree<ID> {
    pub(crate) fn new(
        n: usize,
        d: usize,
        w: usize,
        h: usize,
        x: usize,
        y: usize,
        k: usize,
    ) -> Self {
        Self {
            tree: Tree::new(n, d, w as f32, h as f32, x as f32, y as f32, k as f32 / 10.),
            entities: RwLock::new(EntityHashMap::default()),
        }
    }

    fn len(&self) -> usize {
        self.entities.read().len()
    }

    pub(crate) fn insert<S>(&self, entity: Entity, shape: S)
    where
        S: AsDynCollision + 'static,
    {
        let shape = shape.as_dyn_collision();
        let mut changed = vec![];
        match self.entities.read().get(&entity) {
            Some(id) => self.tree.update(*id, entity, shape, &mut changed),
            None => self.tree.insert(0, entity, shape, &mut changed, vec![]),
        };
        let mut entities = self.entities.write();
        for c in changed {
            match c {
                Change::Move(entity, id) => entities.insert(entity, id),
                Change::Leave(entity) => entities.remove(&entity),
            };
        }
    }

    pub(crate) fn remove(&self, entity: &Entity) {
        if let Some(id) = self.entities.write().remove(entity) {
            self.tree.remove(id, entity);
            self.tree.merge_up(id);
        }
    }

    /// Query the entities within the given relation with the boundary [`S: CollisionQuery`](crate::collision::CollisionQuery),
    /// such as [`CollisionRect`](crate::CollisionRect), [`CollisionRotatedRect`](crate::CollisionRotatedRect), [`CollisionCircle`](crate::CollisionCircle) and tuple/array of them.
    /// The rule of the relation is defined in [`CollisionQuery::query`] and [`query`](crate::tree::query).
    ///
    /// [`QRelation`]: implemented for [`Disjoint`](crate::Disjoint), [`Overlap`](crate::Overlap),
    /// [`Contain`](crate::Contain), [`Contained`](crate::Contained), [`QOr`](crate::QOr), [`QNot`](crate::QNot).
    pub fn query<Q>(&self, boundary: &dyn CollisionQuery) -> EntityHashSet
    where
        Q: QRelation,
    {
        Q::filter(&self.tree.query_tree(), boundary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CollisionCircle, CollisionRect, Contain, Contained, Disjoint, Overlap, QNot, QOr};
    use bevy_math::prelude::*;
    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn non_loose() {
        let qtree: QuadTree<0> = QuadTree::new(2, 4, 4, 4, 0, 0, 10);
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        assert_eq!(qtree.len(), 0);
        // (0, 0) r = 1
        qtree.insert(Entity::PLACEHOLDER, CollisionCircle::new(Vec2::ZERO, 1.));
        assert_eq!(qtree.len(), 1);
        // overwrites the Entity::PLACEHOLDER
        // (0, 0) 1x1
        qtree.insert(
            Entity::PLACEHOLDER,
            CollisionRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE)),
        );
        assert_eq!(qtree.len(), 1);
        // (0, 0) 1x1
        qtree.insert(
            Entity::from_raw(rng.random()),
            CollisionRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE)),
        );
        assert_eq!(qtree.len(), 2);
        // (1, 1) 1x1
        qtree.insert(
            Entity::from_raw(rng.random()),
            CollisionRect::from(Rect::from_center_size(Vec2::splat(1.), Vec2::ONE)),
        );
        {
            assert_eq!(qtree.tree.total(1), 1);
        }
        // (1, 1) 1x1
        qtree.insert(
            Entity::from_raw(rng.random()),
            CollisionRect::from(Rect::from_center_size(Vec2::splat(1.), Vec2::ONE)),
        );
        assert_eq!(qtree.len(), 4);
        {
            assert_eq!(qtree.tree.total(1), 2);
        }
        // (0.5, 0.5) 0.2x0.2
        qtree.insert(
            Entity::from_raw(rng.random()),
            CollisionRect::from(Rect::from_center_size(Vec2::splat(0.5), Vec2::splat(0.2))),
        );
        assert_eq!(qtree.len(), 5);
        {
            assert_eq!(qtree.tree[1].len(), 2);
            assert_eq!(qtree.tree.total(1), 3);
            assert_eq!(qtree.tree.total(7), 1);
        }
        // update Entity::PLACEHOLDER from (0, 0) 1x1 to (1, 0) 1x1
        qtree.insert(
            Entity::PLACEHOLDER,
            CollisionRect::from(Rect::from_center_size(Vec2::new(1., 0.), Vec2::ONE)),
        );
        assert_eq!(qtree.len(), 5);
        assert_eq!(qtree.tree.total(0), 5);
        // update Entity::PLACEHOLDER from (1, 0) 1x1 to (0.5, 0.5) 0.2x0.3
        qtree.insert(
            Entity::PLACEHOLDER,
            CollisionRect::from(Rect::from_center_size(
                Vec2::splat(0.5),
                Vec2::new(0.2, 0.3),
            )),
        );
        assert_eq!(qtree.len(), 5);
        {
            assert_eq!(qtree.tree[0].len(), 1);
            assert_eq!(qtree.tree.total(0), 5);
            assert_eq!(qtree.tree[1].len(), 2);
            assert_eq!(qtree.tree.total(1), 4);
            assert_eq!(qtree.tree[7].len(), 2);
            assert_eq!(qtree.tree.total(7), 2);
        }
        // update Entity::PLACEHOLDER from (0.5, 0.5) 0.2x0.3 to (1, -1) 1x1
        qtree.insert(
            Entity::PLACEHOLDER,
            CollisionRect::from(Rect::from_center_size(Vec2::new(1., -1.), Vec2::splat(1.))),
        );
        assert_eq!(qtree.len(), 5);
        {
            assert_eq!(qtree.tree[0].len(), 1);
            assert_eq!(qtree.tree.total(0), 5);
            assert_eq!(qtree.tree[1].len(), 2);
            assert_eq!(qtree.tree.total(1), 3);
            assert_eq!(qtree.tree[4].len(), 1);
        }
        // remove Entity::PLACEHOLDER
        qtree.remove(&Entity::PLACEHOLDER);
        assert_eq!(qtree.len(), 4);
        {
            assert_eq!(qtree.tree[0].len(), 1);
            assert_eq!(qtree.tree.total(0), 4);
            assert_eq!(qtree.tree[1].len(), 2);
            assert_eq!(qtree.tree.total(1), 3);
            assert_eq!(qtree.tree[4].len(), 0);
        }
        // Test merge after remove
        qtree.insert(
            Entity::from_raw(rng.random()),
            CollisionRect::from(Rect::from_center_size(
                Vec2::splat(-1.),
                Vec2::new(0.2, 0.3),
            )),
        );
        qtree.insert(
            Entity::from_raw(rng.random()),
            CollisionRect::from(Rect::from_center_size(
                Vec2::splat(-1.),
                Vec2::new(0.2, 0.3),
            )),
        );
        qtree.insert(
            Entity::PLACEHOLDER,
            CollisionRect::from(Rect::from_center_size(
                Vec2::splat(-0.5),
                Vec2::new(0.2, 0.3),
            )),
        );
        qtree.remove(&Entity::PLACEHOLDER);
        assert_eq!(qtree.len(), 6);
        {
            assert_eq!(qtree.tree[0].len(), 1);
            assert_eq!(qtree.tree.total(0), 6);
            assert_eq!(qtree.tree[3].len(), 2);
            assert_eq!(qtree.tree.total(3), 2);
            assert!(qtree.tree[3].is_leaf());
        }
        let q = qtree.query::<Overlap>(&CollisionRect::from(Rect::from_center_size(
            Vec2::ZERO,
            Vec2::ONE,
        )));
        assert_eq!(q.len(), 4);
        let q = qtree.query::<QOr<(Overlap, Contain)>>(&CollisionCircle::new(Vec2::ZERO, 1.));
        assert_eq!(q.len(), 4);
        let q = qtree.query::<QNot<Contain>>(&CollisionCircle::new(Vec2::ZERO, 1.));
        assert_eq!(q.len(), 4);
        let q = qtree.query::<Disjoint>(&CollisionCircle::new(Vec2::splat(0.5), 1.));
        assert_eq!(q.len(), 2);
        let q = qtree.query::<Contain>(&CollisionCircle::new(Vec2::splat(0.5), 1.));
        assert_eq!(q.len(), 1);
        let q = qtree.query::<Contained>(&CollisionCircle::new(Vec2::ONE, 0.4));
        assert_eq!(q.len(), 2);
    }
}
