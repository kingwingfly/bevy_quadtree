#![doc = include_str!("../README.md")]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![allow(clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod collision;
mod node;
mod plugin;
mod system;
mod tree;

#[cfg(test)]
mod test_utils;

pub use collision::{AsCollision, Collision, DynCollision, RelativePosition, UpdateCollision};
pub use plugin::QuadTreePlugin;
