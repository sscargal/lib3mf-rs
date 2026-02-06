use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lib3mf_core::archive::{find_model_path, ArchiveReader, ZipArchiver};
use lib3mf_core::parser::parse_model;
use lib3mf_core::validation::ValidationLevel;
use lib3mf_core::Model;
use std::io::Cursor;

/// Get test data for different file size categories
fn get_test_file(size_category: &str) -> Vec<u8> {
    match size_category {
        "small" => {
            // Small file: box.3mf (~1.2 KB)
            std::fs::read("tests/conformance/3mf-samples/examples/core/box.3mf")
                .expect("Failed to read small test file. Run: git submodule update --init")
        }
        "medium" => {
            // Medium file: cube_gears.3mf (~258 KB)
            std::fs::read("tests/conformance/3mf-samples/examples/core/cube_gears.3mf")
                .expect("Failed to read medium test file. Run: git submodule update --init")
        }
        "large" => {
            // Large file: Benchy.3mf (~3.1 MB)
            std::fs::read("models/Benchy.3mf")
                .expect("Failed to read large test file (Benchy.3mf)")
        }
        _ => panic!("Unknown size category: {}", size_category),
    }
}

/// Parse a 3MF file completely (unzip + parse XML)
fn parse_complete(data: &[u8]) -> Model {
    let mut archiver = ZipArchiver::new(Cursor::new(data)).expect("Failed to open archive");
    let model_path = find_model_path(&mut archiver).expect("Failed to find model path");
    let model_data = archiver.read_entry(&model_path).expect("Failed to read model entry");
    parse_model(Cursor::new(model_data)).expect("Failed to parse model")
}

/// Benchmark: Parse speed for different file sizes
fn bench_parse_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_speed");

    for size in ["small", "medium", "large"].iter() {
        let data = get_test_file(size);
        let bytes = data.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::new("full_parse", size), &data, |b, data| {
            b.iter(|| {
                let model = parse_complete(black_box(data));
                black_box(model);
            })
        });
    }

    group.finish();
}

/// Benchmark: Validation at different levels
fn bench_validation_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation_levels");

    // Use medium-sized file for validation benchmarks
    let data = get_test_file("medium");
    let model = parse_complete(&data);

    let levels = [
        ("minimal", ValidationLevel::Minimal),
        ("standard", ValidationLevel::Standard),
        ("strict", ValidationLevel::Strict),
        ("paranoid", ValidationLevel::Paranoid),
    ];

    for (name, level) in levels.iter() {
        group.bench_with_input(BenchmarkId::new("validate", name), level, |b, level| {
            b.iter(|| {
                let report = black_box(&model).validate(*level);
                black_box(report);
            })
        });
    }

    group.finish();
}

/// Benchmark: Statistics computation
fn bench_statistics(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics");

    for size in ["small", "medium"].iter() {
        let data = get_test_file(size);
        let model = parse_complete(&data);

        group.bench_with_input(BenchmarkId::new("compute_stats", size), &data, |b, data| {
            b.iter(|| {
                let mut archiver = ZipArchiver::new(Cursor::new(black_box(data)))
                    .expect("Failed to open archive");
                let stats = black_box(&model)
                    .compute_stats(&mut archiver)
                    .expect("Failed to compute stats");
                black_box(stats);
            })
        });
    }

    group.finish();
}

/// Benchmark: Archive operations (ZIP unzip + OPC parsing)
fn bench_archive_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_ops");

    for size in ["small", "medium", "large"].iter() {
        let data = get_test_file(size);
        let bytes = data.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::new("unzip_and_find", size), &data, |b, data| {
            b.iter(|| {
                let mut archiver = ZipArchiver::new(Cursor::new(black_box(data)))
                    .expect("Failed to open archive");
                let model_path = find_model_path(&mut archiver).expect("Failed to find model");
                black_box(model_path);
            })
        });
    }

    group.finish();
}

/// Benchmark: XML parsing only (excludes ZIP overhead)
fn bench_xml_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("xml_parsing");

    for size in ["small", "medium"].iter() {
        let data = get_test_file(size);

        // Pre-extract the XML from the ZIP
        let mut archiver = ZipArchiver::new(Cursor::new(&data)).expect("Failed to open archive");
        let model_path = find_model_path(&mut archiver).expect("Failed to find model path");
        let model_xml = archiver.read_entry(&model_path).expect("Failed to read model");

        let xml_bytes = model_xml.len() as u64;
        group.throughput(Throughput::Bytes(xml_bytes));

        group.bench_with_input(
            BenchmarkId::new("parse_xml_only", size),
            &model_xml,
            |b, xml| {
                b.iter(|| {
                    let model = parse_model(Cursor::new(black_box(xml)))
                        .expect("Failed to parse XML");
                    black_box(model);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Memory usage patterns (iteration vs. random access)
fn bench_memory_access_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_access");

    let data = get_test_file("medium");
    let model = parse_complete(&data);

    group.bench_function("iterate_all_objects", |b| {
        b.iter(|| {
            for obj in black_box(&model).resources.iter_objects() {
                black_box(obj);
            }
        })
    });

    group.bench_function("iterate_build_items", |b| {
        b.iter(|| {
            for item in &black_box(&model).build.items {
                black_box(item);
            }
        })
    });

    // Access pattern: random lookup by ID
    group.bench_function("lookup_objects_by_id", |b| {
        let object_ids: Vec<_> = model.resources.iter_objects().map(|o| o.id).collect();
        b.iter(|| {
            for id in &object_ids {
                let obj = black_box(&model).resources.get_object(*id);
                black_box(obj);
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_speed,
    bench_validation_levels,
    bench_statistics,
    bench_archive_operations,
    bench_xml_parsing,
    bench_memory_access_patterns
);
criterion_main!(benches);
