# lib3mf-converters

[![Crates.io](https://img.shields.io/crates/v/lib3mf-converters.svg)](https://crates.io/crates/lib3mf-converters)
[![docs.rs](https://docs.rs/lib3mf-converters/badge.svg)](https://docs.rs/lib3mf-converters)
[![License](https://img.shields.io/crates/l/lib3mf-converters.svg)](LICENSE)

Convert between 3MF, STL, and OBJ formats for 3D printing workflows.

## When to Use This Crate

Use `lib3mf-converters` when you need:
- Convert legacy STL files to modern 3MF format
- Export 3MF models to OBJ for visualization
- Batch format conversion in manufacturing pipelines
- Integration with CAD tools that use different formats

## Quick Start

```toml
[dependencies]
lib3mf-converters = "0.4"
```

### STL to 3MF (auto-detects binary or ASCII)

```rust,no_run
use lib3mf_converters::stl::StlImporter;
use std::fs::File;

let file = File::open("model.stl")?;
let model = StlImporter::read(file)?;
model.write(File::create("output.3mf")?)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 3MF to Binary STL

```rust,no_run
use lib3mf_converters::stl::BinaryStlExporter;
use lib3mf_core::archive::{ZipArchiver, find_model_path, ArchiveReader};
use lib3mf_core::parser::parse_model;
use std::fs::File;

let file = File::open("model.3mf")?;
let mut archiver = ZipArchiver::new(file)?;
let model_path = find_model_path(&mut archiver)?;
let data = archiver.read_entry(&model_path)?;
let model = parse_model(std::io::Cursor::new(data))?;
BinaryStlExporter::write(&model, File::create("output.stl")?)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 3MF to ASCII STL

```rust,no_run
use lib3mf_converters::stl::AsciiStlExporter;
use lib3mf_core::archive::{ZipArchiver, find_model_path, ArchiveReader};
use lib3mf_core::parser::parse_model;
use std::fs::File;

let file = File::open("model.3mf")?;
let mut archiver = ZipArchiver::new(file)?;
let model_path = find_model_path(&mut archiver)?;
let data = archiver.read_entry(&model_path)?;
let model = parse_model(std::io::Cursor::new(data))?;
AsciiStlExporter::write(&model, File::create("output.stl")?)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Supported Formats

| Format | Import | Export | Notes |
|--------|--------|--------|-------|
| STL (Binary) | ✅ | ✅ | Standard binary STL |
| STL (ASCII) | ✅ | ✅ | Text-based STL |
| OBJ | ✅ | ✅ | Wavefront OBJ with vertices and faces |

## Features

- Binary and ASCII STL import with automatic format detection
- Binary STL export (`BinaryStlExporter`) and ASCII STL export (`AsciiStlExporter`)
- OBJ import (vertices and faces) and export
- Preserves mesh topology
- Error recovery for malformed files

## CLI Usage

Format conversion is also available via the CLI tool:

```bash
cargo install lib3mf-cli
3mf convert input.stl output.3mf
3mf convert model.3mf output.obj
```

## Related

- [lib3mf-core](https://crates.io/crates/lib3mf-core) - Core parsing library (required dependency)
- [lib3mf-cli](https://crates.io/crates/lib3mf-cli) - Command-line converter
- [Full Documentation](https://sscargal.github.io/lib3mf-rs/)

## License

BSD-2-Clause
