#![no_main]
use libfuzzer_sys::fuzz_target;
use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Attempt to treat random data as a ZIP archive
    let cursor = Cursor::new(data);
    if let Ok(mut archiver) = ZipArchiver::new(cursor) {
        // If it's a valid zip, try to find the 3D model path
        if let Ok(path) = find_model_path(&mut archiver) {
             // If found, try to read it
             if let Ok(model_data) = archiver.read_entry(&path) {
                  // Finally, fuzz the XML parser with the extracted data
                  let _ = parse_model(Cursor::new(model_data));
             }
        }
    }
});
