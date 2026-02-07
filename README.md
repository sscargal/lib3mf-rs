# lib3mf-rs

**A pure Rust 3D Manufacturing Format (3MF) parser and toolkit for 3D printing and CAD**

<p align="center">
  <a href="https://crates.io/crates/lib3mf-core"><img src="https://img.shields.io/crates/v/lib3mf-core.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/lib3mf-core"><img src="https://docs.rs/lib3mf-core/badge.svg" alt="Documentation"></a>
  <a href="https://crates.io/crates/lib3mf-core"><img src="https://img.shields.io/crates/d/lib3mf-core.svg" alt="Downloads"></a>
  <br>
  <a href="https://github.com/sscargal/lib3mf-rs/actions"><img src="https://github.com/sscargal/lib3mf-rs/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <a href="https://sscargal.github.io/lib3mf-rs/"><img src="https://img.shields.io/badge/docs-guide-blue.svg" alt="Documentation"></a>
</p>

Memory-safe, high-performance library for reading, writing, and processing [3MF files](https://3mf.io/) used in 3D printing, additive manufacturing, and CAD workflows. Supports digital signatures, encryption, advanced materials, slicing, boolean operations, and all 9 official 3MF specifications.

---

## Quick Start

### Install CLI (fastest way to try)

```bash
# Install the command-line tool
cargo install lib3mf-cli

# Analyze a 3MF file
lib3mf-cli stats path/to/model.3mf

# Get JSON output
lib3mf-cli stats path/to/model.3mf --format json
```

### Use as Library

Add to your `Cargo.toml`:
```toml
[dependencies]
lib3mf-core = "0.1"
```

Simple example:
```rust
use lib3mf_core::Model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::from_file("model.3mf")?;
    let stats = model.compute_stats()?;
    println!("Triangles: {}", stats.geometry.triangle_count);
    Ok(())
}
```

**[📖 More Examples](examples/)** | **[📋 Full Feature List](docs/features.md)** | **[🔧 API Documentation](https://docs.rs/lib3mf-core)**

---

## Documentation

Comprehensive documentation is available at **[sscargal.github.io/lib3mf-rs](https://sscargal.github.io/lib3mf-rs/)**:

- **[User Guide](https://sscargal.github.io/lib3mf-rs/stable/book/)** – Tutorial, architecture overview, and practical examples
- **[API Reference](https://sscargal.github.io/lib3mf-rs/stable/rustdoc/lib3mf_core/)** – Complete rustdoc for all public APIs
- **[Dev Docs](https://sscargal.github.io/lib3mf-rs/dev/book/)** – Latest unreleased changes on main branch

The documentation covers:
- Getting Started tutorial with step-by-step examples
- Architecture deep-dive into parsing, validation, and writing
- Extension guides for Materials, Secure Content, Boolean Operations, Slicing, and more
- Advanced topics: streaming parsing, mesh repair, format conversion
- Full API documentation with testable code examples

---

## Why lib3mf-rs?

✅ **Pure Rust** – No C++ dependencies, memory-safe by design  
✅ **Complete Spec Support** – All 9 official 3MF extensions (Materials, Slicing, Security, Boolean Ops, Displacement, etc.)  
✅ **Production Ready** – Geometry repair, validation, digital signature verification  
✅ **Fast** – Optional multi-threading for large files, streaming parser for low memory usage  
✅ **Vendor Support** – Native Bambu Studio project file parsing  
✅ **Format Conversion** – Built-in STL and OBJ converters  

| Feature | lib3mf-rs | lib3MF (C++) |
|---------|-----------|---------------|
| Language | Rust | C++ |
| Memory Safety | ✅ Guaranteed | ⚠️ Manual |
| Dependencies (minimal) | 0 | Many |
| WASM Support | ✅ | ❌ |
| Async I/O | ✅ | ❌ |

---

## Rust 3MF Ecosystem

lib3mf-rs is one of several Rust libraries for 3MF. See [docs/alternatives.md](docs/alternatives.md) for detailed comparison.

| Crate | Version | Last Updated | Core capabilities | Modern features | Quality metrics | License & maintenance |
|-------|---------|--------------|-------------------|-----------------|-----------------|----------------------|
| lib3mf-rs | 0.1.0 | 2026-02-04 | Parse, Write, Validation (4 levels), 9 extensions | Async, WASM, CLI, Converters | 90%+ coverage, Fuzzing, Zero unsafe | BSD 2-Clause, Active |
| [lib3mf](https://crates.io/crates/lib3mf) | 0.1.0 | 2026-02-04 | Parse, Write, Validation, 4 extensions | Geometry (parry3d) | 2200+ tests | MIT, Active |
| [threemf2](https://crates.io/crates/threemf2) | 0.1.2 | 2025-11-30 | Parse, Write (basic) | None | Basic | MIT, Limited |
| [thdmaker](https://crates.io/crates/thdmaker) | 0.0.4 | 2026-01-11 | STL/AMF focus | None | Basic | MIT, Active |
| [mesh_rs](https://crates.io/crates/mesh_rs) | 1.0.4 | 2025-12-17 | Multi-format (STL, OBJ, PLY) | None | Basic | MIT, Active |
| [stlto3mf](https://crates.io/crates/stlto3mf) | 0.1.0 | 2024-07-16 | STL to 3MF conversion | None | Basic | MIT, Stable |

**Quick Guide:**
- **Production apps with async/WASM:** Use lib3mf-rs
- **Academic research:** Consider lib3mf (parry3d integration)
- **Simple reading:** threemf2 is lightweight
- **STL/AMF focus:** thdmaker
- **Multi-format mesh:** mesh_rs
- **STL conversion only:** stlto3mf

---

## Features

- **Pure Rust**: No C++ bindings, memory-safe.
- **Robust Parsing**: Validation of XML structure and relationships. Includes "Paranoid" mode for geometry checks.
- **Geometry Repair**: Stitching of vertices (epsilon merge) and removal of degenerate faces.
- **Secure Content**: Full support for XML Digital Signatures and Content Encryption.
    - Parse and verify X.509 certificate chains.
    - Decrypt protected resources.
    - CLI `verify` command for signature validation.
- **Model Statistics**: Compute geometry counts (vertices, triangles) and instance counts.
- **Vendor Extensions**: Native support for **Bambu Studio** project files (recognizing plates and metadata).
- **CLI Tool**: Inspect 3MF files directly from the command line.
- **Format Conversion**: Convert between 3MF, STL, and OBJ formats.

**[📋 Complete Feature Matrix](docs/features.md)** – Detailed implementation status of all 325 features across 9 specifications.

## Specification Compliance

`lib3mf-rs` implements the following 3MF specifications:

- 3MF Core Specification v1.4.0
- Beam Lattice Extension v1.2.0
- Boolean Operations Extension v1.1.1
- Displacement Extension v1.0.0
- Materials and Properties Extension v1.2.1
- Production Extension v1.1.2
- Secure Content Extension v1.0.2
- Slice Extension v1.0.2
- Volumetric Extension v0.8.0

`lib3mf-rs` has limited support for vendor extensions, such as:

- Bambu Studio 3MF project files

---

## Library Usage

### Basic Example

Add `lib3mf-core` to your `Cargo.toml`:

```toml
[dependencies]
lib3mf-core = "0.1"
```

Parse a 3MF file and get statistics:

```rust
use lib3mf_core::Model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load and parse the model
    let model = Model::from_file("model.3mf")?;
    
    // Get statistics
    let stats = model.compute_stats()?;
    println!("Triangles: {}", stats.geometry.triangle_count);
    println!("Vertices: {}", stats.geometry.vertex_count);
    
    Ok(())
}
```

**Advanced usage** (direct archive access for streaming):
```rust
use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
use lib3mf_core::parser::parse_model;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("models/Benchy.3mf")?;
    let mut archiver = ZipArchiver::new(file)?;
    
    // Find and parse the 3D model
    let model_path = find_model_path(&mut archiver)?;
    let model_data = archiver.read_entry(&model_path)?;
    let model = parse_model(std::io::Cursor::new(model_data))?;
    
    // Compute statistics (needs archiver for extensions)
    let stats = model.compute_stats(&mut archiver)?;
    
    println!("Triangles: {}", stats.geometry.triangle_count);
    Ok(())
}
```

### Feature Flags

`lib3mf-core` uses cargo feature flags to keep dependencies minimal. By default, **no optional features are enabled**.

| Feature | Description | Crate Dependencies |
|---------|-------------|-------------------|
| `crypto` | Digital signatures and encryption (Secure Content Extension) | aes-gcm, rsa, sha1, sha2, x509-parser, rand, base64 |
| `parallel` | Multi-threaded mesh processing for large files | rayon |
| `png-validation` | Validate PNG texture files | png |
| `full` | All features enabled | all of the above |

**Usage examples:**

Add one of the following to your project's `Cargo.toml` depending on your needs:

```toml
# Minimal (smallest dependency footprint)
lib3mf-core = "0.1"

# With parallel processing (recommended for large files)
lib3mf-core = { version = "0.1", features = ["parallel"] }

# With crypto support (for signed/encrypted 3MF files)
lib3mf-core = { version = "0.1", features = ["crypto"] }

# With PNG texture validation
lib3mf-core = { version = "0.1", features = ["png-validation"] }

# Everything enabled
lib3mf-core = { version = "0.1", features = ["full"] }
```

**Note:** The `lib3mf-core` library uses minimal features by default (zero optional dependencies). The `lib3mf-cli` binary enables `crypto` and `parallel` features by default for optimal performance and security.

---

## CLI Commands

The `lib3mf-cli` tool provides comprehensive commands for inspecting and analyzing 3MF files.

### Installation

```bash
cargo install lib3mf-cli
```

### Common Commands

#### 1. Quick Stats
Get a summary of the model, including geometry counts, advanced materials (textures, composites), and vendor metadata (like Bambu Studio plates).

```bash
lib3mf-cli stats path/to/model.3mf

# Output as JSON
lib3mf-cli stats path/to/model.3mf --format json

# Or run from source
cargo run -p lib3mf-cli -- stats path/to/model.3mf
```

#### 2. List Archive Contents
See what's inside the 3MF container. Use `--format tree` for a file tree view.

```bash
lib3mf-cli list path/to/model.3mf --format tree
```

#### 3. Inspect Relationships
Debug OPC relationships and content types.

```bash
lib3mf-cli rels path/to/model.3mf
```

#### 4. Extract Files
Extract specific files (like thumbnails or internal config) from the archive.

```bash
lib3mf-cli extract path/to/model.3mf "Auxiliaries/.thumbnails/thumbnail_small.png" --output thumb.png
```

#### 5. Compare Models
Compare two 3MF files to find structural or metadata differences.

```bash
lib3mf-cli diff v1.3mf v2.3mf
```

#### 6. Copy (Roundtrip)
Read a 3MF file and write it back to a new file. Validates the read/write cycle.

```bash
lib3mf-cli copy path/to/model.3mf output.3mf
```

---

## Building from Source

### Prerequisites
- [Rust](https://rust-lang.org/) toolchain (latest stable, v1.93 or later)

### Clone and Build

```bash
git clone https://github.com/stevescargall/lib3mf-rs.git
cd lib3mf-rs

# Development build (faster compilation, debug info)
cargo build

# Release build (optimized for performance)
cargo build --release
```

---

## Code Examples

You can run any of the examples using `cargo run -p <crate> --example <name>`.

#### `lib3mf-core`
Core 3MF logic, specifications, and SPEC-defined extensions.

```bash
cargo run -p lib3mf-core --example create_cube
```

- `advanced_materials`: Parsing Texture 2D, Composite Materials, and Multi Properties.
- `geometry_validation`: Using "Paranoid" validation to find non-manifold edges.
- `geometry_repair`: Programmatically repairing a mesh (stitching, degenerate removal).
- `secure_content`: Digital signatures and encrypted content handling.
- `model_diff`: Structural comparison between two 3MF models.
- `create_cube`: Building a 3MF model from scratch.
- `components_transform`: Efficient object instancing using components.
- `metadata_properties`: Managing standard and custom model metadata.
- `beam_lattice_ext`: Creating structural designs with the Beam Lattice extension.
- `boolean_operations`: Boolean operations (union, difference, intersection).
- `displacement_mesh`: Creating displacement meshes with texture-driven surface modification.
- `slice_data`: Defining geometry using 2D slice stacks (DLP/SLA printing).
- `streaming_stats`: SAX-style parser for massive files with constant memory.

#### `lib3mf-converters`
Format conversion and external data processing.

```bash
cargo run -p lib3mf-converters --example stl_conversion
```

- `stl_conversion`: Roundtrip between STL and 3MF `Model` structure.
- `obj_conversion`: Roundtrip between Wavefront OBJ and 3MF.

#### `lib3mf-async`
High-performance asynchronous I/O.

```bash
cargo run -p lib3mf-async --example async_load
```

- `async_load`: Non-blocking 3MF loading using `tokio` and `async-zip`.

## Running Tests

We have a comprehensive test suite covering spec compliance and vendor integrations.

```bash
cargo test
```

## Project Structure

```text
lib3mf-rs/
├── crates/             # Workspace members
│   ├── lib3mf-core/    # Main library implementation
│   ├── lib3mf-cli/     # Command-line interface
│   ├── lib3mf-converters/      # Format converters (STL, OBJ)
│   ├── lib3mf-wasm/    # WebAssembly bindings
│   └── lib3mf-async/   # Async I/O (In Progress)
├── docs/               # Documentation
├── examples/           # Code examples
├── fuzz/               # Fuzzing targets
└── CONTRIBUTING.md     # Developer guide
```

See the `README.md` in each subdirectory for more details.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for build instructions, testing guide, and architecture overview.
