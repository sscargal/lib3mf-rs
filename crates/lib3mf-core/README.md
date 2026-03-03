# lib3mf-core

[![Crates.io](https://img.shields.io/crates/v/lib3mf-core.svg)](https://crates.io/crates/lib3mf-core)
[![docs.rs](https://docs.rs/lib3mf-core/badge.svg)](https://docs.rs/lib3mf-core)
[![Downloads](https://img.shields.io/crates/d/lib3mf-core.svg)](https://crates.io/crates/lib3mf-core)
[![CI](https://github.com/sscargal/lib3mf-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/sscargal/lib3mf-rs/actions)
[![License](https://img.shields.io/crates/l/lib3mf-core.svg)](LICENSE)

Parse and validate 3MF files for manufacturing workflows - production-ready with streaming parser and comprehensive validation.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
lib3mf-core = "0.4"
```

Parse a 3MF file:

```rust
use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
use lib3mf_core::parser::parse_model;
use std::fs::File;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("model.3mf")?;
    let mut archiver = ZipArchiver::new(file)?;
    let model_path = find_model_path(&mut archiver)?;
    let model_data = archiver.read_entry(&model_path)?;
    let model = parse_model(Cursor::new(model_data))?;
    println!("Build items: {}", model.build.items.len());
    Ok(())
}
```

## Examples

### Parse and Validate

```rust
use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
use lib3mf_core::parser::parse_model;
use lib3mf_core::validation::ValidationLevel;
use std::fs::File;
use std::io::Cursor;

let file = File::open("model.3mf")?;
let mut archiver = ZipArchiver::new(file)?;
let model_path = find_model_path(&mut archiver)?;
let model_data = archiver.read_entry(&model_path)?;
let model = parse_model(Cursor::new(model_data))?;
let report = model.validate(ValidationLevel::Standard)?;

if report.has_errors() {
    for error in report.errors() {
        eprintln!("Error: {}", error);
    }
}
```

### Streaming Parser (Large Files)

```rust
use lib3mf_core::parser::parse_model_streaming;
use lib3mf_core::streaming::StatsVisitor;

let mut visitor = StatsVisitor::new();
parse_model_streaming(reader, &mut visitor)?;
println!("Vertices: {}", visitor.vertex_count());
```

### Write 3MF

```rust
use lib3mf_core::Model;
use std::fs::File;

let model = Model::default();
let output = File::create("output.3mf")?;
model.write(output)?;
```

### Geometry Repair

```rust
use lib3mf_core::model::repair::{MeshRepair, RepairOptions};

let repaired = model.repair_geometry(&RepairOptions::default())?;
println!("Stitched {} vertices", repaired.vertices_merged);
```

## Why lib3mf-rs?

- **Production-ready**: Comprehensive validation system with 4 levels (Minimal/Standard/Strict/Paranoid)
- **Zero unsafe code**: Memory-safe by design
- **Comprehensive testing**: 90%+ test coverage, fuzzing infrastructure, property-based tests
- **Feature flags**: Minimal dependencies by default, add crypto/parallel/png-validation as needed
- **Complete spec support**: All 9 official 3MF extensions (Materials, Slicing, Security, Boolean Ops, Displacement, etc.)
- **Conformance tested**: 86% pass rate on 3MF Consortium official test suite

## Feature Flags

| Feature | Description |
|---------|-------------|
| `crypto` | Digital signatures and encryption (Secure Content Extension) |
| `parallel` | Multi-threaded mesh processing for large files |
| `png-validation` | Validate PNG texture files |
| `full` | All features enabled |

```toml
# Minimal (default)
lib3mf-core = "0.4"

# With parallel processing
lib3mf-core = { version = "0.4", features = ["parallel"] }

# All features
lib3mf-core = { version = "0.4", features = ["full"] }
```

## Ecosystem

lib3mf-rs is a multi-crate workspace:

| Crate | When to Use |
|-------|-------------|
| **lib3mf-core** (this crate) | Core parsing, validation, and writing |
| [lib3mf-async](https://crates.io/crates/lib3mf-async) | Non-blocking async I/O with tokio |
| [lib3mf-cli](https://crates.io/crates/lib3mf-cli) | Command-line analysis and processing |
| [lib3mf-converters](https://crates.io/crates/lib3mf-converters) | STL/OBJ format conversion |
| [lib3mf-wasm](https://www.npmjs.com/package/@lib3mf-rs/lib3mf-wasm) | Browser and edge deployment (NPM) |

## Specification Compliance

Implements 3MF specifications:
- Core Specification v1.4.0
- Beam Lattice Extension v1.2.0
- Boolean Operations Extension v1.1.1
- Displacement Extension v1.0.0
- Materials and Properties Extension v1.2.1
- Production Extension v1.1.2
- Secure Content Extension v1.0.2
- Slice Extension v1.0.2
- Volumetric Extension v0.8.0

## Documentation

- [API Reference (docs.rs)](https://docs.rs/lib3mf-core)
- [User Guide](https://sscargal.github.io/lib3mf-rs/stable/book/)
- [Examples](https://github.com/sscargal/lib3mf-rs/tree/main/crates/lib3mf-core/examples)

## Performance

Benchmarks on M1 Mac with 3MB test file:
- Parse: 15ms (200 MiB/s)
- Validate (Standard): 18ms
- Validate (Paranoid): 45ms
- Write: 12ms

See [benchmarks](../../benches/) for detailed metrics.

## License

BSD-2-Clause
