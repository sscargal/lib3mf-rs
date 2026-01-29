use crate::model::{Build, ResourceCollection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root element of a 3MF document.
///
/// The `Model` contains all information required to describe a 3D model, including:
/// - Resources (Meshes, Materials, Textures)
/// - Build instructions (Item positioning)
/// - Metadata (Authors, Copyright, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// The unit of measurement for geometry coordinates.
    #[serde(default)]
    pub unit: Unit,

    /// The language of the model content (e.g., "en-US").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Arbitrary metadata key-value pairs.
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Collection of all resources (objects, materials) used in the build.
    #[serde(default)]
    pub resources: ResourceCollection,

    /// The build definition, containing instances of objects to be printed.
    #[serde(default)]
    pub build: Build,
}

impl Model {
    pub fn validate(
        &self,
        level: crate::validation::ValidationLevel,
    ) -> crate::validation::ValidationReport {
        use crate::validation::{ValidationLevel, geometry, schema, semantic};

        let mut report = crate::validation::ValidationReport::new();

        // Minimal: Schema validation (placeholders usually checked by parser, but explicit invariants here)
        if level >= ValidationLevel::Minimal {
            schema::validate_schema(self, &mut report);
        }

        // Standard: Semantic validation (integrity)
        if level >= ValidationLevel::Standard {
            semantic::validate_semantic(self, &mut report);
        }

        // Paranoid: Geometry validation
        if level >= ValidationLevel::Paranoid {
            geometry::validate_geometry(self, &mut report);
        }

        report
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            unit: Unit::Millimeter,
            language: None,
            metadata: HashMap::new(),
            resources: ResourceCollection::default(),
            build: Build::default(),
        }
    }
}

/// Units of measurement used in the 3MF model.
///
/// Affects how vertex coordinates are interpreted in real-world dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    /// 0.000001 meters
    Micron,
    /// 0.001 meters (Default)
    #[default]
    Millimeter,
    /// 0.01 meters
    Centimeter,
    /// 0.0254 meters
    Inch,
    /// 0.3048 meters
    Foot,
    /// 1.0 meters
    Meter,
}
