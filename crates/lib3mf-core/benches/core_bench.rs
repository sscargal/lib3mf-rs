use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use lib3mf_core::Model;
use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path};
use lib3mf_core::error::Result;
use lib3mf_core::model::{BaseMaterialsGroup, BuildItem, ColorGroup, ResourceId};
use lib3mf_core::parser::model_parser::parse_model;
use lib3mf_core::parser::streaming::parse_model_streaming;
use lib3mf_core::parser::visitor::ModelVisitor;
use lib3mf_core::validation::ValidationLevel;
use std::io::{BufReader, Cursor};
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// File helpers
// ---------------------------------------------------------------------------

fn get_test_file(name: &str) -> Vec<u8> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    match name {
        "small" => std::fs::read(
            workspace_root.join("tests/conformance/3mf-samples/examples/core/box.3mf"),
        )
        .expect("Failed to read small test file. Run: git submodule update --init"),
        "medium" => std::fs::read(
            workspace_root.join("tests/conformance/3mf-samples/examples/core/cube_gears.3mf"),
        )
        .expect("Failed to read medium test file. Run: git submodule update --init"),
        "large" => std::fs::read(workspace_root.join("models/Benchy.3mf"))
            .expect("Failed to read large test file (Benchy.3mf)"),
        _ => panic!("Unknown size category: {}", name),
    }
}

/// Full parse: open ZIP, find model path, read entry, parse XML.
fn parse_complete(data: &[u8]) -> Model {
    let mut archiver = ZipArchiver::new(Cursor::new(data)).expect("Failed to open archive");
    let model_path = find_model_path(&mut archiver).expect("Failed to find model path");
    let model_data = archiver
        .read_entry(&model_path)
        .expect("Failed to read model entry");
    parse_model(Cursor::new(model_data)).expect("Failed to parse model")
}

/// Extract raw model XML bytes from a 3MF archive (excludes archive overhead).
fn extract_model_xml(data: &[u8]) -> Vec<u8> {
    let mut archiver = ZipArchiver::new(Cursor::new(data)).expect("Failed to open archive");
    let model_path = find_model_path(&mut archiver).expect("Failed to find model path");
    archiver
        .read_entry(&model_path)
        .expect("Failed to read model entry")
}

// ---------------------------------------------------------------------------
// Synthetic mesh generator
// ---------------------------------------------------------------------------

/// Generates a flat triangle grid XML (3MF core XML) with approximately
/// `approx_triangles` triangles. Each grid cell has 2 triangles and 4 vertices
/// (minus shared edges). We use a full vertex-per-triangle approach to keep
/// the generator simple: 3 vertices per triangle, no sharing.
fn generate_triangle_grid_xml(approx_triangles: usize) -> Vec<u8> {
    // We'll emit n*n grid cells of 2 triangles each, so n = sqrt(approx_triangles/2).
    let n = ((approx_triangles / 2) as f64).sqrt().max(1.0) as usize;
    let actual_triangles = n * n * 2;

    let mut xml = String::with_capacity(actual_triangles * 120);
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
"#);

    // Emit one quad (4 vertices) per cell: each cell has unique vertices.
    // Vertex index: cell(i,j) => base_idx = (i*n + j) * 4
    for i in 0..n {
        for j in 0..n {
            let x0 = i as f32;
            let x1 = (i + 1) as f32;
            let y0 = j as f32;
            let y1 = (j + 1) as f32;
            xml.push_str(&format!(
                "          <vertex x=\"{x0}\" y=\"{y0}\" z=\"0\" />\n"
            ));
            xml.push_str(&format!(
                "          <vertex x=\"{x1}\" y=\"{y0}\" z=\"0\" />\n"
            ));
            xml.push_str(&format!(
                "          <vertex x=\"{x1}\" y=\"{y1}\" z=\"0\" />\n"
            ));
            xml.push_str(&format!(
                "          <vertex x=\"{x0}\" y=\"{y1}\" z=\"0\" />\n"
            ));
        }
    }

    xml.push_str("        </vertices>\n        <triangles>\n");

    for i in 0..n {
        for j in 0..n {
            let base = ((i * n + j) * 4) as u32;
            // Triangle 1: v0, v1, v2
            xml.push_str(&format!(
                "          <triangle v1=\"{b}\" v2=\"{v1}\" v3=\"{v2}\" />\n",
                b = base,
                v1 = base + 1,
                v2 = base + 2
            ));
            // Triangle 2: v0, v2, v3
            xml.push_str(&format!(
                "          <triangle v1=\"{b}\" v2=\"{v2}\" v3=\"{v3}\" />\n",
                b = base,
                v2 = base + 2,
                v3 = base + 3
            ));
        }
    }

    xml.push_str("        </triangles>\n      </mesh>\n    </object>\n  </resources>\n  <build>\n    <item objectid=\"1\" />\n  </build>\n</model>");

    xml.into_bytes()
}

// OnceLock caches for synthetic meshes (avoid regenerating on every bench call)
static GRID_1K: OnceLock<Vec<u8>> = OnceLock::new();
static GRID_10K: OnceLock<Vec<u8>> = OnceLock::new();

fn get_grid_xml(approx: usize) -> &'static Vec<u8> {
    match approx {
        1_000 => GRID_1K.get_or_init(|| generate_triangle_grid_xml(1_000)),
        10_000 => GRID_10K.get_or_init(|| generate_triangle_grid_xml(10_000)),
        _ => panic!("Unsupported grid size"),
    }
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

    fn on_base_materials(&mut self, _id: ResourceId, _group: &BaseMaterialsGroup) -> Result<()> {
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
// Benchmark: parse_xml (XML-only parse, no archive overhead)
// ---------------------------------------------------------------------------

fn bench_parse_xml(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_xml");

    for size in ["small", "medium"] {
        let file_bytes = get_test_file(size);
        let xml = extract_model_xml(&file_bytes);
        let xml_len = xml.len() as u64;

        group.throughput(Throughput::Bytes(xml_len));
        group.bench_with_input(BenchmarkId::new("parse_xml", size), &xml, |b, xml| {
            b.iter(|| {
                let model = parse_model(Cursor::new(black_box(xml.as_slice())))
                    .expect("parse_model failed");
                black_box(model);
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: parse_full (ZIP + OPC + XML)
// ---------------------------------------------------------------------------

fn bench_parse_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_full");

    for size in ["small", "medium", "large"] {
        let data = get_test_file(size);
        let bytes = data.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::new("parse_full", size), &data, |b, data| {
            b.iter(|| {
                let model = parse_complete(black_box(data));
                black_box(model);
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: parse_synthetic (programmatic mesh scaling)
// ---------------------------------------------------------------------------

fn bench_parse_synthetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_synthetic");

    for approx in [1_000usize, 10_000] {
        let xml = get_grid_xml(approx);
        let xml_len = xml.len() as u64;
        let label = format!("{}k_triangles", approx / 1_000);

        group.throughput(Throughput::Bytes(xml_len));
        group.bench_with_input(
            BenchmarkId::new("parse_synthetic", &label),
            xml,
            |b, xml| {
                b.iter(|| {
                    let model = parse_model(Cursor::new(black_box(xml.as_slice())))
                        .expect("parse_model failed");
                    black_box(model);
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: write_xml (serialize model to XML bytes)
// ---------------------------------------------------------------------------

fn bench_write_xml(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_xml");

    for size in ["small", "medium"] {
        let data = get_test_file(size);
        let model = parse_complete(&data);

        // Measure output size by running once outside the loop
        let mut sample_out = Vec::new();
        model
            .write_xml(&mut sample_out, None)
            .expect("write_xml failed");
        let output_bytes = sample_out.len() as u64;

        group.throughput(Throughput::Bytes(output_bytes));
        group.bench_with_input(BenchmarkId::new("write_xml", size), &model, |b, model| {
            b.iter(|| {
                let mut out = Vec::with_capacity(output_bytes as usize);
                black_box(model)
                    .write_xml(&mut out, None)
                    .expect("write_xml failed");
                black_box(out);
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: write_package (full ZIP package write)
// ---------------------------------------------------------------------------

fn bench_write_package(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_package");

    let data = get_test_file("medium");
    let model = parse_complete(&data);

    // Measure output size once
    let mut sample_out = Cursor::new(Vec::new());
    model.write(&mut sample_out).expect("write failed");
    let output_bytes = sample_out.into_inner().len() as u64;

    group.throughput(Throughput::Bytes(output_bytes));
    group.bench_with_input(
        BenchmarkId::new("write_package", "medium"),
        &model,
        |b, model| {
            b.iter(|| {
                let buf = Vec::with_capacity(output_bytes as usize);
                let writer = Cursor::new(buf);
                black_box(model).write(writer).expect("write failed");
            });
        },
    );

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: streaming (SAX parse vs DOM)
// ---------------------------------------------------------------------------

fn bench_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming");

    // streaming/parse_medium -- cube_gears XML
    {
        let data = get_test_file("medium");
        let xml = extract_model_xml(&data);
        let xml_len = xml.len() as u64;

        group.throughput(Throughput::Bytes(xml_len));
        group.bench_with_input(
            BenchmarkId::new("streaming", "parse_medium"),
            &xml,
            |b, xml| {
                b.iter(|| {
                    let reader = BufReader::new(Cursor::new(black_box(xml.as_slice())));
                    let mut visitor = CountingVisitor::new();
                    parse_model_streaming(reader, &mut visitor).expect("streaming parse failed");
                    black_box((visitor.vertices, visitor.triangles));
                });
            },
        );
    }

    // streaming/parse_large -- Benchy XML
    {
        let data = get_test_file("large");
        let xml = extract_model_xml(&data);
        let xml_len = xml.len() as u64;

        group.throughput(Throughput::Bytes(xml_len));
        group.bench_with_input(
            BenchmarkId::new("streaming", "parse_large"),
            &xml,
            |b, xml| {
                b.iter(|| {
                    let reader = BufReader::new(Cursor::new(black_box(xml.as_slice())));
                    let mut visitor = CountingVisitor::new();
                    parse_model_streaming(reader, &mut visitor).expect("streaming parse failed");
                    black_box((visitor.vertices, visitor.triangles));
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: validation (different levels)
// ---------------------------------------------------------------------------

fn bench_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation");

    let data = get_test_file("medium");
    let model = parse_complete(&data);

    group.bench_with_input(
        BenchmarkId::new("validation", "minimal_medium"),
        &model,
        |b, model| {
            b.iter(|| {
                let report = black_box(model).validate(ValidationLevel::Minimal);
                black_box(report);
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("validation", "paranoid_medium"),
        &model,
        |b, model| {
            b.iter(|| {
                let report = black_box(model).validate(ValidationLevel::Paranoid);
                black_box(report);
            });
        },
    );

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion boilerplate
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_parse_xml,
    bench_parse_full,
    bench_parse_synthetic,
    bench_write_xml,
    bench_write_package,
    bench_streaming,
    bench_validation,
);
criterion_main!(benches);
