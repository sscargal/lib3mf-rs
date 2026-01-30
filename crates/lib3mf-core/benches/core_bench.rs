use criterion::{Criterion, criterion_group, criterion_main};
use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path};
use lib3mf_core::parser::parse_model;

use std::io::Cursor;
use std::path::PathBuf;

fn bench_core(c: &mut Criterion) {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop(); // crates
    d.pop(); // lib3mf-rs
    let repo_root = d.clone();

    let benchy_path = repo_root.join("models/Benchy.3mf");
    let deadpool_path = repo_root.join("models/Deadpool_3_Mask.3mf");

    // --- Benchy Benchmarks ---
    if benchy_path.exists() {
        let file_bytes = std::fs::read(&benchy_path).expect("Failed to read Benchy.3mf");

        // Full workflow (Zip + XML)
        c.bench_function("parse_benchy_zip_xml", |b| {
            b.iter(|| {
                let mut archiver = ZipArchiver::new(Cursor::new(&file_bytes)).unwrap();
                let model_path = find_model_path(&mut archiver).unwrap();
                let model_xml = archiver.read_entry(&model_path).unwrap();
                let _model = parse_model(Cursor::new(model_xml)).unwrap();
            })
        });

        // XML parsing only
        let mut archiver = ZipArchiver::new(Cursor::new(&file_bytes)).unwrap();
        let model_path = find_model_path(&mut archiver).unwrap();
        let model_xml = archiver.read_entry(&model_path).unwrap();
        c.bench_function("parse_benchy_xml_only", |b| {
            b.iter(|| {
                let _model = parse_model(Cursor::new(&model_xml)).unwrap();
            })
        });

        // Stats calculation
        let model = parse_model(Cursor::new(&model_xml)).unwrap();
        c.bench_function("compute_stats_benchy", |b| {
            b.iter(|| {
                let mut archiver = ZipArchiver::new(Cursor::new(&file_bytes)).unwrap();
                let _stats = model.compute_stats(&mut archiver).unwrap();
            })
        });
    }

    // --- Deadpool Benchmarks (Larger) ---
    if deadpool_path.exists() {
        let file_bytes = std::fs::read(&deadpool_path).expect("Failed to read Deadpool");

        c.bench_function("parse_deadpool_zip_xml", |b| {
            b.iter(|| {
                let mut archiver = ZipArchiver::new(Cursor::new(&file_bytes)).unwrap();
                let model_path = find_model_path(&mut archiver).unwrap();
                let model_xml = archiver.read_entry(&model_path).unwrap();
                let _model = parse_model(Cursor::new(model_xml)).unwrap();
            })
        });
    }
}

criterion_group!(benches, bench_core);
criterion_main!(benches);
