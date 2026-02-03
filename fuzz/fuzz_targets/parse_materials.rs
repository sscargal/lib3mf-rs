#![no_main]
use libfuzzer_sys::fuzz_target;
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Fuzz material parsing - focus on color groups, textures, composites
    // XML must be well-formed to reach material parser
    if let Ok(model) = parse_model(Cursor::new(data)) {
        // Touch material data to ensure it was parsed
        let _ = model.resources.base_material_groups_count();
        let _ = model.resources.color_groups_count();
        let _ = model.resources.texture_2d_groups_count();
    }
});
