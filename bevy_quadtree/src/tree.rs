use crate::{
    node::{ArcNode, Node},
    DynCollision,
};
use bevy::{ecs::entity::EntityHashMap, prelude::*};
use core::fmt;
use parking_lot::RwLock;
use std::sync::Arc;

/// The QuadTree used as `Resource` in this plugin.
/// The root node boundary's center is (0, 0).
#[derive(Resource)]
pub struct QuadTree<const N: usize, const W: usize, const H: usize, const K: usize = 10> {
    pub(self) root: ArcNode<N, K>,
    entities: Arc<RwLock<EntityHashMap<ArcNode<N, K>>>>,
}

impl<const N: usize, const W: usize, const H: usize, const K: usize> fmt::Debug
    for QuadTree<N, W, H, K>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuadTree")
            .field("total entities", &self.len())
            .field("root", &self.root)
            .finish()
    }
}

impl<const N: usize, const W: usize, const H: usize, const K: usize> Default
    for QuadTree<N, W, H, K>
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

impl<const N: usize, const W: usize, const H: usize, const K: usize> QuadTree<N, W, H, K> {
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
        for (e, n) in new_node {
            entities.insert(e, n);
        }
    }

    pub(crate) fn remove(&self, entity: &Entity) {
        if let Some(node) = self.entities.write().remove(entity) {
            node.write().remove(entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::{CollisionCircle, CollisionRect};
    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn non_loose() {
        let tree: QuadTree<2, 4, 4> = QuadTree::default();
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        assert_eq!(tree.len(), 0);
        // (0, 0) r = 1
        tree.insert(
            Entity::PLACEHOLDER,
            CollisionCircle {
                center: Vec2::ZERO,
                radius: 1.,
            },
        );
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
            assert_eq!(child.len(), 1);
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
            assert_eq!(child.len(), 2);
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
            assert!(child.children.is_some());
            let child = child.children.as_ref().unwrap()[2].read();
            assert_eq!(child.len(), 1);
        }
        // update Entity::PLACEHOLDER from (0, 0) 1x1 to (1, 0) 1x1
        tree.insert(
            Entity::PLACEHOLDER,
            CollisionRect::from(Rect::from_center_size(Vec2::new(1., 0.), Vec2::ONE)),
        );
        assert_eq!(tree.len(), 5);
        assert_eq!(tree.root.read().len(), 2);
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
            let child = root.children.as_ref().unwrap()[0].read();
            assert_eq!(child.len(), 2);
            assert!(child.children.is_some());
            let child = child.children.as_ref().unwrap()[2].read();
            assert_eq!(child.len(), 2);
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
            let child = root.children.as_ref().unwrap()[0].read();
            assert_eq!(child.len(), 2);
            let child = root.children.as_ref().unwrap()[3].read();
            assert_eq!(child.len(), 1);
        }
    }
}
