//! Progressive validation system for 3MF models.
//!
//! This module provides a four-level validation system that lets you choose the right balance
//! between performance and thoroughness for your use case.
//!
//! ## Validation Levels
//!
//! The [`ValidationLevel`] enum defines four progressively stricter validation modes:
//!
//! ### Minimal
//!
//! Basic structural checks:
//! - Required attributes present (`unit`, `id`, etc.)
//! - Valid XML structure
//! - No obviously malformed data
//!
//! **Performance**: Very fast (< 1ms for typical models)
//! **Use case**: Quick sanity check, development testing
//!
//! ### Standard (Recommended)
//!
//! Reference integrity and semantic correctness:
//! - All resource IDs are unique
//! - Build items reference valid objects
//! - Material references point to existing materials
//! - Component references form valid DAG (no cycles)
//! - Vertex indices within mesh bounds
//!
//! **Performance**: Fast (< 10ms for typical models)
//! **Use case**: Production parsing, most applications
//!
//! ### Strict
//!
//! Full 3MF specification compliance:
//! - All Standard checks
//! - Metadata requirements enforced
//! - No unknown attributes or elements
//! - Extension namespaces correctly declared
//!
//! **Performance**: Moderate
//! **Use case**: Spec conformance testing, quality assurance
//!
//! ### Paranoid
//!
//! Deep geometry analysis with advanced algorithms:
//! - All Strict checks
//! - Mesh manifoldness (edge-manifold, vertex-manifold)
//! - Self-intersection detection (BVH-accelerated)
//! - Orientation consistency (outward-facing normals)
//! - Degenerate triangle detection
//! - Island detection (connected components)
//! - Type-specific constraints (Model objects must be manifold)
//!
//! **Performance**: Slow (can be seconds for complex models, O(n²) worst case)
//! **Use case**: Critical manufacturing workflows, geometry repair pipelines
//!
//! ## Usage
//!
//! ```
//! use lib3mf_core::{Model, validation::ValidationLevel};
//!
//! let model = Model::default();
//!
//! // Quick check
//! let report = model.validate(ValidationLevel::Minimal);
//! assert!(!report.has_errors());
//!
//! // Production use
//! let report = model.validate(ValidationLevel::Standard);
//! if report.has_errors() {
//!     for item in &report.items {
//!         eprintln!("[{}] {}", item.code, item.message);
//!     }
//! }
//!
//! // Critical applications
//! let report = model.validate(ValidationLevel::Paranoid);
//! ```
//!
//! ## Validation Report
//!
//! Validation returns a [`ValidationReport`] containing:
//!
//! - **Errors**: Spec violations, broken references, invalid geometry
//! - **Warnings**: Suspicious patterns, deprecated features, non-standard usage
//! - **Info**: Informational messages, optimization suggestions
//!
//! Each item includes:
//! - Error code (numeric, for programmatic handling)
//! - Human-readable message
//! - Optional suggestion for fixing the issue
//! - Optional context (e.g., "Object 5", "Triangle 123")
//!
//! ## Geometry Validation Algorithms
//!
//! The [`geometry`] module implements advanced mesh validation:
//!
//! - **Manifoldness**: Each edge shared by exactly 2 triangles (edge-manifold). Each vertex has
//!   a single connected fan of triangles (vertex-manifold).
//! - **Self-intersection**: BVH (Bounding Volume Hierarchy) acceleration for O(n log n) triangle-triangle
//!   intersection tests. See [`bvh`] module.
//! - **Orientation**: Directed edge analysis to detect reversed normals.
//! - **Island detection**: Connected component analysis using depth-first search.
//!
//! ## Performance Optimization
//!
//! For large models, validation can be expensive. Strategies:
//!
//! - **Use Standard for production**: Catches 99% of issues in < 10ms
//! - **Defer Paranoid to background**: Run geometry checks asynchronously
//! - **Cache results**: Validation reports are cloneable and serializable
//! - **Progressive checking**: Validate incrementally during parsing (not yet implemented)

pub mod bvh;
pub mod displacement;
pub mod geometry;
pub mod report;
pub mod schema;
pub mod semantic;

use serde::{Deserialize, Serialize};

/// Level of validation to perform.
///
/// This enum defines four progressively stricter validation modes. Higher levels include
/// all checks from lower levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ValidationLevel {
    /// Basic structural checks (required attributes, valid XML).
    ///
    /// Very fast (< 1ms). Suitable for quick sanity checks during development.
    ///
    /// Checks:
    /// - Required attributes present
    /// - Valid data types (numbers parse, IDs are positive)
    /// - No obviously malformed structures
    Minimal,

    /// Full 3MF Core Spec compliance (resource IDs, reference integrity).
    ///
    /// Fast (< 10ms). **Recommended for production use.**
    ///
    /// Includes all Minimal checks plus:
    /// - Resource IDs are unique
    /// - Build items reference valid objects
    /// - Material/property references are valid
    /// - Component references form valid DAG (no cycles)
    /// - Vertex indices within mesh bounds
    Standard,

    /// Strict adherence to spec (metadata, no unknown attributes).
    ///
    /// Moderate performance. Suitable for conformance testing and quality assurance.
    ///
    /// Includes all Standard checks plus:
    /// - Metadata requirements enforced
    /// - No unknown attributes or elements
    /// - Extension namespaces correctly declared
    /// - Proper content type and relationship registration
    Strict,

    /// Deep geometry inspection (manifold checks, intersection tests).
    ///
    /// Slow (can be seconds for complex models). Use for critical manufacturing workflows.
    ///
    /// Includes all Strict checks plus:
    /// - Mesh manifoldness (edge-manifold and vertex-manifold)
    /// - Self-intersection detection (BVH-accelerated, still O(n²) worst case)
    /// - Orientation consistency (outward-facing normals)
    /// - Degenerate triangle detection
    /// - Connected component analysis
    /// - Type-specific geometry constraints
    Paranoid,
}

// Re-exports
pub use displacement::validate_displacement;
pub use geometry::validate_geometry;
pub use report::{ValidationReport, ValidationSeverity};
