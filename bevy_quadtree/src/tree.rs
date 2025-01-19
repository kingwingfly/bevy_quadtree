//! QuadTree

use crate::{
    collision::DynCollision,
    node::{ArcNode, Node},
    CollisionQuery, QRelation,
};
use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    prelude::*,
};
use core::fmt;
use parking_lot::RwLock;
use std::sync::Arc;

/// The QuadTree used as `Resource` in this plugin.
/// The root node boundary's center is (0, 0).
#[derive(Resource)]
pub struct QuadTree<
    const N: usize,
    const W: usize,
    const H: usize,
    const K: usize = 10,
    const ID: usize = 1,
> {
    pub(crate) root: ArcNode<N, K>,
    pub(crate) entities: Arc<RwLock<EntityHashMap<ArcNode<N, K>>>>,
}

impl<const N: usize, const W: usize, const H: usize, const K: usize, const ID: usize> fmt::Debug
    for QuadTree<N, W, H, K, ID>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuadTree")
            .field("quadtree id", &ID)
            .field("total entities", &self.len())
            .field("root", &self.root)
            .finish()
    }
}

impl<const N: usize, const W: usize, const H: usize, const K: usize, const ID: usize> Default
    for QuadTree<N, W, H, K, ID>
{
    fn default() -> Self {
        let root = Arc::new(RwLock::new(Node::root(Rect::from_center_size(
            Vec2::ZERO,
            Vec2::new(W as f32, H as f32),
        ))));
        Self {
            root,
            entities: Arc::new(RwLock::new(EntityHashMap::default())),
        }
    }
}

impl<const N: usize, const W: usize, const H: usize, const K: usize, const ID: usize>
    QuadTree<N, W, H, K, ID>
{
    fn len(&self) -> usize {
        self.entities.read().len()
    }

    pub(crate) fn insert<S>(&self, entity: Entity, shape: S)
    where
        S: DynCollision + 'static,
    {
        let shape: Box<dyn DynCollision> = Box::new(shape);
        let new_node = {
            let entities = self.entities.read();
            match entities.get(&entity) {
                Some(node) => Node::update(node, entity, shape),
                None => Node::insert(&self.root, entity, shape),
            }
        };
        let mut entities = self.entities.write();
        entities.extend(new_node);
    }

    pub(crate) fn remove(&self, entity: &Entity) {
        if let Some(node) = self.entities.write().remove(entity) {
            Node::remove(&node, entity);
        }
    }

    /// Query the entities within the given relation with the boundary [`S: CollisionQuery`](crate::CollisionQuery),
    /// such as [`CollisionRect`](crate::CollisionRect), [`CollisionRotatedRect`](crate::CollisionRotatedRect), [`CollisionCircle`](crate::CollisionCircle) and tuple/array of them.
    /// The rule of the relation is defined in [`CollisionQuery::query`] and [`query`](crate::query).
    ///
    /// [`QRelation`]: implemented for [`Disjoint`](crate::Disjoint), [`Overlap`](crate::Overlap),
    /// [`Contain`](crate::Contain), [`Contained`](crate::Contained), [`QOr`](crate::QOr), [`QNot`](crate::QNot).
    pub fn query<S, Q>(&self, boundary: &S) -> EntityHashSet
    where
        S: CollisionQuery,
        Q: QRelation,
    {
        Q::filter(&self.root, boundary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CollisionCircle, CollisionRect, Contain, Contained, Disjoint, Overlap, QNot, QOr};
    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn non_loose() {
        let tree: QuadTree<2, 4, 4> = QuadTree::default();
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        assert_eq!(tree.len(), 0);
        // (0, 0) r = 1
        tree.insert(Entity::PLACEHOLDER, CollisionCircle::new(Vec2::ZERO, 1.));
        assert_eq!(tree.len(), 1);
        // overwrites the Entity::PLACEHOLDER
        // (0, 0) 1x1
        tree.insert(
            Entity::PLACEHOLDER,
            CollisionRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE)),
        );
        assert_eq!(tree.len(), 1);
        // (0, 0) 1x1
        tree.insert(
            Entity::from_raw(rng.gen()),
            CollisionRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE)),
        );
        assert_eq!(tree.len(), 2);
        assert!(tree.root.read().children.is_none());
        // (1, 1) 1x1
        tree.insert(
            Entity::from_raw(rng.gen()),
            CollisionRect::from(Rect::from_center_size(Vec2::splat(1.), Vec2::ONE)),
        );
        assert!(tree.root.read().children.is_some());
        {
            let tree = tree.root.read();
            let child = tree.children.as_ref().unwrap()[0].read();
            assert_eq!(child.total(), 1);
        }
        // (1, 1) 1x1
        tree.insert(
            Entity::from_raw(rng.gen()),
            CollisionRect::from(Rect::from_center_size(Vec2::splat(1.), Vec2::ONE)),
        );
        assert!(tree.root.read().children.is_some());
        assert_eq!(tree.len(), 4);
        {
            let tree = tree.root.read();
            let child = tree.children.as_ref().unwrap()[0].read();
            assert_eq!(child.total(), 2);
        }
        // (0.5, 0.5) 0.2x0.2
        tree.insert(
            Entity::from_raw(rng.gen()),
            CollisionRect::from(Rect::from_center_size(Vec2::splat(0.5), Vec2::splat(0.2))),
        );
        assert!(tree.root.read().children.is_some());
        assert_eq!(tree.len(), 5);
        {
            let root = tree.root.read();
            let child = root.children.as_ref().unwrap()[0].read();
            assert_eq!(child.len(), 2);
            assert_eq!(child.total(), 3);
            assert!(child.children.is_some());
            let child = child.children.as_ref().unwrap()[2].read();
            assert_eq!(child.total(), 1);
        }
        // update Entity::PLACEHOLDER from (0, 0) 1x1 to (1, 0) 1x1
        tree.insert(
            Entity::PLACEHOLDER,
            CollisionRect::from(Rect::from_center_size(Vec2::new(1., 0.), Vec2::ONE)),
        );
        assert_eq!(tree.len(), 5);
        assert_eq!(tree.root.read().total(), 5);
        // update Entity::PLACEHOLDER from (1, 0) 1x1 to (0.5, 0.5) 0.2x0.3
        tree.insert(
            Entity::PLACEHOLDER,
            CollisionRect::from(Rect::from_center_size(
                Vec2::splat(0.5),
                Vec2::new(0.2, 0.3),
            )),
        );
        assert_eq!(tree.len(), 5);
        {
            let root = tree.root.read();
            assert_eq!(root.len(), 1);
            assert_eq!(root.total(), 5);
            let child = root.children.as_ref().unwrap()[0].read();
            assert_eq!(child.len(), 2);
            assert_eq!(child.total(), 4);
            assert!(child.children.is_some());
            let child = child.children.as_ref().unwrap()[2].read();
            assert_eq!(child.len(), 2);
            assert_eq!(child.total(), 2);
        }
        // update Entity::PLACEHOLDER from (0.5, 0.5) 0.2x0.3 to (1, -1) 1x1
        tree.insert(
            Entity::PLACEHOLDER,
            CollisionRect::from(Rect::from_center_size(Vec2::new(1., -1.), Vec2::splat(1.))),
        );
        assert_eq!(tree.len(), 5);
        {
            let root = tree.root.read();
            assert_eq!(root.len(), 1);
            assert_eq!(root.total(), 5);
            let child = root.children.as_ref().unwrap()[0].read();
            assert_eq!(child.len(), 2);
            assert_eq!(child.total(), 3);
            let child = root.children.as_ref().unwrap()[3].read();
            assert_eq!(child.len(), 1);
        }
        // remove Entity::PLACEHOLDER
        tree.remove(&Entity::PLACEHOLDER);
        assert_eq!(tree.len(), 4);
        {
            let root = tree.root.read();
            assert_eq!(root.len(), 1);
            assert_eq!(root.total(), 4);
            let child = root.children.as_ref().unwrap()[0].read();
            assert_eq!(child.len(), 2);
            assert_eq!(child.total(), 3);
            let child = root.children.as_ref().unwrap()[3].read();
            assert_eq!(child.len(), 0);
        }
        // Test merge after remove
        tree.insert(
            Entity::from_raw(rng.gen()),
            CollisionRect::from(Rect::from_center_size(
                Vec2::splat(-1.),
                Vec2::new(0.2, 0.3),
            )),
        );
        tree.insert(
            Entity::from_raw(rng.gen()),
            CollisionRect::from(Rect::from_center_size(
                Vec2::splat(-1.),
                Vec2::new(0.2, 0.3),
            )),
        );
        tree.insert(
            Entity::PLACEHOLDER,
            CollisionRect::from(Rect::from_center_size(
                Vec2::splat(-0.5),
                Vec2::new(0.2, 0.3),
            )),
        );
        tree.remove(&Entity::PLACEHOLDER);
        assert_eq!(tree.len(), 6);
        {
            let root = tree.root.read();
            assert_eq!(root.len(), 1);
            assert_eq!(root.total(), 6);
            let child = root.children.as_ref().unwrap()[2].read();
            assert_eq!(child.len(), 2);
            assert_eq!(child.total(), 2);
            assert!(child.children.is_none());
        }
        let q = tree.query::<_, Overlap>(&CollisionRect::from(Rect::from_center_size(
            Vec2::ZERO,
            Vec2::ONE,
        )));
        assert_eq!(q.len(), 4);
        let q = tree.query::<_, QOr<(Overlap, Contain)>>(&CollisionCircle::new(Vec2::ZERO, 1.));
        assert_eq!(q.len(), 4);
        let q = tree.query::<_, QNot<Contain>>(&CollisionCircle::new(Vec2::ZERO, 1.));
        assert_eq!(q.len(), 4);
        let q = tree.query::<_, Disjoint>(&CollisionCircle::new(Vec2::splat(0.5), 1.));
        assert_eq!(q.len(), 2);
        let q = tree.query::<_, Contain>(&CollisionCircle::new(Vec2::splat(0.5), 1.));
        assert_eq!(q.len(), 1);
        let q = tree.query::<_, Contained>(&CollisionCircle::new(Vec2::ONE, 0.4));
        assert_eq!(q.len(), 2);
    }
}
