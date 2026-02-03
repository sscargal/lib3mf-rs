#![no_main]
use libfuzzer_sys::fuzz_target;
use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Attempt to treat random data as a ZIP archive containing a 3MF model
    let cursor = Cursor::new(data);
    if let Ok(mut archiver) = ZipArchiver::new(cursor) {
        // If it's a valid zip, try to find the 3D model path
        if let Ok(path) = find_model_path(&mut archiver) {
            // If found, try to read and parse the model
            if let Ok(model_data) = archiver.read_entry(&path) {
                if let Ok(model) = parse_model(Cursor::new(model_data)) {
                    // INVARIANT 1: All build items reference valid objects
                    for item in &model.build.items {
                        assert!(
                            model.resources.get_object(item.object_id).is_some(),
                            "Build item references non-existent object ID {:?}",
                            item.object_id
                        );
                    }

                    // INVARIANT 2: Triangle vertex indices are in bounds
                    for obj in model.resources.iter_objects() {
                        match &obj.geometry {
                            lib3mf_core::model::Geometry::Mesh(mesh) => {
                                let vertex_count = mesh.vertices.len();
                                for (tri_idx, tri) in mesh.triangles.iter().enumerate() {
                                    assert!(
                                        (tri.v1 as usize) < vertex_count
                                        && (tri.v2 as usize) < vertex_count
                                        && (tri.v3 as usize) < vertex_count,
                                        "Object {:?} triangle {} has out-of-bounds vertex index",
                                        obj.id, tri_idx
                                    );
                                }
                            }
                            _ => {}
                        }
                    }

                    // INVARIANT 3: Component references are valid
                    for obj in model.resources.iter_objects() {
                        match &obj.geometry {
                            lib3mf_core::model::Geometry::Components(components) => {
                                for comp in &components.components {
                                    assert!(
                                        model.resources.get_object(comp.object_id).is_some(),
                                        "Object {:?} component references non-existent object {:?}",
                                        obj.id, comp.object_id
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
});
