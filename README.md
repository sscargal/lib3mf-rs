# lib3mf-rs

![CI](https://github.com/sscargal/lib3mf-rs/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

`lib3mf-rs` is a pure Rust implementation of the [3D Manufacturing Format (3MF)](https://3mf.io/) standard. `lib3mf-rs` provides a memory-safe, high-performance library and CLI tools for reading, analyzing, and processing 3MF files. It provides 3MF reading and writing capabilities, as well as conversion and validation tools for input and output data. lib3mf runs on Windows, Linux, and MacOS and offers a clean and easy-to-use API. It complements the [existing C++ implementation](https://github.com/3MFConsortium/lib3mf).

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

## Usage

### Prerequisites
- [Rust](https://rust-lang.org/) (latest stable, v1.93 or later)

### Building from Source

Clone the repository and build the project:

```bash
git clone https://github.com/stevescargall/lib3mf-rs.git
cd lib3mf-rs
```

**Development Build (Debug)**:
Faster compilation, but slower runtime execution. Best for testing and development.
```bash
cargo build
```

**Release Build**:
Optimized for performance. Use this for production or benchmarking.
```bash
cargo build --release
```

### Running the CLI

The `3mf` CLI tool allows you to inspect and analyze 3MF files.

#### 1. Quick Stats
Get a summary of the model, including geometry counts, advanced materials (textures, composites), and vendor metadata (like Bambu Studio plates).

```bash
cargo run -p lib3mf-cli -- stats path/to/model.3mf

# Output as JSON
cargo run -p lib3mf-cli -- stats path/to/model.3mf --format json
```

#### 2. List Archive Contents
See what's inside the 3MF container. Use `--format tree` for a file tree view.

```bash
cargo run -p lib3mf-cli -- list path/to/model.3mf --format tree
```

#### 3. Inspect Relationships
Debug OPC relationships and content types.

```bash
cargo run -p lib3mf-cli -- rels path/to/model.3mf
```

#### 4. Extract Files
Extract specific files (like thumbnails or internal config) from the archive.

```bash
cargo run -p lib3mf-cli -- extract path/to/model.3mf "Auxiliaries/.thumbnails/thumbnail_small.png" --output thumb.png
```

#### 5. Compare Models
Compare two 3MF files to find structural or metadata differences.

```bash
cargo run -p lib3mf-cli -- diff v1.3mf v2.3mf
```

#### 6. Copy (Roundtrip)
Read a 3MF file and write it back to a new file. Validates the read/write cycle.

```bash
cargo run -p lib3mf-cli -- copy path/to/model.3mf output.3mf
```

### Library Usage

Add `lib3mf-core` to your `Cargo.toml`:

```toml
[dependencies]
lib3mf-core = { path = "crates/lib3mf-core" } # Or git dependency
```

Example: parsing a model and getting stats.

```rust
use lib3mf_core::archive::ZipArchiver;
use lib3mf_core::parser::parse_model;
use lib3mf_core::archive::{ArchiveReader, find_model_path};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("models/Benchy.3mf")?;
    let mut archiver = ZipArchiver::new(file)?;
    
    // 1. Find and parse the 3D model
    let model_path = find_model_path(&mut archiver)?;
    let model_data = archiver.read_entry(&model_path)?;
    let model = parse_model(std::io::Cursor::new(model_data))?;
    
    // 2. Compute statistics (needs archiver to look up extensions like Bambu configs)
    let stats = model.compute_stats(&mut archiver)?;
    
    println!("Triangles: {}", stats.geometry.triangle_count);
    Ok(())
}
```

### Code Examples

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
