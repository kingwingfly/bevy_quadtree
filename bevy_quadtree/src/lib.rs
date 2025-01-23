#![doc = include_str!("../README.md")]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![allow(clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod collision;
pub mod plugin;
pub mod shape;
mod system;
pub mod tree;

pub use collision::Disassemble;
pub use plugin::QuadTreePlugin;
#[cfg(feature = "multi-quadtree")]
pub use plugin::{MultiQuadTreePlugin, QTConfig, QuadTreePluginConfig};
pub use shape::{CollisionCircle, CollisionRect, CollisionRotatedRect};
pub use tree::{All, Contain, Contained, Disjoint, Overlap, QNot, QOr, QuadTree};
