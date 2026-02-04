# Architecture Overview

This chapter explains the internal architecture of lib3mf-rs, including crate structure, data flow pipeline, and key design patterns.

## Crate Structure

lib3mf-rs is organized as a Cargo workspace with five crates:

```text
lib3mf-rs/
├── crates/
│   ├── lib3mf-core/        # Main library implementation
│   ├── lib3mf-cli/         # Command-line interface
│   ├── lib3mf-converters/  # STL and OBJ format converters
│   ├── lib3mf-async/       # Async I/O with tokio
│   └── lib3mf-wasm/        # WebAssembly bindings
├── fuzz/                   # Fuzzing targets (cargo-fuzz)
├── book/                   # This documentation (mdBook)
└── Cargo.toml              # Workspace definition
```

**lib3mf-core** — The main library containing:
- Archive layer (ZIP/OPC package handling)
- Parser layer (XML to model conversion)
- Model layer (immutable data structures)
- Validation layer (4-level progressive validation)
- Writer layer (model to XML/ZIP serialization)
- Crypto layer (digital signatures and encryption)

**lib3mf-cli** — Binary crate providing command-line tools for inspecting, validating, and analyzing 3MF files.

**lib3mf-converters** — Standalone converters between 3MF and other formats (STL, OBJ).

**lib3mf-async** — Asynchronous I/O support using tokio and async-zip for non-blocking file operations.

**lib3mf-wasm** — WebAssembly bindings for running lib3mf-rs in browsers and WASM runtimes.

## Data Flow Pipeline

The library follows a layered architecture where each layer has a single responsibility:

```
Archive Layer (ZIP/OPC)
    ↓
Parser Layer (XML)
    ↓
Model Layer (Immutable)
    ↓
Validation/Processing
    ↓
Writer Layer (XML/ZIP)
```

### Typical Read Workflow

1. **Archive Layer** — `ZipArchiver::new()` opens the 3MF file (ZIP container)
2. **OPC Layer** — Read `_rels/.rels` to discover main model XML path via `find_model_path()`
3. **Parser Layer** — `parse_model()` converts XML into in-memory `Model` structure
4. **Model Layer** — `Model` contains resources (objects, materials, textures) and build instructions
5. **Validation Layer** — Apply checks at chosen level (Minimal/Standard/Strict/Paranoid)

### Typical Write Workflow

1. **Model Layer** — Create or modify `Model` structure programmatically
2. **Writer Layer** — `write_package()` serializes model to XML and ZIP
3. **Archive Layer** — Write OPC relationships and package structure
4. **Output** — Produces a valid 3MF file

## Module Structure in lib3mf-core

### `archive/` — OPC Container Handling

**Key files:**
- `zip_archive.rs` — ZIP wrapper implementing `ArchiveReader` trait
- `opc.rs` — Open Packaging Convention (relationships and content types)

The `ArchiveReader` trait abstracts over different archive backends, allowing the parser to work with any ZIP implementation or even in-memory archives for testing.

```rust
pub trait ArchiveReader {
    fn read_entry(&mut self, path: &str) -> Result<Vec<u8>>;
    fn entry_names(&mut self) -> Result<Vec<String>>;
}
```

### `parser/` — XML to Model Conversion

**Key files:**
- `model_parser.rs` — Main orchestrator parsing `<model>` element
- `mesh_parser.rs` — Geometry parsing (vertices, triangles)
- `material_parser.rs` — Materials (colors, textures, composites)
- `beamlattice_parser.rs` — Beam Lattice Extension parser
- `slice_parser.rs` — Slice Extension parser
- `secure_content_parser.rs` — Secure Content Extension parser
- `streaming.rs` — SAX-style event-based parser for large files

**Design:** Modular parsing where each extension has its own parser. The main parser delegates to extension parsers based on XML namespaces.

### `model/` — Core Data Structures

**Key files:**
- `core.rs` — Root `Model` struct
- `resources.rs` — `ResourceCollection` (central resource registry)
- `mesh.rs` — Geometry types (`Mesh`, `Triangle`, `Vertex`, `BeamLattice`)
- `materials.rs` — Material types (BaseMaterial, ColorGroup, Texture2D, Composite, MultiProperties)
- `build.rs` — `Build` and `BuildItem` (what to print and where)
- `secure_content.rs` — Encryption/signature metadata
- `repair.rs` — `MeshRepair` trait for geometry fixing

**Design:** Immutable-by-default. Structures use Clone semantics. Mutation happens via explicit repair operations.

```rust
pub struct Model {
    pub unit: String,
    pub metadata: Vec<MetadataEntry>,
    pub resources: ResourceCollection,
    pub build: Build,
    pub attachments: Vec<Attachment>,
}
```

### `validation/` — Progressive Validation System

**Key files:**
- `validator.rs` — Main validation orchestration
- `geometry.rs` — Geometry checks (manifoldness, self-intersection, orientation)
- `bvh.rs` — Bounding Volume Hierarchy for O(n log n) intersection tests
- `report.rs` — Structured validation results

**Four validation levels:**

1. **Minimal** — Basic structural checks (well-formed XML, valid IDs)
2. **Standard** — Reference integrity (all referenced resources exist)
3. **Strict** — Full spec compliance (ranges, formats, constraints)
4. **Paranoid** — Deep geometry analysis (manifoldness, self-intersection, orientation)

### `writer/` — Model to File Serialization

**Key files:**
- `model_writer.rs` — Main model serialization
- `mesh_writer.rs` — Geometry serialization
- `package_writer.rs` — Top-level package orchestration
- `opc_writer.rs` — OPC relationships and content types

**Design:** Mirrors parser structure but in reverse. Each module is responsible for writing its corresponding XML elements.

### `crypto/` — Secure Content Extension

**Key files:**
- `signature.rs` — XML-DSIG digital signature verification
- `encryption.rs` — Content encryption/decryption (AES-GCM)
- `cert.rs` — X.509 certificate parsing

**Feature-gated:** Only available when `crypto` feature is enabled to reduce dependencies.

## Key Design Patterns

### Immutable Model Design

`Model` and child structures are immutable by default. This provides:

- **Thread safety** — Safe to share models across threads
- **Predictable behavior** — No hidden mutations
- **Easier testing** — No state changes between operations

Mutation happens explicitly via repair operations:

```rust
use lib3mf_core::model::repair::MeshRepair;

let repaired = mesh.stitch_vertices(epsilon)?;
```

### Trait-Based Abstractions

**ArchiveReader** — Decouples parser from ZIP implementation:

```rust
pub trait ArchiveReader {
    fn read_entry(&mut self, path: &str) -> Result<Vec<u8>>;
    fn entry_names(&mut self) -> Result<Vec<String>>;
}
```

**ModelVisitor** — Visitor pattern for streaming parser:

```rust
pub trait ModelVisitor {
    fn visit_object(&mut self, id: ResourceId, object: &Object);
    fn visit_build_item(&mut self, item: &BuildItem);
}
```

**MeshRepair** — Trait for mesh repair operations:

```rust
pub trait MeshRepair {
    fn stitch_vertices(&self, epsilon: f32) -> Result<Self>;
    fn remove_degenerate_faces(&self) -> Result<Self>;
    fn harmonize_orientation(&self) -> Result<Self>;
}
```

### Error Handling Strategy

Uses `thiserror` for custom error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Lib3mfError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Lib3mfError>;
```

**Philosophy:** No panics in library code. All errors are expected and recoverable.

### Two Parsing Modes

**DOM Mode** — Loads entire model into memory:

```rust
let model = parse_model(reader)?;  // Fast, simple, <100MB files
```

**SAX Mode** — Event-based streaming:

```rust
struct MyVisitor;
impl ModelVisitor for MyVisitor {
    fn visit_object(&mut self, id: ResourceId, object: &Object) {
        // Process objects as they're parsed
    }
}

parse_model_streaming(reader, &mut MyVisitor)?;  // Constant memory, GB+ files
```

### Resource Management

Resources use a typed ID system to prevent mixing different resource types:

```rust
pub struct ResourceId(pub u32);  // Newtype pattern

pub struct ResourceCollection {
    objects: HashMap<ResourceId, Object>,
    materials: HashMap<ResourceId, BaseMaterialGroup>,
    textures: HashMap<ResourceId, Texture2D>,
    // ...
}
```

All resources share a global ID namespace within a model. Duplicate IDs are detected and rejected.

### Property System (Materials on Geometry)

Materials can be applied at multiple levels with a clear precedence hierarchy:

1. **Per-vertex properties** — `Triangle` with `p1`, `p2`, `p3` attributes
2. **Per-triangle properties** — `Triangle` with `pid` attribute
3. **Per-object default** — `Object` with `pid` and `pindex`

Resolution order: Triangle → Vertex → Object → None

## Performance Characteristics

### Hotspots

**XML Parsing** — Uses `quick-xml` (event-based, fast, zero-copy where possible)

**Float Parsing** — Uses `lexical-core` (spec-compliant and 2-5x faster than stdlib)

**Self-Intersection Detection** — Uses BVH (Bounding Volume Hierarchy) for O(n log n) instead of naive O(n²)

**Statistics Computation** — Can be parallelized with Rayon when `parallel` feature enabled

### Memory vs Speed Tradeoffs

| Approach | Speed | Memory | Best For |
|----------|-------|--------|----------|
| DOM mode (`parse_model`) | Fast | O(n) | Files <100MB |
| SAX mode (streaming) | Slower | O(1) | Files >1GB |
| Parallel (Rayon) | Fastest | O(n) | Large meshes with multi-core CPU |

### Feature Flags and Dependencies

| Configuration | Dependency Count | Use Case |
|---------------|------------------|----------|
| Minimal (`default = []`) | ~154 crates | Embedded systems, WASM, size-critical |
| Crypto only | ~300 crates | Secure files, signature verification |
| Parallel only | ~160 crates | Fast mesh processing, no security |
| Full (`features = ["full"]`) | ~300 crates | Complete functionality |

Users who don't need crypto save 48% of dependencies.

## Extension Architecture

Extensions are first-class citizens integrated directly into core structures:

**Parser Integration:**
- Each extension has its own parser module
- Main parser delegates based on XML namespace
- Extensions can add new resource types to `ResourceCollection`

**Model Integration:**
- Extension data stored directly in model structures
- No separate "extension bag" — type-safe access
- Geometry enum includes extension types (BeamLattice, SliceStack, etc.)

**Validation Integration:**
- Extensions contribute their own validation rules
- Integrated into 4-level validation system
- Extension-specific error codes

**Example:** Boolean Operations Extension adds `BooleanShape` to `Geometry` enum and provides its own parser, validator, and writer.

## Next Steps

- **[Extensions](extensions.md)** — Details on all 9 3MF extensions
- **[Validation Guide](validation-guide.md)** — Deep dive into validation system
- **[Contributing](contributing.md)** — Adding new extensions or features
- **[API Reference](../rustdoc/lib3mf_core/index.html)** — Complete API documentation
