# Coding Conventions

**Analysis Date:** 2026-02-02

## Naming Patterns

**Files:**
- Snake case for all Rust files: `model_parser.rs`, `mesh_writer.rs`, `beamlattice_parser.rs`
- Module files: `core.rs`, `mesh.rs`, `materials.rs`, `resources.rs`
- Test files: Descriptive names like `roundtrip_benchy.rs`, `error_scenarios.rs`, `verification_test.rs`
- Examples use snake case: `create_cube.rs`, `geometry_validation.rs`, `streaming_stats.rs`

**Functions:**
- Snake case for all function names: `parse_model()`, `add_vertex()`, `compute_aabb()`, `read_next_event()`
- Private helper functions use snake case prefix: `parse_resources()`, `validate_mesh()`, `check_manifoldness()`
- Getter methods use simple names: `get_object()`, `get_attribute()`, `get_attribute_f32()`
- Parser/validator functions: `parse_model()`, `validate_geometry()`, `parse_base_materials()`

**Variables:**
- Snake case for all local and instance variables: `model_path`, `resource_id`, `vertex_count`, `triangle_indices`
- Constant names use UPPER_SNAKE_CASE: Not heavily used in codebase but follows Rust convention
- Loop variables use short descriptive names: `i`, `j`, or meaningful names like `tri`, `vertex`, `object`

**Types:**
- PascalCase for struct/enum names: `Model`, `Mesh`, `Object`, `ResourceId`, `Lib3mfError`
- PascalCase for trait names: `ArchiveReader`, `ArchiveWriter`, `MeshRepair`
- Newtype wrappers use PascalCase: `ResourceId(u32)`
- Enum variants use PascalCase: `ValidationLevel::Standard`, `Unit::Millimeter`, `Geometry::Mesh`

## Code Style

**Formatting:**
- Tool: `rustfmt` with custom config in `rustfmt.toml`
- Maximum line width: 100 characters
- Edition: 2021 (defined in `rustfmt.toml` and workspace)
- Indentation: 4 spaces (Rust default)
- Run formatting: `cargo fmt`

**Linting:**
- Tool: `clippy`
- Run linter: `cargo clippy -- -D warnings`
- Enforces no warnings in CI/QA suite
- Common patterns checked: unused imports, trivial patterns, complexity

**Derive Attributes:**
- Standard derives: `#[derive(Debug, Clone, PartialEq, Eq)]`
- With serialization: `#[derive(Debug, Clone, Serialize, Deserialize)]`
- Used throughout: `core.rs`, `mesh.rs`, `materials.rs`

**Trait Implementations:**
- Use explicit `impl` blocks for custom logic
- Use `#[derive(...)]` for auto-derivable traits
- Error types use `#[derive(Error, Debug)]` with `thiserror` crate

## Import Organization

**Order:**
1. Internal crate imports: `use crate::model::...`, `use crate::error::...`
2. Standard library imports: `use std::io::...`, `use std::collections::HashMap`
3. External crate imports: `use serde::...`, `use quick_xml::...`, `use glam::Vec3`
4. Relative imports within module: `use super::...`

**Examples:**

From `src/parser/model_parser.rs`:
```rust
use crate::error::{Lib3mfError, Result};
use crate::model::{Geometry, Model, Object, Unit};
use crate::parser::build_parser::parse_build;
use crate::parser::component_parser::parse_components;
use crate::parser::material_parser::{...};
use quick_xml::events::Event;
use std::io::BufRead;
```

From `src/model/core.rs`:
```rust
use super::units::Unit;
use crate::model::{Build, ResourceCollection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
```

**Path Aliases:**
- None configured in workspace
- Absolute paths preferred: `crate::model::`, `crate::parser::`, `crate::validation::`

## Error Handling

**Strategy:** Custom error enum with `thiserror` for ergonomic error definitions

**Error Type:** `Lib3mfError` enum defined in `src/error.rs`
```rust
#[derive(Error, Debug)]
pub enum Lib3mfError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(u32),

    #[error("Invalid 3MF structure: {0}")]
    InvalidStructure(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),
}
```

**Result Type Alias:**
- `pub type Result<T> = std::result::Result<T, Lib3mfError>;`
- Used consistently throughout all public APIs
- Location: `src/error.rs`

**Patterns:**
- Functions return `Result<T>` for fallible operations
- Use `?` operator for error propagation: `let val = parse_u32(&bytes)?;`
- Use `.map_err()` to convert error types: `.map_err(|e| Lib3mfError::Validation(e.to_string()))?`
- Validation errors include context: `Lib3mfError::Validation(format!("Duplicate resource ID: {}", id))`
- No panic-based error handling in library code (3 unwrap() calls in entire `src/` directory, in repair logic)

**No unwrap() in library:**
- Library code uses `Result<T>` exclusively
- Only 3 `unwrap()` calls total in `src/`:
  - `src/model/repair.rs:218` - in controlled mesh repair context
  - `src/model/resolver.rs:74` - part resolution with guaranteed entries
  - `src/model/resolver.rs:78` - root model access with guaranteed key
- CLI and tests may use `.unwrap()` when errors are test fixtures

## Logging

**Framework:** `std::io::Write` for output in CLI, `println!()` for examples

**Patterns:**
- No centralized logging framework in library code
- CLI uses direct output: `println!()`, `eprintln!()`
- Examples use `println!()` for diagnostics
- Error messages from `Lib3mfError` use `Display` trait via `thiserror`

**Example from CLI:**
```rust
println!("ORIG Objects: {}, Items: {}",
    model.resources.iter_objects().count(),
    model.build.items.len());
```

## Comments

**When to Comment:**
- Document public API functions with `///` doc comments
- Explain non-obvious algorithms or workarounds
- Flag limitations or known issues (minimal in current codebase)
- Rarely used for obvious code

**Doc Comments (///):**
- Used extensively on public types and functions
- Appear in `src/model/`, `src/archive/`, `src/utils/`
- Examples:
  - `src/model/core.rs`: Documented all `Model` fields
  - `src/parser/visitor.rs`: Documented all visitor callback methods
  - `src/utils/c14n.rs`: Documented canonicalization implementation

**Example:**
```rust
/// Root element of a 3MF document.
///
/// The `Model` contains all information required to describe a 3D model, including:
/// - Resources (Meshes, Materials, Textures)
/// - Build instructions (Item positioning)
/// - Metadata (Authors, Copyright, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// The unit of measurement for geometry coordinates.
    pub unit: Unit,
    /// The language of the model content (e.g., "en-US").
    pub language: Option<String>,
    // ...
}
```

**Inline Comments:**
- Sparse use; code is written to be self-documenting
- Used for algorithm explanation or specification compliance notes
- Example in `src/validation/geometry.rs`: Comments on BVH tree construction

## Function Design

**Size:** Typically 20-80 lines for parser/validator functions
- Smaller utility functions: 5-15 lines
- Larger functions (e.g., `parse_model()`): 150-250 lines, broken into helpers
- Complex geometry algorithms: 100-200 lines with clear variable names

**Parameters:**
- Functions take type-specific params rather than generic structs
- Parser functions take `&mut XmlParser<R>` for state management
- Validators take `&Model` for read-only access
- Writers take `&mut Write` for output

**Return Values:**
- Use `Result<T>` for fallible operations exclusively
- Builders return `()` with mutation through references
- Computational functions return single values or tuples
- Iterators use `impl Iterator` where appropriate

**Mutability:**
- Parser state is mutable: `&mut parser`
- Model is immutable by default (Clone semantics for modification)
- Repair operations mutate mesh in-place and return stats

## Module Design

**Exports (pub use):**
- `src/lib.rs`: Re-exports main types and error types
```rust
pub use error::{Lib3mfError, Result};
pub use model::*;
```
- Modules re-export their primary types
- Parser helpers are private: `use crate::parser::xml_parser::get_attribute;` (internal use)

**Barrel Files:**
- `src/model/mod.rs`: Aggregates and re-exports all model types
- `src/parser/mod.rs`: Aggregates parser modules
- Keeps public API focused and organized

**Module Structure:**
- Functional modules grouped by responsibility: `archive/`, `parser/`, `model/`, `validation/`, `writer/`, `crypto/`, `utils/`
- Each module has its own `mod.rs` or single file depending on size
- Large modules like `parser/` split into sub-modules: `model_parser.rs`, `mesh_parser.rs`, `material_parser.rs`

**Example Module Hierarchy:**
```
crates/lib3mf-core/src/
├── lib.rs                      (main exports)
├── error.rs                    (error types)
├── archive/
│   ├── mod.rs                 (trait definitions)
│   ├── zip_archive.rs         (implementation)
│   └── opc.rs                 (OPC parsing)
├── parser/
│   ├── mod.rs                 (re-exports)
│   ├── model_parser.rs        (main entry point)
│   ├── mesh_parser.rs
│   ├── material_parser.rs
│   ├── xml_parser.rs          (utilities)
│   └── visitor.rs             (trait)
└── model/
    ├── mod.rs                 (re-exports)
    ├── core.rs                (Model struct)
    ├── mesh.rs                (Mesh, Vertex, Triangle)
    └── materials.rs           (Color, Material types)
```

## Serialization

**Framework:** `serde` with `serde_json` support
- Structures derive `#[derive(Serialize, Deserialize)]`
- Used for internal data structure representation, not public API serialization
- XML parsing/writing handled separately via `quick_xml`

**Attributes:**
- `#[serde(default)]`: Used when field can be omitted
- `#[serde(skip)]`: Skip binary attachments, relationships
- `#[serde(skip_serializing_if = "Option::is_none")]`: Only serialize present options
- `#[serde(rename_all = "lowercase")]`: Enum variant naming

## Visibility

**Public vs Private:**
- Public: Types and methods intended for external use
- Private (default): Helper functions, internal state
- Crate-visible: `pub(crate)` used sparingly for internal APIs
- Module-visible: Functions kept private to module unless re-exported

**Example:**
- `pub fn parse_model()` - public entry point
- `fn parse_resources()` - private helper
- `pub struct Model` - public type
- `pub(crate) fn validate_schema()` - internal validation API

---

*Convention analysis: 2026-02-02*
