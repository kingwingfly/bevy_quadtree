//! QuadTree

pub mod quad_tree;
pub mod query;
mod tree_impl;

pub use quad_tree::QuadTree;
pub use query::{All, Contain, Contained, Disjoint, Overlap, QNot, QOr, QRelation};
