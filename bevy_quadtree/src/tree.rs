use crate::{
    node::{ArcNode, Node},
    DynCollision,
};
use bevy::{ecs::entity::EntityHashMap, prelude::*};
use core::fmt;
use std::sync::{Arc, RwLock};

/// The QuadTree used as `Resource` in this plugin.
#[derive(Resource)]
pub struct QuadTree<const N: usize, const W: usize, const H: usize, const K: usize = 10> {
    root: ArcNode<N, K>,
    entities: Arc<RwLock<EntityHashMap<(ArcNode<N, K>, Arc<dyn DynCollision>)>>>,
}

impl<const N: usize, const W: usize, const H: usize, const K: usize> fmt::Debug
    for QuadTree<N, W, H, K>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuadTree")
            .field("root", &self.root)
            .field("total entities", &self.len())
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
        self.entities.read().unwrap().len()
    }

    pub(crate) fn insert<S>(&self, entity: Entity, shape: S)
    where
        S: DynCollision + 'static,
    {
        let shape: Arc<dyn DynCollision> = Arc::new(shape);
        let new_node = {
            let entities = self.entities.read().unwrap();
            match entities.get(&entity) {
                Some((node, old)) => {
                    let mut node = node.write().unwrap();
                    node.update_arc(entity, Arc::clone(old), Arc::clone(&shape))
                }
                None => Node::insert_arc(&self.root, entity, Arc::clone(&shape)),
            }
        };
        let mut entities = self.entities.write().unwrap();
        entities.insert(entity, (new_node, shape));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::CollisionCircle;

    #[test]
    fn insert_test() {
        let tree: QuadTree<2, 4, 4> = QuadTree::default();
    }
}
