use crate::model::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ModelDiff {
    pub metadata_diffs: Vec<MetadataDiff>,
    pub resource_diffs: Vec<ResourceDiff>,
    pub build_diffs: Vec<BuildDiff>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataDiff {
    pub key: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResourceDiff {
    Added { id: u32, type_name: String },
    Removed { id: u32, type_name: String },
    Changed { id: u32, details: Vec<String> },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BuildDiff {
    Added {
        object_id: u32,
    },
    Removed {
        object_id: u32,
    },
    Changed {
        object_id: u32,
        details: Vec<String>,
    },
}

impl ModelDiff {
    pub fn is_empty(&self) -> bool {
        self.metadata_diffs.is_empty()
            && self.resource_diffs.is_empty()
            && self.build_diffs.is_empty()
    }
}

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
