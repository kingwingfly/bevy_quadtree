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
        let root = Arc::new(RwLock::new(Node::from(Rect::from_center_size(
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
        let shape: Arc<dyn DynCollision> = Arc::new(shape);
        let new_node = {
            let entities = self.entities.read();
            match entities.get(&entity) {
                Some(node) => Node::update(node, entity, Arc::clone(&shape)),
                None => Node::insert(&self.root, entity, Arc::clone(&shape)),
            }
        };
        let mut entities = self.entities.write();
        for (e, n) in new_node {
            entities.insert(e, n);
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
        tree.insert(
            Entity::PLACEHOLDER,
            CollisionCircle {
                center: Vec2::ZERO,
                radius: 1.,
            },
        );
        assert_eq!(tree.len(), 1);
        tree.insert(
            Entity::PLACEHOLDER,
            CollisionRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE)),
        );
        assert_eq!(tree.len(), 1);
        tree.insert(
            Entity::from_raw(rng.gen()),
            CollisionRect::from(Rect::from_center_size(Vec2::ZERO, Vec2::ONE)),
        );
        assert_eq!(tree.len(), 2);
        assert!(tree.root.read().children.is_none());
        tree.insert(
            Entity::from_raw(rng.gen()),
            CollisionRect::from(Rect::from_center_size(Vec2::splat(1.), Vec2::ONE)),
        );
        assert!(tree.root.read().children.is_some());
        assert_eq!(
            {
                let tree = tree.root.read();
                let child = tree.children.as_ref().unwrap()[0].read();
                child.len()
            },
            1
        );
        tree.insert(
            Entity::from_raw(rng.gen()),
            CollisionRect::from(Rect::from_center_size(Vec2::splat(1.), Vec2::ONE)),
        );
        assert!(tree.root.read().children.is_some());
        assert_eq!(tree.len(), 4);
        assert_eq!(
            {
                let tree = tree.root.read();
                let child = tree.children.as_ref().unwrap()[0].read();
                child.len()
            },
            2
        )
    }
}
