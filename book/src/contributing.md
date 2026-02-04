# Contributing

Thank you for your interest in contributing to lib3mf-rs! This guide covers development setup, testing strategies, code conventions, and how to add new features.

## Development Setup

### Prerequisites

- Rust toolchain (stable, v1.70 or later)
- Git

### Clone and Build

```bash
git clone https://github.com/sscargal/lib3mf-rs.git
cd lib3mf-rs

# Debug build (fast compilation)
cargo build

# Run tests
cargo test

# Run linter
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Project Structure

```text
lib3mf-rs/
├── crates/
│   ├── lib3mf-core/        # Main library
│   ├── lib3mf-cli/         # CLI tool
│   ├── lib3mf-converters/  # Format converters
│   ├── lib3mf-async/       # Async I/O
│   └── lib3mf-wasm/        # WebAssembly bindings
├── fuzz/                   # Fuzzing targets
├── book/                   # This documentation
├── scripts/                # QA and utility scripts
└── .github/workflows/      # CI/CD pipelines
```

## Build Commands

### Standard Builds

```bash
# Debug build (fast compilation, slower execution)
cargo build

# Release build (optimized for performance)
cargo build --release

# Build specific crate
cargo build -p lib3mf-core
cargo build -p lib3mf-cli
```

### Feature-Specific Builds

```bash
# Minimal build (no optional dependencies)
cargo build -p lib3mf-core --no-default-features

# With crypto support
cargo build -p lib3mf-core --features crypto

# With parallel processing
cargo build -p lib3mf-core --features parallel

# All features
cargo build -p lib3mf-core --all-features
```

### Check Dependency Count

```bash
# Verify minimal build dependency count
cargo tree -p lib3mf-core --no-default-features | wc -l
# Expected: ~154 crates

# Full-featured build
cargo tree -p lib3mf-core --all-features | wc -l
# Expected: ~300 crates
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p lib3mf-core

# Run tests with specific features
cargo test -p lib3mf-core --no-default-features
cargo test -p lib3mf-core --features crypto
cargo test -p lib3mf-core --all-features

# Run property-based tests
cargo test -p lib3mf-core --test proptests
```

### Test Organization

**Unit tests** — Located in same file as code, in `#[cfg(test)] mod tests` blocks:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vertex() {
        // Test implementation
    }
}
```

**Integration tests** — Located in `tests/` directories:

```text
crates/lib3mf-core/tests/
├── parse_tests.rs          # Full file parsing
├── roundtrip_tests.rs      # Parse → Write → Parse
├── validation_tests.rs     # Validation system
└── extension_tests.rs      # Extension-specific tests
```

**Property-based tests** — Using `proptest` for robustness:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_vertex_roundtrip(x in -1000.0f32..1000.0, y in -1000.0..1000.0, z in -1000.0..1000.0) {
        let vertex = Vertex::new(x, y, z);
        let xml = write_vertex(&vertex);
        let parsed = parse_vertex(&xml)?;
        assert_eq!(vertex, parsed);
    }
}
```

### Fuzzing

Fuzzing tests for security and robustness using cargo-fuzz:

**Setup:**

```bash
# Install nightly toolchain (required for fuzzing)
rustup toolchain install nightly

# Install cargo-fuzz
cargo +nightly install cargo-fuzz
```

**Running fuzz targets:**

```bash
# List available targets
cargo +nightly fuzz list

# Run a specific target (indefinitely until Ctrl+C)
cargo +nightly fuzz run parse_model

# Run with time limit (60 second smoke test)
cargo +nightly fuzz run parse_model -- -max_total_time=60

# Run with dictionary for better coverage
cargo +nightly fuzz run parse_xml -- -dict=fuzz/dictionaries/3mf.dict
```

**Available fuzz targets:**
- `parse_model` — Full 3MF file parsing (ZIP + XML + invariants)
- `parse_xml` — Direct XML model parsing (bypasses ZIP)
- `parse_materials` — Material/texture parsing isolation
- `parse_crypto` — Signature/encryption parsing
- `parse_extensions` — Extension-specific parsers
- `parse_opc` — OPC relationship parsing
- `writer_roundtrip` — Fuzz writer by round-tripping models

**Crash handling:**

If fuzzing finds a crash:

```bash
# Minimize crash file (reduce to smallest reproducer)
cargo +nightly fuzz tmin parse_model fuzz/artifacts/parse_model/crash-abc123

# Add to regression tests
cp fuzz/artifacts/parse_model/crash-abc123 crates/lib3mf-core/tests/fuzz_regression/parse_model_001.bin

# Create test case
#[test]
fn test_fuzz_regression_001() {
    let data = include_bytes!("fuzz_regression/parse_model_001.bin");
    // Verify it doesn't crash or produces expected error
    let result = parse_model(Cursor::new(data));
    assert!(result.is_err());  // Should error gracefully, not panic
}
```

### QA Test Suite

Run comprehensive validation before submitting PRs:

```bash
./scripts/qa_test_suite.sh
```

This script runs:
1. Format check (`cargo fmt --check`)
2. Linter (`cargo clippy`)
3. All tests with all feature combinations
4. Benchmarks
5. Example runs
6. CLI validation

Expected runtime: ~5 minutes

## Code Quality Standards

### Formatting

Use `rustfmt` with default settings:

```bash
cargo fmt
```

**Before committing:**

```bash
cargo fmt --check
```

### Linting

Use `clippy` with warnings as errors:

```bash
cargo clippy -- -D warnings
```

**Common clippy warnings to avoid:**
- Unnecessary clones
- Inefficient string operations
- Missing error propagation
- Unsafe code without justification

### Documentation

**Public items must have doc comments:**

```rust
/// Parses a 3MF model from an XML reader.
///
/// # Arguments
///
/// * `reader` - The XML data source
///
/// # Returns
///
/// A `Model` instance on success, or `Lib3mfError` on parse failure.
///
/// # Examples
///
/// ```
/// use lib3mf_core::parser::parse_model;
/// use std::io::Cursor;
///
/// let xml = r#"<model>...</model>"#;
/// let model = parse_model(Cursor::new(xml))?;
/// # Ok::<(), lib3mf_core::error::Lib3mfError>(())
/// ```
pub fn parse_model<R: Read>(reader: R) -> Result<Model> {
    // Implementation
}
```

**Module-level documentation:**

```rust
//! Mesh repair utilities for fixing geometry issues.
//!
//! This module provides the `MeshRepair` trait and implementations
//! for common repair operations like vertex stitching and orientation
//! harmonization.
```

## Code Conventions

### Immutable Design

Prefer immutable data structures:

```rust
// Good: Return new instance
pub fn stitch_vertices(&self, epsilon: f32) -> Result<Mesh> {
    let mut new_mesh = self.clone();
    // Perform stitching on new_mesh
    Ok(new_mesh)
}

// Avoid: Mutate in place (unless explicitly required)
pub fn stitch_vertices_mut(&mut self, epsilon: f32) {
    // Avoid this pattern
}
```

**Rationale:** Thread safety, predictable behavior, easier testing.

### Error Handling

Use `Result<T>` for fallible operations:

```rust
use crate::error::{Lib3mfError, Result};

pub fn parse_vertex(xml: &str) -> Result<Vertex> {
    let x = xml.parse::<f32>()
        .map_err(|e| Lib3mfError::ParseError(format!("Invalid X coordinate: {}", e)))?;
    // ...
    Ok(Vertex { x, y, z })
}
```

**Never panic in library code:**

```rust
// Bad
pub fn get_object(&self, id: ResourceId) -> Object {
    self.objects.get(&id).unwrap()  // DON'T DO THIS
}

// Good
pub fn get_object(&self, id: ResourceId) -> Option<&Object> {
    self.objects.get(&id)
}

// Or
pub fn get_object(&self, id: ResourceId) -> Result<&Object> {
    self.objects.get(&id)
        .ok_or_else(|| Lib3mfError::ResourceNotFound(id))
}
```

### Trait Abstractions

Use traits for extensibility:

```rust
pub trait ArchiveReader {
    fn read_entry(&mut self, path: &str) -> Result<Vec<u8>>;
    fn entry_names(&mut self) -> Result<Vec<String>>;
}

impl ArchiveReader for ZipArchiver {
    // Implementation
}
```

### Resource Management

Use newtype pattern for type-safe IDs:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(pub u32);

// Prevents mixing different ID types
```

## Adding New Features

### Adding a New Extension

Extensions require integration across multiple layers:

1. **Define data structures** (`model/` module):

```rust
// In model/geometry.rs or new file
pub struct MyExtensionData {
    pub property_a: String,
    pub property_b: f32,
}

// Add to Geometry enum if geometry-related
pub enum Geometry {
    Mesh(Mesh),
    BeamLattice(BeamLattice),
    MyExtension(MyExtensionData),  // Add here
}
```

2. **Create parser** (`parser/my_extension_parser.rs`):

```rust
use quick_xml::Reader;
use crate::model::MyExtensionData;

pub fn parse_my_extension<R: BufRead>(reader: &mut Reader<R>) -> Result<MyExtensionData> {
    // XML parsing logic
}
```

3. **Integrate into main parser** (`parser/model_parser.rs`):

```rust
match event {
    // Existing cases...
    Start(ref e) if e.local_name().as_ref() == b"myextension" => {
        let ext_data = parse_my_extension(reader)?;
        // Add to model
    }
}
```

4. **Add writer** (`writer/my_extension_writer.rs`):

```rust
pub fn write_my_extension<W: Write>(writer: &mut Writer<W>, data: &MyExtensionData) -> Result<()> {
    // XML writing logic
}
```

5. **Add validation** (`validation/validator.rs`):

```rust
fn validate_my_extension(&self, data: &MyExtensionData, report: &mut ValidationReport) {
    if data.property_b < 0.0 {
        report.add_error(5100, "Property B must be non-negative");
    }
}
```

6. **Write tests** (`tests/my_extension_tests.rs`):

```rust
#[test]
fn test_my_extension_roundtrip() {
    let xml = r#"<myextension property_a="test" property_b="1.5" />"#;
    let parsed = parse_my_extension(xml)?;
    assert_eq!(parsed.property_a, "test");
    assert_eq!(parsed.property_b, 1.5);
}
```

7. **Add example** (`examples/my_extension_demo.rs`):

```rust
//! Demonstrates using MyExtension feature

use lib3mf_core::Model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::from_file("example.3mf")?;
    // Show extension usage
    Ok(())
}
```

8. **Update documentation**:
   - Add to `book/src/extensions.md`
   - Update rustdoc comments
   - Update README.md feature list

### Adding a New Resource Type

Similar process but focuses on `ResourceCollection`:

1. Define struct in `model/resources.rs`
2. Add to `ResourceCollection` with `add_*/get_*` methods
3. Implement parser and writer
4. Add validation rules
5. Write tests and examples

## Feature Flag Testing

Test all feature combinations in CI:

```bash
# Test minimal build
cargo test -p lib3mf-core --no-default-features

# Test crypto only
cargo test -p lib3mf-core --no-default-features --features crypto

# Test parallel only
cargo test -p lib3mf-core --no-default-features --features parallel

# Test all features
cargo test -p lib3mf-core --all-features
```

**CI matrix (`.github/workflows/ci.yml`):**

```yaml
strategy:
  matrix:
    features:
      - "--no-default-features"
      - "--features crypto"
      - "--features parallel"
      - "--all-features"
```

## Pull Request Process

1. **Fork and clone** the repository
2. **Create a feature branch** (`git checkout -b feature/my-feature`)
3. **Make changes** following code conventions
4. **Write tests** (unit + integration)
5. **Run QA suite** (`./scripts/qa_test_suite.sh`)
6. **Commit** with clear messages:
   ```
   feat: add MyExtension support

   - Parse myextension elements
   - Add MyExtensionData to Geometry enum
   - Write tests and example
   - Update documentation
   ```
7. **Push** and create PR
8. **Address review feedback**

## Finding Work

Check GitHub Issues for:
- **Good First Issue** — Beginner-friendly tasks
- **Help Wanted** — Community contributions welcome
- **Bug** — Reported bugs needing fixes
- **Enhancement** — Feature requests

Or propose your own improvements!

## Questions?

- **GitHub Issues** — For bugs and feature requests
- **GitHub Discussions** — For questions and ideas
- **Email** — steve@scargall.com

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
