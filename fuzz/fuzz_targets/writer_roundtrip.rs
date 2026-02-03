#![no_main]
use libfuzzer_sys::fuzz_target;
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Fuzz writer by parsing, writing, and re-parsing
    // Checks write/read symmetry and writer robustness
    if let Ok(model1) = parse_model(Cursor::new(data)) {
        let mut output = Vec::new();
        if model1.write_xml(&mut output, None).is_ok() {
            // Re-parse written output
            if let Ok(model2) = parse_model(Cursor::new(&output)) {
                // Basic invariant: object counts should match
                assert_eq!(
                    model1.resources.iter_objects().count(),
                    model2.resources.iter_objects().count(),
                    "Object count mismatch after roundtrip"
                );
            }
        }
    }
});
