use criterion::{Criterion, criterion_group, criterion_main};
use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path};
use lib3mf_core::parser::parse_model;

use std::io::Cursor;
use std::path::PathBuf;

fn bench_parse_benchy(c: &mut Criterion) {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop(); // crates
    d.pop(); // lib3mf-rs
    d.push("models");
    d.push("Benchy.3mf");

    if !d.exists() {
        return; // Skip if file not found
    }

    // Pre-load file into memory to avoid I/O noise in benchmark
    let file_bytes = std::fs::read(&d).expect("Failed to read Benchy.3mf");

    c.bench_function("parse_benchy_full", |b| {
        b.iter(|| {
            let cursor = Cursor::new(&file_bytes);
            let mut archiver = ZipArchiver::new(cursor).unwrap();
            let model_path = find_model_path(&mut archiver).unwrap();
            let model_xml = archiver.read_entry(&model_path).unwrap();
            let _model = parse_model(Cursor::new(model_xml)).unwrap();
        })
    });
}

criterion_group!(benches, bench_parse_benchy);
criterion_main!(benches);
