use crate::model::{Geometry, Model};
use crate::validation::report::ValidationReport;

pub fn validate_semantic(model: &Model, report: &mut ValidationReport) {
    // Check Resources
    for object in model.resources.iter_objects() {
        // Check PID validity
        if let Some(pid) = object.pid {
            // Must exist in base_materials or color_groups or texture_groups
            if !model.resources.exists(pid) {
                 report.add_error(2001, format!("Object {} references non-existent property group {}", object.id.0, pid.0));
            }
        }
        
        match &object.geometry {
            Geometry::Mesh(mesh) => {
                for (i, tri) in mesh.triangles.iter().enumerate() {
                    // Check indices bounds
                    if tri.v1 as usize >= mesh.vertices.len() ||
                       tri.v2 as usize >= mesh.vertices.len() || 
                       tri.v3 as usize >= mesh.vertices.len() {
                           report.add_error(3001, format!("Triangle {} in Object {} references out-of-bounds vertex", i, object.id.0));
                    }
                    
                    // Check PID
                    if let Some(pid) = tri.pid.map(crate::model::ResourceId) {
                         if !model.resources.exists(pid) {
                            report.add_error(2002, format!("Triangle {} in Object {} references non-existent property group {}", i, object.id.0, pid.0));
                        }
                    }
                }
            }
            Geometry::Components(comps) => {
                for comp in &comps.components {
                    if model.resources.get_object(comp.object_id).is_none() {
                        report.add_error(2003, format!("Component in Object {} references non-existent object {}", object.id.0, comp.object_id.0));
                    }
                }
            }
            Geometry::SliceStack(stack_id) => {
                if model.resources.get_slice_stack(*stack_id).is_none() {
                     report.add_error(2004, format!("Object {} references non-existent slicestack {}", object.id.0, stack_id.0));
                }
            }
            Geometry::VolumetricStack(stack_id) => {
                if model.resources.get_volumetric_stack(*stack_id).is_none() {
                     report.add_error(2005, format!("Object {} references non-existent volumetricstack {}", object.id.0, stack_id.0));
                }
            }
        }
    }
}
