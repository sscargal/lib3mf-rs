# Testing Patterns

**Analysis Date:** 2026-02-02

## Test Framework

**Runner:**
- Rust built-in test runner with `#[test]` attribute
- External frameworks: `proptest` for property-based testing, `criterion` for benchmarking
- No mocking framework used; tests build real objects or use fixtures

**Assertion Library:**
- Rust standard library `assert!()`, `assert_eq!()`, `assert_ne!()`
- `proptest` for property-based assertions: `prop_assert_eq!()`, `prop_assert!()`
- `criterion` for performance comparisons

**Run Commands:**
```bash
cargo test                        # Run all tests
cargo test -p lib3mf-core        # Run tests for specific crate
cargo test --test proptests      # Run property-based tests only
cargo test -- --nocapture        # Show println! output
cargo test -- --include-ignored  # Run ignored tests
cargo bench -p lib3mf-core       # Run benchmarks
cargo test --doc                 # Run doc comment examples
```

## Test File Organization

**Location:**
- Integration tests: `crates/lib3mf-core/tests/` (separate directory)
- Unit tests: In-module with `#[cfg(test)] mod tests { ... }` (used sparingly)
- Benchmarks: `crates/lib3mf-core/benches/core_bench.rs` with criterion configuration
- Examples: `crates/lib3mf-core/examples/` for API demonstration and integration validation

**Naming:**
- Test files match functionality: `roundtrip_benchy.rs`, `error_scenarios.rs`, `repair_test.rs`
- Test functions use `test_` prefix: `test_roundtrip_benchy()`, `test_mesh_verification()`
- Descriptive names indicate what is tested: `test_vertex_stitching()`, `test_degenerate_removal()`

**File Count:**
- 22 test/benchmark files in `crates/lib3mf-core/tests/` directory
- 34 total test functions across test suite
- 8 example files demonstrating API usage

**Test Files:**
```
crates/lib3mf-core/tests/
├── proptests.rs                 # Property-based testing (proptest)
├── roundtrip_benchy.rs          # Full model read/write cycle
├── error_scenarios.rs           # Error handling and edge cases
├── verification_test.rs         # Mesh geometry verification
├── repair_test.rs               # Mesh repair operations
├── beamlattice_test.rs          # Beam Lattice extension
├── slice_test.rs                # Slice Stack extension
├── volumetric_test.rs           # Volumetric extension
├── production_test.rs           # Production extension
├── secure_content_test.rs       # Digital signatures
├── encryption_test.rs           # Encryption operations
├── crypto_test.rs               # Cryptographic utilities
├── keys_test.rs                 # Key management
├── streaming_test.rs            # Streaming parser
├── validation_test.rs           # Schema and semantic validation
├── parse_benchy.rs              # Parser performance benchmark
├── stats_benchy.rs              # Statistics computation benchmark
└── [more integration tests...]
```

## Test Structure

**Suite Organization:**

Standard test file structure:
```rust
use lib3mf_core::model::{...};  // Imports at top

#[test]
fn test_functionality() {
    // Arrange: Create test data
    let mut mesh = Mesh::new();
    mesh.add_vertex(0.0, 0.0, 0.0);

    // Act: Call functionality
    let result = mesh.compute_aabb();

    // Assert: Verify result
    assert_eq!(result.min, [0.0, 0.0, 0.0]);
}
```

**Patterns:**

*Setup Pattern* - Direct object construction:
```rust
#[test]
fn test_mesh_construction() {
    let mut mesh = Mesh::new();
    let v1 = mesh.add_vertex(0.0, 0.0, 0.0);
    let v2 = mesh.add_vertex(1.0, 0.0, 0.0);
    let v3 = mesh.add_vertex(0.0, 1.0, 0.0);

    mesh.add_triangle(v1, v2, v3);

    assert_eq!(mesh.vertices.len(), 3);
    assert_eq!(mesh.triangles.len(), 1);
}
```

*Teardown Pattern* - None required (RAII cleanup, test file system cleanup explicit):
```rust
#[test]
fn test_error_invalid_zip() {
    let path = std::env::temp_dir().join("test_invalid.3mf");
    std::fs::write(&path, "This is not a zip file").unwrap();

    let file = File::open(&path).unwrap();
    let result = ZipArchiver::new(file);
    assert!(result.is_err());

    let _ = std::fs::remove_file(path);  // Explicit cleanup
}
```

*Assertion Pattern* - Direct assertions with clear messages:
```rust
assert!(resources.add_object(object.clone()).is_ok());
assert!(resources.add_object(object).is_err());  // Duplicate ID

assert!(resources.get_object(ResourceId(1)).is_some());
assert!(resources.get_object(ResourceId(2)).is_none());
```

## Mocking

**Framework:** None (manual test fixtures instead)

**Patterns:**

*Object Construction:* Create real objects with minimal setup
```rust
let object = Object {
    id: ResourceId(1),
    name: Some("Test Object".to_string()),
    part_number: None,
    uuid: None,
    pid: None,
    pindex: None,
    geometry: Geometry::Mesh(mesh),
};
```

*Test Doubles:* Use real types with test values
```rust
// No mocking - build real Mesh with minimal geometry
let mut mesh = Mesh::new();
mesh.add_vertex(0.0, 0.0, 0.0);
mesh.add_vertex(1.0, 0.0, 0.0);
mesh.add_vertex(0.0, 1.0, 0.0);
mesh.add_triangle(0, 1, 2);
```

**What to Mock:**
- Nothing mocked in current suite (No mock framework used)
- File I/O tests use real temp files: `std::env::temp_dir().join(...)`
- XML parsing tests use string literals for model XML

**What NOT to Mock:**
- Core model types - always construct real objects
- Validation logic - run real validators with test models
- Repair operations - operate on actual mesh data

## Fixtures and Factories

**Test Data:**

Minimal model XML fixture in `core_bench.rs`:
```rust
let root_model = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="100" y="0" z="0" />
                    <vertex x="100" y="100" z="0" />
                    <vertex x="0" y="100" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                    <triangle v2="0" v3="3" v1="2" />
                </triangles>
            </mesh>
        </object>
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"#;
```

Generated fixture example from `streaming_test.rs`:
```rust
let mut xml = String::from(
    r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>"#,
);

for i in 0..1000 {
    xml.push_str(&format!(r#"<vertex x="{}" y="{}" z="{}" />"#, i, i, i));
}
```

**Real Files:**
- Reference models in `models/` directory: `Benchy.3mf` used for integration testing
- Roundtrip test: Parse real file, write to buffer, re-parse, verify consistency

**Location:**
- Inline fixtures: String literals in test files
- Real model files: `models/Benchy.3mf` (shared across test suite)

## Coverage

**Requirements:** No explicit coverage target enforced
- Coverage tracking not configured in test suite
- Comprehensive test coverage focused on critical paths:
  - Parser functions (all main parsers tested)
  - Validation rules (geometry and schema)
  - Repair operations (all repair types)
  - Error scenarios (invalid inputs)

**View Coverage:**
```bash
# No built-in coverage reporting configured
# To add coverage:
cargo install tarpaulin
cargo tarpaulin -p lib3mf-core --out Html
```

## Test Types

**Unit Tests:**
- Scope: Individual functions or small components
- Approach: Direct function calls with simple inputs
- Location: Mostly in `tests/` as integration tests (some in-module)
- Examples:
  - `test_mesh_construction()` - Mesh API
  - `test_scale_factors()` - Unit conversion (in `src/model/units.rs`)
  - `test_conversion()` - Unit conversion roundtrip

**Integration Tests:**
- Scope: Full workflows (parse → validate → write)
- Approach: Load real 3MF files or generate complete model XML
- Location: `crates/lib3mf-core/tests/`
- Examples:
  - `test_roundtrip_benchy()` - Full read/write cycle on Benchy.3mf
  - `test_large_model_streaming()` - Generated 1000-vertex model parsing
  - Extension tests: Beam Lattice, Slice, Volumetric, Production, Secure Content

**Property-Based Tests:**
- Framework: `proptest` 1.6.0
- Location: `tests/proptests.rs`
- Scope: Properties that should hold for generated inputs
- Examples:
  ```rust
  proptest! {
      #[test]
      fn test_vertex_eq(x in -1000.0f32..1000.0, y in -1000.0f32..1000.0, z in -1000.0f32..1000.0) {
          let v1 = Vertex { x, y, z };
          let v2 = Vertex { x, y, z };
          prop_assert_eq!(v1, v2);
      }
  }
  ```

**Benchmark Tests:**
- Framework: `criterion` 0.5
- Location: `benches/core_bench.rs`
- Scope: Performance of parser, statistics, and roundtrip operations
- Named with `*_benchy.rs` suffix: `parse_benchy.rs`, `stats_benchy.rs`, `roundtrip_benchy.rs`
- Run: `cargo bench -p lib3mf-core`

**Example Tests (Integration/API Validation):**
- Location: `examples/` directory
- Serve as both documentation and working tests
- Examples:
  - `create_cube.rs` - Basic model creation
  - `geometry_validation.rs` - Validation API usage
  - `geometry_repair.rs` - Repair operations
  - `secure_content.rs` - Digital signatures and encryption
  - `streaming_stats.rs` - Streaming parser
  - `beam_lattice_ext.rs` - Extension handling

## Common Patterns

**Async Testing:**
Not applicable - library is synchronous. Async I/O available via `lib3mf-async` crate with feature gate.

**Error Testing:**

Pattern for validating error types:
```rust
#[test]
fn test_error_file_not_found() {
    let path = PathBuf::from("non_existent_file.3mf");
    let result = File::open(&path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}
```

Pattern for error matching:
```rust
#[test]
fn test_error_invalid_zip() {
    let result = ZipArchiver::new(file);
    assert!(result.is_err());
    match result.unwrap_err() {
        Lib3mfError::Io(e) => {
            println!("Got expected IO error: {}", e);
        }
        e => panic!("Unexpected error type: {:?}", e),
    }
}
```

**Roundtrip Testing:**

Full cycle validation in `roundtrip_benchy.rs`:
```rust
#[test]
fn test_roundtrip_benchy() -> anyhow::Result<()> {
    // 1. Read Original
    let file = File::open("../../models/Benchy.3mf")?;
    let mut archiver = ZipArchiver::new(file)?;
    let model_path = find_model_path(&mut archiver)?;
    let model_data = archiver.read_entry(&model_path)?;
    let model = parse_model(Cursor::new(model_data))?;

    // 2. Write to memory buffer
    let mut buffer = Cursor::new(Vec::new());
    model.write(&mut buffer)?;

    // 3. Read back from memory buffer
    let mut buffer_archiver = ZipArchiver::new(Cursor::new(buffer.into_inner()))?;
    let model_path_new = find_model_path(&mut buffer_archiver)?;
    let model_data_new = buffer_archiver.read_entry(&model_path_new)?;
    let model_new = parse_model(Cursor::new(model_data_new))?;

    // 4. Assert consistency
    assert_eq!(
        model_new.resources.iter_objects().count(),
        model.resources.iter_objects().count()
    );
}
```

**Mesh Repair Testing:**

Example from `repair_test.rs`:
```rust
#[test]
fn test_vertex_stitching() {
    let mut mesh = Mesh::new();
    // Create vertices with tiny duplicates
    mesh.add_vertex(0.0, 0.0, 0.0);
    mesh.add_vertex(1.0, 0.0, 0.0);
    mesh.add_vertex(1.0, 0.0, 0.00001);  // Duplicate within epsilon
    mesh.add_triangle(0, 1, 2);

    let stats = mesh.repair(RepairOptions::default());
    assert_eq!(stats.vertices_removed, 1);
    assert_eq!(mesh.vertices.len(), 2);
}
```

## QA Test Suite

**Location:** `scripts/qa_test_suite.sh`

**Scope:** Comprehensive validation encompassing:
- Build verification
- Linting with clippy
- All unit and integration tests
- Benchmarks
- CLI validation

**Run:** `./scripts/qa_test_suite.sh`

---

*Testing analysis: 2026-02-02*
