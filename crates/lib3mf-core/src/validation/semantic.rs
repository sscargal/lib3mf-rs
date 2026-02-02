use crate::model::{Geometry, Model, ResourceId};
use crate::validation::report::ValidationReport;
use std::collections::{HashMap, HashSet};

pub fn validate_semantic(model: &Model, report: &mut ValidationReport) {
    // Validate build items
    validate_build_references(model, report);

    // Validate boolean operation cycles
    validate_boolean_cycles(model, report);

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
                    if model.resources.get_object(comp.object_id).is_none() {
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
        if !visited.contains(&start_id) {
            if has_cycle_dfs(start_id, &graph, &mut visited, &mut rec_stack) {
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
