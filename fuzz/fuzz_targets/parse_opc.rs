#![no_main]
use libfuzzer_sys::fuzz_target;
use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
use lib3mf_core::archive::opc::{parse_relationships, parse_content_types};
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Fuzz OPC (Open Packaging Convention) parsing
    // This targets relationship and content type XML parsing
    let cursor = Cursor::new(data);
    if let Ok(mut archiver) = ZipArchiver::new(cursor) {
        // Fuzz relationship parsing
        if let Ok(rels_data) = archiver.read_entry("_rels/.rels") {
            let _ = parse_relationships(&rels_data);
        }

        // Fuzz content types parsing
        if let Ok(ct_data) = archiver.read_entry("[Content_Types].xml") {
            let _ = parse_content_types(&ct_data);
        }

        // Also try model locator
        let _ = find_model_path(&mut archiver);
    }
});
