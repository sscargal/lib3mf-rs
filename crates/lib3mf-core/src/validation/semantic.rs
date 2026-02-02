use crate::model::{Geometry, Model};
use crate::validation::report::ValidationReport;

pub fn validate_semantic(model: &Model, report: &mut ValidationReport) {
    // Validate build items
    validate_build_references(model, report);

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
            Geometry::BooleanShape(bool_shape) => {
                // Validate base object reference
                if model.resources.get_object(bool_shape.base_object_id).is_none() {
                    report.add_error(
                        2006,
                        format!(
                            "BooleanShape in Object {} references non-existent base object {}",
                            object.id.0, bool_shape.base_object_id.0
                        ),
                    );
                }
                // Validate operation object references
                for (i, op) in bool_shape.operations.iter().enumerate() {
                    if model.resources.get_object(op.object_id).is_none() {
                        report.add_error(
                            2007,
                            format!(
                                "BooleanShape operation {} in Object {} references non-existent object {}",
                                i, object.id.0, op.object_id.0
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
