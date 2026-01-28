use crate::model::{Build, ResourceCollection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root element of a 3MF document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    #[serde(default)]
    pub unit: Unit,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    #[serde(default)]
    pub metadata: HashMap<String, String>,

    #[serde(default)]
    pub resources: ResourceCollection,

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Micron,
    #[default]
    Millimeter,
    Centimeter,
    Inch,
    Foot,
    Meter,
}
