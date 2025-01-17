#![doc = include_str!("../README.md")]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![allow(clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod collision;
mod node;
mod plugin;
mod shape;
mod system;
mod tree;

pub use collision::{
    AsCollision, Collision, Disassemble, DynCollision, QRelation, Relation, UpdateCollision,
};
pub use plugin::QuadTreePlugin;
pub use shape::{CollisionCircle, CollisionRect};
pub use tree::QuadTree;
