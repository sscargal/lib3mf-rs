#![no_main]
use libfuzzer_sys::fuzz_target;
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Fuzz extension parsing: beamlattice, slice, boolean, displacement, volumetric
    // Extensions are parsed within model parsing, so we use parse_model
    // but specifically check extension data was reached
    if let Ok(model) = parse_model(Cursor::new(data)) {
        // Touch extension data to confirm parsing occurred
        for obj in model.resources.iter_objects() {
            // Check for extension geometries
            match &obj.geometry {
                lib3mf_core::model::Geometry::Mesh(mesh) => {
                    // Check for beam lattice extension
                    let _ = mesh.beam_lattice.as_ref();
                }
                lib3mf_core::model::Geometry::BooleanShape(_) => {
                    // Boolean shape detected
                }
                lib3mf_core::model::Geometry::SliceStack(_) => {
                    // Slice stack detected
                }
                lib3mf_core::model::Geometry::VolumetricStack(_) => {
                    // Volumetric stack detected
                }
                _ => {}
            }
        }
        // Check displacement resources
        let _ = model.resources.displacement_2d_count();
    }
});
