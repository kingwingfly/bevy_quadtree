#![doc = include_str!("../README.md")]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod bound_check;
mod node;
mod plugin;
mod system;
mod tree;

#[cfg(test)]
mod test_utils;

pub use bound_check::{BoundCheck, RelativePosition};
pub use plugin::QuadTreePlugin;
