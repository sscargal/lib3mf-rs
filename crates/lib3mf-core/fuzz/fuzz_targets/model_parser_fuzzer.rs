#![no_main]

use libfuzzer_sys::fuzz_target;
use lib3mf_core::parser::model_parser::parse_model;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // We try to parse any random byte string as a 3MF model.
    // The goal is to ensure the parser never panics on malformed input.
    let _ = parse_model(Cursor::new(data));
});
