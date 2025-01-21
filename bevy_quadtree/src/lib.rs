#![doc = include_str!("../README.md")]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![allow(clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod collision;
pub mod plugin;
pub mod shape;
mod system;
pub mod tree;

pub use collision::{Collision, CollisionQuery, Disassemble, UpdateCollision};
pub use plugin::{QuadTreePlugin, TrackingPair};
pub use shape::{CollisionCircle, CollisionRect, CollisionRotatedRect};
pub use tree::{All, Contain, Contained, Disjoint, Overlap, QNot, QOr, QRelation, QuadTree};
