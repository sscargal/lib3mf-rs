//! Core data model for 3MF files.
//!
//! This module contains the in-memory representation of a 3MF document, including all geometry,
//! materials, build instructions, and extension data.
//!
//! ## Key Types
//!
//! - [`Model`]: The root structure representing an entire 3MF document. Contains resources,
//!   build instructions, metadata, and attachments.
//! - [`ResourceCollection`]: Central registry for all resources (objects, materials, textures)
//!   using a global ID namespace within the model.
//! - [`Object`]: A 3MF resource representing geometry or components. Can be a mesh, support,
//!   surface, solid support, boolean shape, or other type.
//! - [`Mesh`]: Triangle mesh geometry with vertices, triangles, and optional beam lattice data.
//! - [`Build`]: Collection of [`BuildItem`]s that define which objects to print and where to
//!   position them.
//! - [`BuildItem`]: An instance of an object in the build volume, with optional transformation.
//!
//! ## Material System
//!
//! Materials are applied to geometry via property IDs (`pid`):
//!
//! - [`BaseMaterial`]: Simple named materials with optional display color
//! - [`ColorGroup`]: Per-vertex or per-triangle color assignments
//! - [`Texture2DGroup`]: Image-based materials with UV coordinates
//! - [`CompositeMaterials`]: Blended combinations of base materials
//! - [`MultiProperties`]: Multi-channel property assignments
//!
//! Materials can be assigned at the object level (default for all triangles) or overridden
//! per-triangle or per-vertex.
//!
//! ## Extension Support
//!
//! The model includes first-class support for 3MF extensions:
//!
//! - **Beam Lattice**: [`BeamLattice`] and [`BeamSet`] for structural lattices
//! - **Slice**: [`SliceStack`] for layer-based geometry
//! - **Volumetric**: [`VolumetricStack`] for voxel data
//! - **Boolean Operations**: [`BooleanShape`] for CSG operations
//! - **Displacement**: [`DisplacementMesh`] for texture-driven surface modification
//! - **Secure Content**: Cryptographic features (see [`secure_content`] module)
//!
//! ## Design Philosophy
//!
//! The model follows an **immutable-by-default** design:
//!
//! - All structures derive `Clone` for easy copying
//! - Modification happens via explicit operations (e.g., [`MeshRepair`] trait)
//! - Thread-safe by default (no interior mutability)
//! - Predictable behavior: functions don't have hidden side effects
//!
//! ## Re-exports
//!
//! For convenience, all public types are re-exported at the crate root via `pub use model::*`.
//! You can use `lib3mf_core::Model` instead of `lib3mf_core::model::Model`.

/// Build instructions — `Build` and `BuildItem` types.
pub mod build;
/// Root `Model` struct and its `validate` / `compute_stats` methods.
pub mod core;
/// XML-DSIG crypto data structures used by the Secure Content Extension.
pub mod crypto;
/// Material and texture types (colors, base materials, composites, etc.).
pub mod materials;
/// Mesh geometry types (`Mesh`, `Triangle`, `Vertex`, `BeamLattice`, etc.).
pub mod mesh;
/// Multi-part `Package` type for Production Extension multi-model files.
pub mod package;
/// Mesh repair operations (`MeshRepair` trait and `RepairStats`).
pub mod repair;
/// Cross-file component resolver (`PartResolver`, `ResolvedMesh`, `ResolveOptions`).
pub mod resolver;
/// `ResourceCollection` — central registry for all model resources.
pub mod resources;
/// Secure Content Extension key store types (`KeyStore`, `Consumer`, etc.).
pub mod secure_content;
/// Slice Extension types (`SliceStack`, `Slice`, `Polygon`, etc.).
pub mod slice;
/// Model statistics types returned by `Model::compute_stats()`.
pub mod stats;
/// Internal implementation of `compute_stats()` — not part of the public API surface.
pub mod stats_impl;

/// Unit of measurement enum and conversion utilities.
pub mod units;
/// Volumetric Extension types (`VolumetricStack`, `VolumetricLayer`, etc.).
pub mod volumetric;

pub use build::*;
pub use core::*;
pub use crypto::*;
pub use materials::*;
pub use mesh::*;
pub use package::*;
pub use repair::*;
pub use resolver::{ResolveOptions, ResolvedMesh};
pub use resources::*;
pub use secure_content::*;
pub use slice::*;
pub use stats::*;

pub use units::*;
pub use volumetric::*;
