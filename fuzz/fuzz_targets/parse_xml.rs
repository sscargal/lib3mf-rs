#![no_main]
use libfuzzer_sys::fuzz_target;
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Directly fuzz XML parser, bypassing ZIP layer
    // This finds XML-specific bugs faster than parse_model
    let _ = parse_model(Cursor::new(data));
});
