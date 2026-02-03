pub mod bvh;
pub mod displacement;
pub mod geometry;
pub mod report;
pub mod schema;
pub mod semantic;

use serde::{Deserialize, Serialize};

/// Level of validation to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ValidationLevel {
    /// Basic structural checks (e.g. required attributes).
    /// Fast.
    Minimal,
    /// Full 3MF Core Spec compliance (Resource IDs, Reference integrity).
    /// Recommended.
    Standard,
    /// Strict adherence (e.g. Metadata presence, no unknown attributes).
    Strict,
    /// Deep inspection (Geometry manifold checks, intersection tests).
    /// Expensive.
    Paranoid,
}

// Re-exports
pub use displacement::validate_displacement;
pub use geometry::validate_geometry;
pub use report::{ValidationReport, ValidationSeverity};
