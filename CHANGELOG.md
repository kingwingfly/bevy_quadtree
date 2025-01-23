# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org).

<!--
Note: In this file, do not use the hard wrap in the middle of a sentence for compatibility with GitHub comment style markdown rendering.
-->

## [Unreleased]
## [0.15.1-beta.3] - 2025-01-23

- remove unrelated type params of `QuadTree`
- support center setting

## [0.15.1-beta.2] - 2025-01-23

- improved the organization of public exports (`pub use`) to enhance usability and clarity.

## [0.15.1-beta.1] - 2025-01-23

- this crate is ready, use beta SemVer
- fix a bug in merge up empty nodes
- multi-quadtree support: see `MultiQuadTreePlugin`
- fix doc mistakes

## [0.15.1-alpha9] - 2025-01-22

- fix doc mistakes
- fix bug about shapes `ID`

## [0.15.1-alpha8] - 2025-01-22

- refator `QuadTree` (replace recursion with loop), a huge performance improvement in theory
- remove recursion in queries
- better Debug for shapes
- shapes with ID

## [0.15.1-alpha7] - 2025-01-20

- doc improvement

## [0.15.1-alpha6] - 2025-01-20

- simplify `CollisionQuery` trait
- improve documentation
- simplify dependencies
- type params check

## [0.15.1-alpha5] - 2025-01-20

- decouple `Disassemble` from `CollisionQuery`

## [0.15.1-alpha4] - 2025-01-20

- performance improvement
- fix bug in feature `sprite`
- fix bug caused by duplicate added updating system
- remove unnecessary type parameter in `QuadTree::query`

## [0.15.1-alpha3] - 2025-01-20

- fix doc mistakes
- remove unreachable impl for tuple

## [0.15.1-alpha2] - 2025-01-20

- Add `ID` type parameter for differentiating multiple quadtree if needed.

## [0.15.1-alpha1] - 2025-01-19

- MVP
