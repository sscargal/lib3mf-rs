use crate::model::{Geometry, Model, ResourceId};
use crate::validation::report::ValidationReport;
use std::collections::{HashMap, HashSet};

pub fn validate_semantic(model: &Model, report: &mut ValidationReport) {
    // Validate build items
    validate_build_references(model, report);

    // Validate boolean operation cycles
    validate_boolean_cycles(model, report);

    // Validate material references and constraints
    validate_material_constraints(model, report);

    // Validate metadata
    validate_metadata(model, report);

    // Check Resources
    for object in model.resources.iter_objects() {
        // Check PID validity
        if let Some(pid) = object.pid {
            // Must exist in base_materials or color_groups or texture_groups
            if !model.resources.exists(pid) {
                report.add_error(
                    2001,
                    format!(
                        "Object {} references non-existent property group {}",
                        object.id.0, pid.0
                    ),
                );
            }
        }

        match &object.geometry {
            Geometry::Mesh(mesh) => {
                for (i, tri) in mesh.triangles.iter().enumerate() {
                    // Check indices bounds
                    if tri.v1 as usize >= mesh.vertices.len()
                        || tri.v2 as usize >= mesh.vertices.len()
                        || tri.v3 as usize >= mesh.vertices.len()
                    {
                        report.add_error(
                            3001,
                            format!(
                                "Triangle {} in Object {} references out-of-bounds vertex",
                                i, object.id.0
                            ),
                        );
                    }

                    // Check PID
                    if let Some(pid) = tri.pid.map(crate::model::ResourceId)
                        && !model.resources.exists(pid)
                    {
                        report.add_error(2002, format!("Triangle {} in Object {} references non-existent property group {}", i, object.id.0, pid.0));
                    }
                }
            }
            Geometry::Components(comps) => {
                for comp in &comps.components {
                    // Only validate internal references (components without external path)
                    if comp.path.is_none() && model.resources.get_object(comp.object_id).is_none() {
                        report.add_error(
                            2003,
                            format!(
                                "Component in Object {} references non-existent object {}",
                                object.id.0, comp.object_id.0
                            ),
                        );
                    }
                }
            }
            Geometry::SliceStack(stack_id) => {
                if model.resources.get_slice_stack(*stack_id).is_none() {
                    report.add_error(
                        2004,
                        format!(
                            "Object {} references non-existent slicestack {}",
                            object.id.0, stack_id.0
                        ),
                    );
                }
            }
            Geometry::VolumetricStack(stack_id) => {
                if model.resources.get_volumetric_stack(*stack_id).is_none() {
                    report.add_error(
                        2005,
                        format!(
                            "Object {} references non-existent volumetricstack {}",
                            object.id.0, stack_id.0
                        ),
                    );
                }
            }
            Geometry::BooleanShape(bs) => {
                // Validate base object exists and is valid type
                if let Some(base_obj) = model.resources.get_object(bs.base_object_id) {
                    // Base can be Mesh or another BooleanShape (per spec)
                    match &base_obj.geometry {
                        Geometry::Mesh(_) | Geometry::BooleanShape(_) => {
                            // Valid base types
                        }
                        Geometry::Components(_) => {
                            report.add_error(
                                2101,
                                format!(
                                    "BooleanShape {} base object {} cannot be Components type",
                                    object.id.0, bs.base_object_id.0
                                ),
                            );
                        }
                        _ => {
                            // Other extensions (SliceStack, VolumetricStack) - allow per spec extensibility
                        }
                    }
                } else {
                    report.add_error(
                        2102,
                        format!(
                            "BooleanShape {} references non-existent base object {}",
                            object.id.0, bs.base_object_id.0
                        ),
                    );
                }

                // Validate each operation object
                for (idx, op) in bs.operations.iter().enumerate() {
                    if let Some(op_obj) = model.resources.get_object(op.object_id) {
                        // Operation objects MUST be triangle meshes (not Components, not BooleanShape)
                        match &op_obj.geometry {
                            Geometry::Mesh(_) => {
                                // Valid - mesh object
                            }
                            _ => {
                                report.add_error(
                                    2103,
                                    format!(
                                        "BooleanShape {} operation {} references non-mesh object {} (type must be mesh)",
                                        object.id.0, idx, op.object_id.0
                                    ),
                                );
                            }
                        }
                    } else {
                        report.add_error(
                            2104,
                            format!(
                                "BooleanShape {} operation {} references non-existent object {}",
                                object.id.0, idx, op.object_id.0
                            ),
                        );
                    }
                }

                // Validate base transformation matrix
                if !is_transform_valid(&bs.base_transform) {
                    report.add_error(
                        2106,
                        format!(
                            "BooleanShape {} has invalid base transformation matrix (contains NaN or Infinity)",
                            object.id.0
                        ),
                    );
                }

                // Validate operation transformation matrices
                for (idx, op) in bs.operations.iter().enumerate() {
                    if !is_transform_valid(&op.transform) {
                        report.add_error(
                            2105,
                            format!(
                                "BooleanShape {} operation {} has invalid transformation matrix (contains NaN or Infinity)",
                                object.id.0, idx
                            ),
                        );
                    }
                }
            }
            Geometry::DisplacementMesh(_mesh) => {
                // Displacement mesh validation will be implemented in displacement.rs
                // For now, just allow it to pass semantic checks
            }
        }
    }
}

fn validate_build_references(model: &Model, report: &mut ValidationReport) {
    for (idx, item) in model.build.items.iter().enumerate() {
        // Check if referenced object exists
        if let Some(obj) = model.resources.get_object(item.object_id) {
            // Check type constraint: Other cannot be in build
            if !obj.object_type.can_be_in_build() {
                report.add_error(
                    3010,
                    format!(
                        "Build item {} references object {} with type '{}' which cannot be in build",
                        idx, item.object_id.0, obj.object_type
                    ),
                );
            }
        } else {
            // Existing check: object must exist
            report.add_error(
                3002,
                format!(
                    "Build item {} references non-existent object {}",
                    idx, item.object_id.0
                ),
            );
        }
    }
}

/// Detects cycles in boolean operation graphs using DFS with recursion stack.
fn validate_boolean_cycles(model: &Model, report: &mut ValidationReport) {
    // Build adjacency list: BooleanShape -> referenced objects
    let mut graph: HashMap<ResourceId, Vec<ResourceId>> = HashMap::new();

    for obj in model.resources.iter_objects() {
        if let Geometry::BooleanShape(bs) = &obj.geometry {
            let mut refs = vec![bs.base_object_id];
            refs.extend(bs.operations.iter().map(|op| op.object_id));
            graph.insert(obj.id, refs);
        }
    }

    // DFS for cycle detection
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for &start_id in graph.keys() {
        if !visited.contains(&start_id)
            && has_cycle_dfs(start_id, &graph, &mut visited, &mut rec_stack)
        {
            report.add_error(
                2100,
                format!(
                    "Cycle detected in boolean operation graph involving object {}",
                    start_id.0
                ),
            );
        }
    }
}

fn has_cycle_dfs(
    node: ResourceId,
    graph: &HashMap<ResourceId, Vec<ResourceId>>,
    visited: &mut HashSet<ResourceId>,
    rec_stack: &mut HashSet<ResourceId>,
) -> bool {
    visited.insert(node);
    rec_stack.insert(node);

    if let Some(neighbors) = graph.get(&node) {
        for &neighbor in neighbors {
            // Only follow edges to other BooleanShape objects (those in the graph)
            if graph.contains_key(&neighbor) {
                if !visited.contains(&neighbor) {
                    if has_cycle_dfs(neighbor, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(&neighbor) {
                    // Back edge found = cycle
                    return true;
                }
            }
        }
    }

    rec_stack.remove(&node);
    false
}

/// Validates that a transformation matrix contains only finite values.
fn is_transform_valid(mat: &glam::Mat4) -> bool {
    mat.x_axis.is_finite()
        && mat.y_axis.is_finite()
        && mat.z_axis.is_finite()
        && mat.w_axis.is_finite()
}

/// Validates material reference constraints and property rules.
fn validate_material_constraints(model: &Model, report: &mut ValidationReport) {
    // Validate pindex requires pid rule
    for object in model.resources.iter_objects() {
        if object.pindex.is_some() && object.pid.is_none() {
            report.add_error(
                2010,
                format!(
                    "Object {} has pindex but no pid (pindex requires pid to be specified)",
                    object.id.0
                ),
            );
        }
    }

    // Validate composite materials matid references basematerials
    for composite in model.resources.iter_composite_materials() {
        // Check that matid references a basematerials group
        if let Some(resource) = model
            .resources
            .get_base_materials(composite.base_material_id)
        {
            // Valid - references basematerials
            let _ = resource; // Use to avoid unused warning
        } else {
            // Check if it references something else (invalid)
            if model.resources.exists(composite.base_material_id) {
                report.add_error(
                    2030,
                    format!(
                        "CompositeMaterials {} matid {} must reference basematerials, not another resource type",
                        composite.id.0, composite.base_material_id.0
                    ),
                );
            } else {
                // Already caught by existing PID validation (2001), but add specific error
                report.add_error(
                    2030,
                    format!(
                        "CompositeMaterials {} matid {} references non-existent basematerials",
                        composite.id.0, composite.base_material_id.0
                    ),
                );
            }
        }
    }

    // Validate multiproperties reference rules
    for multi_prop in model.resources.iter_multi_properties() {
        // Track counts of each resource type referenced
        let mut basematerials_count = 0;
        let mut colorgroup_count = 0;
        let mut texture2dgroup_count = 0;
        let mut composite_count = 0;
        let mut multiproperties_refs = Vec::new();

        for &pid in &multi_prop.pids {
            // Determine what type of resource this pid references
            if model.resources.get_base_materials(pid).is_some() {
                basematerials_count += 1;
            } else if model.resources.get_color_group(pid).is_some() {
                colorgroup_count += 1;
            } else if model.resources.get_texture_2d_group(pid).is_some() {
                texture2dgroup_count += 1;
            } else if model.resources.get_composite_materials(pid).is_some() {
                composite_count += 1;
            } else if model.resources.get_multi_properties(pid).is_some() {
                multiproperties_refs.push(pid);
            }
            // Note: pid might reference other types or be non-existent (caught by other validation)
        }

        // Validate at most one reference to each material type
        if basematerials_count > 1 {
            report.add_error(
                2020,
                format!(
                    "MultiProperties {} references basematerials {} times (maximum 1 allowed)",
                    multi_prop.id.0, basematerials_count
                ),
            );
        }

        if colorgroup_count > 1 {
            report.add_error(
                2021,
                format!(
                    "MultiProperties {} references colorgroup {} times (maximum 1 allowed)",
                    multi_prop.id.0, colorgroup_count
                ),
            );
        }

        if texture2dgroup_count > 1 {
            report.add_error(
                2022,
                format!(
                    "MultiProperties {} references texture2dgroup {} times (maximum 1 allowed)",
                    multi_prop.id.0, texture2dgroup_count
                ),
            );
        }

        if composite_count > 1 {
            report.add_error(
                2023,
                format!(
                    "MultiProperties {} references compositematerials {} times (maximum 1 allowed)",
                    multi_prop.id.0, composite_count
                ),
            );
        }

        // Validate cannot have both basematerials and compositematerials
        // (compositematerials is a material type, not compatible with basematerials)
        if basematerials_count > 0 && composite_count > 0 {
            report.add_error(
                2025,
                format!(
                    "MultiProperties {} references both basematerials and compositematerials (only one material type allowed)",
                    multi_prop.id.0
                ),
            );
        }

        // Validate no references to other multiproperties
        for &ref_id in &multiproperties_refs {
            report.add_error(
                2024,
                format!(
                    "MultiProperties {} references another multiproperties {} (not allowed)",
                    multi_prop.id.0, ref_id.0
                ),
            );
        }
    }
}

/// Validates metadata constraints.
fn validate_metadata(model: &Model, report: &mut ValidationReport) {
    let mut seen_names = HashSet::new();

    for name in model.metadata.keys() {
        // Check for empty names
        if name.is_empty() {
            report.add_error(
                2040,
                "Metadata entry has empty name (name attribute is required)".to_string(),
            );
        }

        // Check for duplicate names
        if !seen_names.insert(name.clone()) {
            report.add_error(
                2041,
                format!(
                    "Metadata name '{}' is duplicated (names must be unique)",
                    name
                ),
            );
        }
    }
}
