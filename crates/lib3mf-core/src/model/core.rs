use super::units::Unit;
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

    /// Binary attachments (Textures, Thumbnails, etc.) stored by package path.
    /// Key: Path in archive (e.g., "Metadata/thumbnail.png", "3D/Textures/diffuse.png")
    /// Value: Binary content
    #[serde(skip)]
    pub attachments: HashMap<String, Vec<u8>>,

    /// Existing OPC relationships loaded from the archive.
    /// Key: Relationship file path (e.g., "3D/_rels/3dmodel.model.rels")
    /// Value: Parsed relationships
    #[serde(skip)]
    pub existing_relationships: HashMap<String, Vec<crate::archive::opc::Relationship>>,
}

impl Model {
    pub fn validate(
        &self,
        level: crate::validation::ValidationLevel,
    ) -> crate::validation::ValidationReport {
        use crate::validation::{ValidationLevel, displacement, geometry, schema, semantic};

        let mut report = crate::validation::ValidationReport::new();

        // Minimal: Schema validation (placeholders usually checked by parser, but explicit invariants here)
        if level >= ValidationLevel::Minimal {
            schema::validate_schema(self, &mut report);
        }

        // Standard: Semantic validation (integrity)
        if level >= ValidationLevel::Standard {
            semantic::validate_semantic(self, &mut report);
        }

        // All levels: Displacement validation (progressive checks)
        displacement::validate_displacement(self, level, &mut report);

        // Paranoid: Geometry validation
        if level >= ValidationLevel::Paranoid {
            geometry::validate_geometry(self, level, &mut report);
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
            attachments: HashMap::new(),
            existing_relationships: HashMap::new(),
        }
    }
}

// Unit enum moved to definition in units.rs
