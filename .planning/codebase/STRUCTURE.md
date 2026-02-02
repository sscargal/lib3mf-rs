# Codebase Structure

**Analysis Date:** 2026-02-02

## Directory Layout

```
lib3mf-rs/
├── crates/                                # Multi-crate workspace
│   ├── lib3mf-core/                      # Main library (parser, model, validation, writer)
│   │   ├── src/
│   │   │   ├── archive/                  # OPC/ZIP handling (traits, implementations)
│   │   │   ├── crypto/                   # Encryption/signature support
│   │   │   ├── model/                    # Core data structures (Model, Mesh, Object, etc.)
│   │   │   ├── parser/                   # XML parsing (modular by element type)
│   │   │   ├── validation/               # Progressive validation system
│   │   │   ├── writer/                   # Model → XML/ZIP serialization
│   │   │   ├── utils/                    # Diff, stats, hardware detection
│   │   │   ├── error.rs                  # Error types and Result alias
│   │   │   └── lib.rs                    # Public API re-exports
│   │   ├── examples/                     # Usage examples (create_cube, validation, etc.)
│   │   ├── benches/                      # Performance benchmarks
│   │   ├── tests/                        # Integration tests (roundtrip, repair, etc.)
│   │   ├── fuzz/                         # Fuzzing targets
│   │   └── Cargo.toml
│   ├── lib3mf-cli/                       # Command-line tool
│   │   ├── src/
│   │   │   ├── main.rs                   # Entry point, command routing
│   │   │   ├── commands.rs               # Implementation of all CLI commands
│   │   │   └── commands/                 # Additional command modules (thumbnails, etc.)
│   │   ├── examples/                     # CLI usage examples
│   │   └── Cargo.toml
│   ├── lib3mf-converters/                # Format converters (STL, OBJ)
│   │   ├── src/
│   │   │   ├── lib.rs                    # Module re-exports
│   │   │   ├── stl.rs                    # STL ↔ 3MF conversion
│   │   │   └── obj.rs                    # OBJ ↔ 3MF conversion
│   │   ├── examples/                     # Converter usage examples
│   │   └── Cargo.toml
│   ├── lib3mf-async/                     # Async I/O support (tokio, async-zip)
│   │   ├── src/
│   │   │   ├── lib.rs                    # Module re-exports
│   │   │   ├── loader.rs                 # Async model loading
│   │   │   ├── archive.rs                # Async archive traits
│   │   │   └── zip.rs                    # Async ZIP wrapper
│   │   ├── examples/                     # Async usage examples
│   │   ├── tests/                        # Async integration tests
│   │   └── Cargo.toml
│   └── lib3mf-wasm/                      # WebAssembly bindings
│       ├── src/
│       │   └── lib.rs                    # Wasm wrapper around core
│       ├── examples/                     # WASM usage examples
│       └── Cargo.toml
├── examples/                              # Root-level examples (shared across crates)
├── models/                                # Test/sample 3MF files
├── scripts/                               # Build and test scripts (qa_test_suite.sh)
├── fuzz/                                  # Fuzzing infrastructure
├── docs/                                  # Documentation
├── .planning/                             # GSD planning workspace
├── Cargo.toml                             # Workspace root
├── CLAUDE.md                              # Development guide
├── README.md                              # Project overview
└── rust-toolchain.toml                    # Rust version specification
```

## Directory Purposes

**crates/lib3mf-core/src/archive/:**
- Purpose: OPC container and ZIP file abstraction
- Contains: `ZipArchiver` (ZIP wrapper), `ArchiveReader` trait, OPC relationship/content type parsers, model path locator
- Key files: `zip_archive.rs`, `opc.rs`, `model_locator.rs`
- Trait-based design allows alternative archive backends

**crates/lib3mf-core/src/crypto/:**
- Purpose: Secure Content Extension implementation (encryption, signatures)
- Contains: AES-GCM encryption/decryption, X.509 certificate handling, signature verification
- Imported by: Secure content parser and writer

**crates/lib3mf-core/src/model/:**
- Purpose: Core data structures representing 3MF content
- Contains: `Model` root struct, `ResourceCollection` registry, `Mesh`, `Object`, `Geometry` enum, material types, build instructions, repair operations, statistics computation
- Key files:
  - `core.rs`: Root `Model` struct
  - `mesh.rs`: `Object`, `Geometry`, `Mesh`, `Triangle`, `Vertex`, `BeamLattice`
  - `materials.rs`: `BaseMaterial`, `ColorGroup`, `Texture2D`, `Composite`, `MultiProperties`
  - `resources.rs`: `ResourceId` and `ResourceCollection` registry
  - `build.rs`: `Build` and `BuildItem` structures
  - `repair.rs`: `MeshRepair` trait for geometry fixes
  - `slice.rs`: `SliceStack` for 2D layer data
  - `volumetric.rs`: `VolumetricStack` for voxel data
  - `stats.rs`: Statistics computation interface
  - `units.rs`: Unit enum and conversion

**crates/lib3mf-core/src/parser/:**
- Purpose: XML → Model conversion; handles all extension-specific parsing
- Contains: Modular parsers for each element type
- Key files:
  - `model_parser.rs`: Orchestrates top-level parsing (resources, build, metadata)
  - `mesh_parser.rs`: Triangle mesh parsing
  - `material_parser.rs`: Material group parsing
  - `build_parser.rs`: Build item and positioning parsing
  - `beamlattice_parser.rs`: Beam Lattice Extension
  - `slice_parser.rs`: Slice Extension
  - `volumetric_parser.rs`: Volumetric Extension
  - `secure_content_parser.rs`: Secure Content Extension
  - `crypto_parser.rs`: Digital signature parsing
  - `component_parser.rs`: Hierarchical assembly parsing
  - `xml_parser.rs`: Low-level XML event handling wrapper
  - `streaming.rs`: SAX-based streaming parser
  - `visitor.rs`: Visitor trait for streaming callbacks
- Design: Each parser function takes `XmlParser` by mutable reference and returns parsed structure

**crates/lib3mf-core/src/validation/:**
- Purpose: Progressive validation at four strictness levels
- Contains: Schema checks, semantic validation, geometry analysis, BVH acceleration
- Key files:
  - `schema.rs`: Basic structural validation
  - `semantic.rs`: Reference integrity and constraint checking
  - `geometry.rs`: Manifoldness, self-intersection, orientation consistency
  - `bvh.rs`: Bounding Volume Hierarchy for O(n log n) triangle tests
  - `report.rs`: `ValidationReport` and `ValidationSeverity` structures
- Entry: `model.validate(level)` returns `ValidationReport`

**crates/lib3mf-core/src/writer/:**
- Purpose: Model → XML/ZIP serialization; mirrors parser structure
- Contains: Writer implementations for each element type
- Key files:
  - `model_writer.rs`: Root model XML generation
  - `mesh_writer.rs`: Triangle mesh serialization
  - `opc_writer.rs`: OPC relationship generation
  - `package_writer.rs`: ZIP archive orchestration
  - `xml_writer.rs`: Low-level XML generation helper
- Entry: `model.write(file)` writes 3MF ZIP file

**crates/lib3mf-core/src/utils/:**
- Purpose: Cross-cutting utilities
- Contains: Model diffing, C14N canonicalization, hardware detection, statistics
- Key files: `diff.rs`, `c14n.rs`, `hardware.rs`

**crates/lib3mf-core/examples/:**
- Purpose: API usage demonstration and integration testing
- Files: `create_cube.rs`, `advanced_materials.rs`, `geometry_validation.rs`, `geometry_repair.rs`, `secure_content.rs`, `beam_lattice_ext.rs`, `slice_data.rs`, `streaming_stats.rs`
- Each example is independently runnable: `cargo run -p lib3mf-core --example <name>`

**crates/lib3mf-core/tests/:**
- Purpose: Integration tests covering features and workflows
- Files: `roundtrip_*.rs` (parse → write → reparse), `repair_test.rs`, `validation_test.rs`, `beamlattice_test.rs`, `slice_test.rs`, `production_test.rs`, `secure_content_test.rs`, `proptests.rs` (property-based robustness)
- Format: Standard Rust integration tests using `#[test]` and `#[cfg(test)]`

**crates/lib3mf-cli/src/:**
- Purpose: Command-line interface
- Key files:
  - `main.rs`: Argument parsing via `clap`, command routing
  - `commands.rs`: Implementation of all CLI commands (stats, list, validate, copy, diff, extract, repair, split, thumbnails)
- Pattern: Each command function takes args and returns `anyhow::Result<()>`

**crates/lib3mf-converters/src/:**
- Purpose: Format conversion to/from 3MF
- Files: `stl.rs` (STL ↔ 3MF), `obj.rs` (OBJ ↔ 3MF)
- Pattern: `StlImporter::read(file)`, `ObjImporter::read(file)` return `lib3mf_core::Model`

**crates/lib3mf-async/src/:**
- Purpose: Async/await support for file I/O
- Files: `loader.rs` (tokio-based model loading), `archive.rs` (async traits), `zip.rs` (async-zip wrapper)

**crates/lib3mf-wasm/src/:**
- Purpose: WebAssembly bindings
- Wraps: Core functionality in `WasmModel` type for JavaScript interoperability

**models/:**
- Purpose: Test and sample 3MF files for integration testing and validation
- Format: Real 3MF files used by integration tests and CLI examples

**scripts/:**
- Purpose: Automation and CI/CD
- Files: `qa_test_suite.sh` (comprehensive build, lint, test, benchmark, CLI validation)

**fuzz/:**
- Purpose: Fuzzing infrastructure for security testing
- Target: `parse_model` fuzzing (malformed XML input robustness)

## Key File Locations

**Entry Points:**
- `crates/lib3mf-core/src/lib.rs`: Library public API
- `crates/lib3mf-cli/src/main.rs`: CLI entry point
- `crates/lib3mf-async/src/lib.rs`: Async API
- `crates/lib3mf-wasm/src/lib.rs`: WebAssembly API
- `crates/lib3mf-core/src/parser/model_parser.rs`: `parse_model()` function

**Configuration:**
- `Cargo.toml`: Workspace and crate dependencies
- `rust-toolchain.toml`: Rust version (1.70+)
- `.editorconfig`: Editor formatting
- `CLAUDE.md`: Development guide

**Core Logic:**
- `crates/lib3mf-core/src/model/core.rs`: `Model` root struct
- `crates/lib3mf-core/src/model/resources.rs`: `ResourceCollection` registry
- `crates/lib3mf-core/src/archive/zip_archive.rs`: ZIP reading
- `crates/lib3mf-core/src/parser/model_parser.rs`: Top-level parser
- `crates/lib3mf-core/src/writer/model_writer.rs`: Serialization
- `crates/lib3mf-core/src/validation/geometry.rs`: Geometry checks

**Testing:**
- `crates/lib3mf-core/tests/`: Integration test directory
- `crates/lib3mf-core/examples/`: Example programs (also serve as tests)
- `crates/lib3mf-core/benches/`: Performance benchmarks
- `crates/lib3mf-core/fuzz/fuzz_targets/`: Fuzzing targets
- `models/`: Sample 3MF files for testing

## Naming Conventions

**Files:**
- Module organization: `foo.rs` for single-responsibility modules, `foo/mod.rs` for module directories
- Test files: `*_test.rs` (e.g., `repair_test.rs`)
- Example files: descriptive names in `examples/` (e.g., `create_cube.rs`)
- Parser files: `*_parser.rs` for element-specific parsers (e.g., `mesh_parser.rs`)

**Directories:**
- Functional grouping: `archive/`, `parser/`, `model/`, `validation/`, `writer/`
- Multi-crate workspace: `crates/<crate-name>/`
- Test location: `tests/` alongside source or `#[cfg(test)]` inline modules

**Code Style:**
- Functions: `snake_case` (e.g., `parse_model`, `add_vertex`)
- Types: `PascalCase` (e.g., `Model`, `Mesh`, `ValidationLevel`)
- Constants: `UPPER_SNAKE_CASE` (e.g., `MIN_VERTEX_COUNT`)
- Module re-exports: `pub use module::*;` in parent `lib.rs` or `mod.rs`

## Where to Add New Code

**New Feature (e.g., new material property type):**
- Primary code: `crates/lib3mf-core/src/model/<category>.rs` (add struct, enum variant)
- Parser: `crates/lib3mf-core/src/parser/<category>_parser.rs` (add parsing logic)
- Writer: `crates/lib3mf-core/src/writer/<category>_writer.rs` (add serialization)
- Tests: `crates/lib3mf-core/tests/<feature>_test.rs` (roundtrip and validation tests)
- Example: `crates/lib3mf-core/examples/<feature>_example.rs` (demonstrate usage)

**New CLI Command (e.g., new analysis):**
- Implementation: `crates/lib3mf-cli/src/commands.rs` (add function)
- Command enum: `crates/lib3mf-cli/src/main.rs` (add variant to `Commands`)
- Tests: `crates/lib3mf-cli/tests/` (integration tests for command behavior)

**New Converter Format (e.g., STEP):**
- Primary code: `crates/lib3mf-converters/src/<format>.rs`
- Module export: `crates/lib3mf-converters/src/lib.rs` (add `pub mod`)
- Example: `crates/lib3mf-converters/examples/<format>_conversion.rs`
- Tests: `crates/lib3mf-converters/tests/` (format conversion tests)

**New Extension (e.g., vendor-specific namespace):**
- Data structures: `crates/lib3mf-core/src/model/<ext>.rs` (new file)
- Parser: `crates/lib3mf-core/src/parser/<ext>_parser.rs` (new file)
- Writer: `crates/lib3mf-core/src/writer/<ext>_writer.rs` (new file)
- Hook: Update `model_parser.rs` to call extension parser
- Hook: Update `model_writer.rs` to serialize extension data
- Validation: `crates/lib3mf-core/src/validation/` (add validation if needed)
- Tests: `crates/lib3mf-core/tests/<ext>_test.rs`

**Shared Utility:**
- Location: `crates/lib3mf-core/src/utils/` (new file or extend existing)
- Re-export: `crates/lib3mf-core/src/utils/mod.rs`
- Usage: Import via `crate::utils::function_name`

**Utilities and Helpers:**
- Math/geometry: `crates/lib3mf-core/src/utils/` (use `glam` for vectors/matrices)
- Validation rules: `crates/lib3mf-core/src/validation/` (add to appropriate module)
- Error types: `crates/lib3mf-core/src/error.rs` (add variant to `Lib3mfError`)

## Special Directories

**crates/lib3mf-core/benches/:**
- Purpose: Performance benchmarks using Rust's `criterion` crate
- Generated: No (checked in)
- Committed: Yes
- Run: `cargo bench -p lib3mf-core`

**crates/lib3mf-core/fuzz/:**
- Purpose: Fuzzing targets for security/robustness testing
- Generated: No (checked in)
- Committed: Yes
- Run: `cargo fuzz run parse_model`
- Infrastructure: Uses `cargo-fuzz` and `libFuzzer`

**crates/lib3mf-core/target/:**
- Purpose: Build artifacts (compiled binaries, intermediate objects)
- Generated: Yes
- Committed: No (in `.gitignore`)
- Location: Git-ignored; regenerated on each build

**models/:**
- Purpose: Test and sample 3MF files
- Generated: No (real files, checked in)
- Committed: Yes (versioned with tests)
- Used by: Integration tests, examples, CLI validation

**scripts/:**
- Purpose: Development automation
- Files: `qa_test_suite.sh` - comprehensive QA pipeline
- Run: `./scripts/qa_test_suite.sh`

---

*Structure analysis: 2026-02-02*
