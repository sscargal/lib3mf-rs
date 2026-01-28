use crate::model::{Geometry, Model};
use crate::validation::report::ValidationReport;

pub fn validate_geometry(model: &Model, report: &mut ValidationReport) {
    for object in model.resources.iter_objects() {
        if let Geometry::Mesh(mesh) = &object.geometry {
            for (i, tri) in mesh.triangles.iter().enumerate() {
                // Degenerate check: duplicate indices
                if tri.v1 == tri.v2 || tri.v2 == tri.v3 || tri.v1 == tri.v3 {
                    report.add_warning(4001, format!("Triangle {} in Object {} is degenerate (duplicate vertices)", i, object.id.0));
                }
            }
        }
    }
}
