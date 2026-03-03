use crate::model::Model;
use serde::{Deserialize, Serialize};

/// The computed diff between two 3MF models.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ModelDiff {
    /// Differences in metadata key-value pairs.
    pub metadata_diffs: Vec<MetadataDiff>,
    /// Differences in resource objects (added, removed, or changed).
    pub resource_diffs: Vec<ResourceDiff>,
    /// Differences in build item lists.
    pub build_diffs: Vec<BuildDiff>,
}

/// A difference in a single metadata key between two models.
#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataDiff {
    /// The metadata key that differs.
    pub key: String,
    /// The value in model A (or `None` if absent).
    pub old_value: Option<String>,
    /// The value in model B (or `None` if absent).
    pub new_value: Option<String>,
}

/// A difference in a resource between two models.
#[derive(Debug, Serialize, Deserialize)]
pub enum ResourceDiff {
    /// A resource was added in model B.
    Added {
        /// Resource ID.
        id: u32,
        /// Type name of the resource (e.g., `"Mesh"`, `"Components"`).
        type_name: String,
    },
    /// A resource was removed from model A.
    Removed {
        /// Resource ID.
        id: u32,
        /// Type name of the resource.
        type_name: String,
    },
    /// A resource changed between the two models.
    Changed {
        /// Resource ID.
        id: u32,
        /// Human-readable descriptions of what changed.
        details: Vec<String>,
    },
}

/// A difference in the build item list between two models.
#[derive(Debug, Serialize, Deserialize)]
pub enum BuildDiff {
    /// A build item was added in model B.
    Added {
        /// Object ID of the added build item.
        object_id: u32,
    },
    /// A build item was removed from model A.
    Removed {
        /// Object ID of the removed build item.
        object_id: u32,
    },
    /// A build item changed (e.g., count or transform).
    Changed {
        /// Object ID of the changed build item.
        object_id: u32,
        /// Human-readable descriptions of what changed.
        details: Vec<String>,
    },
}

impl ModelDiff {
    /// Returns `true` if there are no differences between the two models.
    pub fn is_empty(&self) -> bool {
        self.metadata_diffs.is_empty()
            && self.resource_diffs.is_empty()
            && self.build_diffs.is_empty()
    }
}

/// Compares two 3MF models and returns a `ModelDiff` describing the differences.
pub fn compare_models(model_a: &Model, model_b: &Model) -> ModelDiff {
    let mut diff = ModelDiff::default();

    // 1. Compare Metadata
    // Combine keys
    let mut all_keys: Vec<_> = model_a.metadata.keys().collect();
    for k in model_b.metadata.keys() {
        if !all_keys.contains(&k) {
            all_keys.push(k);
        }
    }
    all_keys.sort();
    all_keys.dedup();

    for key in all_keys {
        let val_a = model_a.metadata.get(key);
        let val_b = model_b.metadata.get(key);

        if val_a != val_b {
            diff.metadata_diffs.push(MetadataDiff {
                key: key.clone(),
                old_value: val_a.cloned(),
                new_value: val_b.cloned(),
            });
        }
    }

    // 2. Compare Resources
    // Strategy: Match by ID.
    // In strict 3MF, IDs are local to the package/model stream.
    // If we compare different files, IDs might differ but content be same.
    // However, usually we want to diff the *structure* as preserved.
    // If comparing two versions of same file, ID matching is appropriate.
    // If comparing totally different files, this naive ID matching might be noisy.
    // We assume "semantic version diff" here, so ID matching is primary.

    let resources_a = &model_a.resources;
    let resources_b = &model_b.resources;

    // Check Removed or Changed
    for res_a in resources_a.iter_objects() {
        match resources_b.get_object(res_a.id) {
            Some(res_b) => {
                let type_a = get_geometry_type_name(&res_a.geometry);
                let type_b = get_geometry_type_name(&res_b.geometry);

                if type_a != type_b {
                    diff.resource_diffs.push(ResourceDiff::Changed {
                        id: res_a.id.0,
                        details: vec![format!("Type changed: {} -> {}", type_a, type_b)],
                    });
                } else {
                    // Check mesh data if mesh
                    if let (
                        crate::model::Geometry::Mesh(mesh_a),
                        crate::model::Geometry::Mesh(mesh_b),
                    ) = (&res_a.geometry, &res_b.geometry)
                    {
                        let mut details = Vec::new();
                        if mesh_a.vertices.len() != mesh_b.vertices.len() {
                            details.push(format!(
                                "Vertex count changed: {} -> {}",
                                mesh_a.vertices.len(),
                                mesh_b.vertices.len()
                            ));
                        }
                        if mesh_a.triangles.len() != mesh_b.triangles.len() {
                            details.push(format!(
                                "Triangle count changed: {} -> {}",
                                mesh_a.triangles.len(),
                                mesh_b.triangles.len()
                            ));
                        }
                        // TODO: Implement deeper hash comparison

                        if !details.is_empty() {
                            diff.resource_diffs.push(ResourceDiff::Changed {
                                id: res_a.id.0,
                                details,
                            });
                        }
                    }
                }
            }
            None => {
                diff.resource_diffs.push(ResourceDiff::Removed {
                    id: res_a.id.0,
                    type_name: get_geometry_type_name(&res_a.geometry).to_string(),
                });
            }
        }
    }

    // Check Added
    for res_b in resources_b.iter_objects() {
        if !resources_a.exists(res_b.id) {
            diff.resource_diffs.push(ResourceDiff::Added {
                id: res_b.id.0,
                type_name: get_geometry_type_name(&res_b.geometry).to_string(),
            });
        }
    }

    // 3. Compare Build Items
    // Naive matching by object_id.
    // Build items are a list, order matters technically for printing but usually we treat as set of items to build.
    // But duplicate instances allowed? Spec says "one or more item elements".
    // We'll compare by (object_id, transform) tuples roughly.
    // Or just ID existence.
    // Let's iterate both lists.

    if model_a.build.items.len() != model_b.build.items.len() {
        diff.build_diffs.push(BuildDiff::Changed {
            object_id: 0, // Global placeholder
            details: vec![format!(
                "Item count changed: {} -> {}",
                model_a.build.items.len(),
                model_b.build.items.len()
            )],
        });
    }

    diff
}

fn get_geometry_type_name(g: &crate::model::Geometry) -> &'static str {
    match g {
        crate::model::Geometry::Mesh(_) => "Mesh",
        crate::model::Geometry::Components(_) => "Components",
        crate::model::Geometry::SliceStack(_) => "SliceStack",
        crate::model::Geometry::VolumetricStack(_) => "VolumetricStack",
        crate::model::Geometry::BooleanShape(_) => "BooleanShape",
        crate::model::Geometry::DisplacementMesh(_) => "DisplacementMesh",
    }
}
