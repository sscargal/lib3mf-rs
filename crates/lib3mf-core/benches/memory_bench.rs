use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path};
use lib3mf_core::error::Result;
use lib3mf_core::model::{BaseMaterialsGroup, BuildItem, ColorGroup, ResourceId};
use lib3mf_core::parser::model_parser::parse_model;
use lib3mf_core::parser::streaming::parse_model_streaming;
use lib3mf_core::parser::visitor::ModelVisitor;
use peakmem_alloc::{PeakMemAlloc, PeakMemAllocTrait};
use std::alloc::System;
use std::io::{BufReader, Cursor};

// ---------------------------------------------------------------------------
// Global peak-tracking allocator
// ---------------------------------------------------------------------------

#[global_allocator]
static GLOBAL: PeakMemAlloc<System> = PeakMemAlloc::new(System);

// ---------------------------------------------------------------------------
// File helpers (duplicated from core_bench.rs — separate binary)
// ---------------------------------------------------------------------------

fn get_test_file(name: &str) -> Vec<u8> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    match name {
        "medium" => std::fs::read(
            workspace_root
                .join("tests/conformance/3mf-samples/examples/core/cube_gears.3mf"),
        )
        .expect("Failed to read medium test file. Run: git submodule update --init"),
        "large" => std::fs::read(workspace_root.join("models/Benchy.3mf"))
            .expect("Failed to read large test file (Benchy.3mf)"),
        _ => panic!("Unknown size category: {}", name),
    }
}

fn parse_complete(data: &[u8]) -> lib3mf_core::Model {
    let mut archiver = ZipArchiver::new(Cursor::new(data)).expect("Failed to open archive");
    let model_path = find_model_path(&mut archiver).expect("Failed to find model path");
    let model_data = archiver
        .read_entry(&model_path)
        .expect("Failed to read model entry");
    parse_model(Cursor::new(model_data)).expect("Failed to parse model")
}

fn extract_model_xml(data: &[u8]) -> Vec<u8> {
    let mut archiver = ZipArchiver::new(Cursor::new(data)).expect("Failed to open archive");
    let model_path = find_model_path(&mut archiver).expect("Failed to find model path");
    archiver
        .read_entry(&model_path)
        .expect("Failed to read model entry")
}

// ---------------------------------------------------------------------------
// Counting visitor for streaming benchmarks
// ---------------------------------------------------------------------------

struct CountingVisitor {
    pub vertices: u64,
    pub triangles: u64,
}

impl CountingVisitor {
    fn new() -> Self {
        Self {
            vertices: 0,
            triangles: 0,
        }
    }
}

impl ModelVisitor for CountingVisitor {
    fn on_vertex(&mut self, _x: f32, _y: f32, _z: f32) -> Result<()> {
        self.vertices += 1;
        Ok(())
    }

    fn on_triangle(&mut self, _v1: u32, _v2: u32, _v3: u32) -> Result<()> {
        self.triangles += 1;
        Ok(())
    }

    fn on_base_materials(
        &mut self,
        _id: ResourceId,
        _group: &BaseMaterialsGroup,
    ) -> Result<()> {
        Ok(())
    }

    fn on_color_group(&mut self, _id: ResourceId, _group: &ColorGroup) -> Result<()> {
        Ok(())
    }

    fn on_build_item(&mut self, _item: &BuildItem) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Memory benchmarks
// ---------------------------------------------------------------------------

/// Benchmark peak heap allocation for full DOM parse of cube_gears.3mf (medium).
fn bench_memory_parse_full_medium(c: &mut Criterion) {
    let data = get_test_file("medium");

    c.bench_with_input(
        BenchmarkId::new("memory", "parse_full_medium"),
        &data,
        |b, data| {
            b.iter(|| {
                GLOBAL.reset_peak_memory();
                let model = parse_complete(black_box(data));
                let peak_bytes = GLOBAL.get_peak_memory();
                black_box(model);
                black_box(peak_bytes);
            });
        },
    );
}

/// Benchmark peak heap allocation for full DOM parse of Benchy.3mf (large).
fn bench_memory_parse_full_large(c: &mut Criterion) {
    let data = get_test_file("large");

    c.bench_with_input(
        BenchmarkId::new("memory", "parse_full_large"),
        &data,
        |b, data| {
            b.iter(|| {
                GLOBAL.reset_peak_memory();
                let model = parse_complete(black_box(data));
                let peak_bytes = GLOBAL.get_peak_memory();
                black_box(model);
                black_box(peak_bytes);
            });
        },
    );
}

/// Benchmark peak heap allocation for streaming parse of Benchy.3mf (large).
fn bench_memory_streaming_large(c: &mut Criterion) {
    let data = get_test_file("large");
    let xml = extract_model_xml(&data);

    c.bench_with_input(
        BenchmarkId::new("memory", "streaming_large"),
        &xml,
        |b, xml| {
            b.iter(|| {
                GLOBAL.reset_peak_memory();
                let reader = BufReader::new(Cursor::new(black_box(xml.as_slice())));
                let mut visitor = CountingVisitor::new();
                parse_model_streaming(reader, &mut visitor).expect("streaming parse failed");
                let peak_bytes = GLOBAL.get_peak_memory();
                black_box((visitor.vertices, visitor.triangles));
                black_box(peak_bytes);
            });
        },
    );
}

// ---------------------------------------------------------------------------
// Criterion boilerplate
// ---------------------------------------------------------------------------

criterion_group!(
    memory_benches,
    bench_memory_parse_full_medium,
    bench_memory_parse_full_large,
    bench_memory_streaming_large,
);
criterion_main!(memory_benches);
