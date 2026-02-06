# lib3mf-converters

[![Crates.io](https://img.shields.io/crates/v/lib3mf-converters.svg)](https://crates.io/crates/lib3mf-converters)
[![docs.rs](https://docs.rs/lib3mf-converters/badge.svg)](https://docs.rs/lib3mf-converters)

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
lib3mf-converters = "0.1"
```

### STL to 3MF

```rust
use lib3mf_converters::stl::StlImporter;
use lib3mf_core::writer::write_model;

let importer = StlImporter::new();
let model = importer.import_file("model.stl")?;
write_model(&model, "output.3mf")?;
```

### 3MF to OBJ

```rust
use lib3mf_core::Model;
use lib3mf_converters::obj::ObjExporter;

let model = Model::from_file("model.3mf")?;
let exporter = ObjExporter::new();
exporter.export(&model, "output.obj")?;
```

## Supported Formats

| Format | Import | Export | Notes |
|--------|--------|--------|-------|
| STL (Binary) | ✅ | ✅ | Standard binary STL |
| STL (ASCII) | ✅ | ✅ | Text-based STL |
| OBJ | ✅ | ✅ | Wavefront OBJ with vertices and faces |

## Features

- Binary and ASCII STL support
- OBJ with vertex normals
- Preserves mesh topology
- Efficient memory usage with parallel processing
- Error recovery for malformed files

## CLI Usage

Format conversion is also available via the CLI tool:

```bash
cargo install lib3mf-cli
lib3mf-cli convert input.stl output.3mf
lib3mf-cli convert model.3mf output.obj
```

## Related

- [lib3mf-core](https://crates.io/crates/lib3mf-core) - Core parsing library (required dependency)
- [lib3mf-cli](https://crates.io/crates/lib3mf-cli) - Command-line converter
- [Full Documentation](https://sscargal.github.io/lib3mf-rs/)

## License

MIT OR Apache-2.0
