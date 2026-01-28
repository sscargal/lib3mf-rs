Lib3mf-rs is a Rust implementation of the 3D Manufacturing Format file standard.

It provides 3MF reading and writing capabilities, as well as conversion and validation tools for input and output data. lib3mf-rs runs on Windows, Linux, and macOS using x86, amd64, and arm CPUs, and offers a clean and easy-to-use API in various programming languages to speed up the development and keep integration costs at a minimum.

As 3MF is a universal 3D Printing standard, its quick adoption is very important. This library shall lower all barriers of adoption to any possible user, let it be software providers, hardware providers, service providers or middleware tools.

The specification and addendums can be downloaded at http://3mf.io/specification/. The C++ implementation can be found at https://github.com/3MFConsortium/lib3mf. You can clone the code within lib3mf-rs to make it easier to review. You can download the 3MF specification document for easy reference. If you need me to assist, let me know.

Act as a software architect and design and plan the implementation from scratch. Use a multi phased approach, where each phase has multiple steps and tasks. Each task should be validated to ensure the project builds and runs successfully before moving to the next major step. This catches and resolves issues early. The project should include unit tests, lint checks, and code coverage that must all pass. The code must be enterprise quality. The project will use spec-driven development methodology, so you will need to write the necessary markdown documents for each phase, step, and feature that Claude's code or Gemini CLI can easily understand and follow. Consider if the project should use Claude or Gemini skills, and write the necessary instructions if yes. We will not implement code in this session, but we will implement it in a new Claude Code or GeminiCLI session. We will use both Gemini and Claude skills to implement the code.

Claude Skills Documentation - https://code.claude.com/docs/en/skills
Gemini Skills Documentation - https://docs.gemini.com/skills
AGENTS.md Documentation - https://agents.md/
A library of agent skills - https://skills.sh/

Find a universal solution for AI Coding agents without duplication of skills, etc.

What else should we include? Ask me any questions you have throughout the design and planning stages. We will implement the code once the plan has been approved and thoroughly thought out.

---

# Lib3mf-rs Architecture & Implementation Plan

## Initial Questions & Clarifications

Before I finalize the comprehensive architecture plan, I need to clarify several critical aspects:

### 1. **FFI Strategy & Existing C++ Library**

- Should we create Rust bindings to the existing C++ lib3mf library, or implement a pure Rust solution from scratch?
- **Recommendation**: Pure Rust implementation for:
  - Memory safety guarantees
  - Better integration with Rust ecosystem
  - Elimination of C++ runtime dependencies
  - Easier cross-compilation
- **Alternative**: FFI bindings would be faster to market but with compromises

### 2. **Feature Scope for MVP (Phase 1)**

Which capabilities should we prioritize?

- Core 3MF reading (REQUIRED)
- Core 3MF writing (REQUIRED)
- Which extensions/addendums to support initially?
  - Materials & Properties
  - Beam Lattice
  - Slice
  - Production
  - Volumetric
- Validation levels (basic schema vs. full spec compliance)?

### 3. **API Design Philosophy**

- **Builder pattern** for constructing 3MF documents?
- **Streaming API** for large files vs. DOM-style in-memory representation?
- **Type-safe API** using Rust's type system to prevent invalid 3MF generation?
- Sync vs. async I/O (or both)?

### 4. **Performance & Scale Requirements**

- Expected file sizes (MB? GB?)
- Memory constraints?
- Concurrent read/write scenarios?
- Should we support streaming parsing for memory efficiency?

### 5. **Testing Strategy**

- Do you have access to a 3MF test suite or sample files?
- Should we include property-based testing (e.g., using `proptest`)?
- Fuzzing requirements?
- Benchmark suite needed?

### 6. **Dependencies & Constraints**

- Are there restrictions on external crate dependencies?
- Preferred XML parsing library: `quick-xml`, `xml-rs`, or `roxmltree`?
- ZIP handling: `zip` crate acceptable?
- Should we support `no_std` environments?

### 7. **CLI Tools & Utilities**

Should the project include:

- CLI validator tool?
- Format converter (e.g., STL → 3MF)?
- Inspector/debugging tool?
- WASM bindings for browser usage?

### 8. **Documentation & Examples**

- API documentation (rustdoc)
- User guide / tutorial
- Migration guide (if coming from C++ lib3mf)
- Example projects needed?

### 9. **Project Structure Preferences**

```
lib3mf-rs/
├── lib3mf-core/      # Core library
├── lib3mf-cli/       # CLI tools
├── lib3mf-ffi/       # C FFI bindings
├── lib3mf-examples/  # Example projects
└── lib3mf-spec/      # Specification documents
```

Does this structure work?

### 10. **Compliance & Standards**

- Target 3MF Core Specification version? (1.x, 2.x, 3.0?)
- Required conformance class (producer, consumer, full)?
- Digital signature support needed?
- Encryption support needed?

------

## Preliminary Recommendations

Based on best practices, I recommend:

### **Technology Stack**

- **XML Parsing**: `quick-xml` (fast, streaming, low memory)
- **ZIP Handling**: `zip` crate with streaming support
- **Validation**: Custom validator + `xmlschema` if available
- **Serialization**: `serde` for internal structures
- **Error Handling**: `thiserror` for library errors, `anyhow` for CLI
- **Async**: `tokio` (optional feature flag)
- **Testing**: `proptest`, `criterion` benchmarks
- **Fuzzing**: `cargo-fuzz` with `libfuzzer`

### **Architecture Pattern**

- **Layered Architecture**:
  1. **Core Layer**: Spec-compliant data structures
  2. **Parser Layer**: XML → Rust structs
  3. **Writer Layer**: Rust structs → XML
  4. **Validation Layer**: Spec compliance checking
  5. **API Layer**: High-level ergonomic interface

### **Development Approach**

- TDD (Test-Driven Development) where feasible
- Spec-Driven Development with markdown specs
- Incremental validation at each phase
- Continuous benchmarking

------

## Proposed Phase Breakdown

### **Phase 0: Foundation**

- Project scaffolding
- CI/CD setup
- Specification analysis
- Design document creation

### **Phase 1: Core 3MF Read

- Basic 3D model reading
- Mesh data structures
- Resource management
- Unit tests

### **Phase 2: Core 3MF Write

- Model serialization
- ZIP archive creation
- Round-trip testing

### **Phase 3: Validation & Compliance

- Schema validation
- Semantic validation
- Conformance testing

### **Phase 4: Extensions

- Materials & Properties
- Production extension
- Other extensions as prioritized

### **Phase 5: Optimization & Polish

- Performance tuning
- Memory optimization
- API refinement
- Documentation

### **Phase 6: Tooling & Ecosystem

- CLI tools
- FFI bindings
- Examples & tutorials

------

## Questions for You

**Please answer these to help me create the detailed specification documents:**

1. **Pure Rust or FFI bindings?**
2. **Which 3MF spec version and extensions are priority?**
3. **Do you have test files/test suite access?**
4. **Memory/performance constraints?**
5. **Need WASM/FFI/CLI tools?**
6. **Preferred project timeline?**
7. **Team size and skill levels?**
8. **Any existing Rust 3MF code to reference or avoid?**

Once you answer these, I'll create comprehensive markdown specification documents for each phase, complete with:

- Architecture diagrams (in markdown/mermaid)
- Detailed task breakdowns
- Acceptance criteria
- Testing strategies
- Code structure specifications
- API design specifications

Should I also create **Claude or Gemini Skills** for domain-specific knowledge like:

- 3MF specification skill
- Mesh processing skill
- XML/ZIP handling patterns skill

Let me know your preferences and answers, and I'll generate the complete implementation plan!

--

Answer 1. We will implement the library in Rust from scratch to avoid challenges and baggage using the C++ library. No C or C++ bindings are required. You can use the C++ library as a guide or reference only to understand how to implement features and algorithms if required.

Answer 2. We must have a full and complete implementation covering all addendum’s and extensions to the specification.

Answer 3. My initial requirement is to allow other rust applications to be build using lib3mf-rs, but these applications may implement REST API servers, bindings for python, typescript, and more. We need to support reading, writing, and validating 3rd files and any included files within the 3mf bundle, such as screenshots, metadata, etc. streaming options should be available but optional if the user requires it. Sync and asynchronous should be implemented, allowing the user or application to decide.

Answer 4. Most 3mf files are in the MiB range. I expect the library to be used on consumer laptops or desktop PCs with 4GiB of memory and 2 VCPUs or more. The library should be as lightweight as possible and aim to use the smallest memory as possible with minimal CPU as possible without sacrificing performance. Streaming may be required if the system resources are low.

Answer 5. I will provide as many 3rd files as you need from makerworld.com. Include property testing, fuzzing testing, and performance benchmarking so we can find regressions. We will make use of GitHub actions that run when a pull request is created, or when a release is generated.

Answer 6. We will search and decide what the best or optimal crate dependencies are during development. If there isn’t a suitable crate, identify the gaps that we will decide whether to find a workaround or implement a crate solution ourselves. Zip file handling must be included. Zip files must be automatically identified, even if they do not include a zip file extension, such as 3mf which is a zip archive.

Answer 7. Yes, include an extensive cli for the library to give users a way to use the library on existing files. The lib3mf and cli should include format conversion from 3mf to stl or stl to 3mf. The library and cli should include inspection, debugging, and fixing common problems. Wasm bindings is required.

Answer 8. Yes, include extensive documentation and examples for the library and cli.

Answer 9. The project structure looks good. I don’t think we need C bindings as that is already covered by the C++ library provided by the consortium.

Answer 10. This version of lib3mf should support the most recent version of the specification. Support full conformance class. Include digital signature and encryption support if the user needs it. It will be up to the user or application to enable and use these features, but lib3mf-rs should support it.

I approve the rest of your recommendations. Create Claude or Gemini skills as required.

--



# Lib3mf-rs: Complete Architecture & Implementation Plan

## Executive Summary

This document outlines the complete architecture and phased implementation plan for **lib3mf-rs**, a pure Rust implementation of the 3D Manufacturing Format (3MF) specification with full support for all extensions, addendums, digital signatures, and encryption.

**Key Design Principles:**

- Pure Rust implementation (no FFI to C++ library)
- Full specification compliance with all extensions
- Both sync and async APIs
- Memory-efficient with optional streaming
- Extensive CLI tooling
- WASM support
- Enterprise-grade quality with comprehensive testing

------

## Project Structure

```
lib3mf-rs/
├── Cargo.toml                          # Workspace root
├── README.md
├── LICENSE
├──.claude/skills                       # Claude skills
│   ├── 3mf-spec-expert/
│   ├── mesh-processing/
│   ├── xml-zip-patterns/
│   └── crypto-security/
├──.gemini/skills/                      # Gemini skills
│   ├── 3mf-spec-expert/
│   ├── mesh-processing/
│   ├── xml-zip-patterns/
│   └── crypto-security/
├── .github/
│   └── workflows/
│       ├── ci.yml                      # PR validation
│       ├── release.yml                 # Release automation
│       ├── benchmark.yml               # Performance tracking
│       └── security-audit.yml          # Dependency auditing
│
├── AGENTS.md                           # https://agents.md/
│
├── crates/
│   ├── lib3mf-core/                   # Core library
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── model/                 # 3MF data structures
│   │   │   ├── parser/                # Reading 3MF files
│   │   │   ├── writer/                # Writing 3MF files
│   │   │   ├── validator/             # Spec compliance validation
│   │   │   ├── extensions/            # All 3MF extensions
│   │   │   ├── crypto/                # Signatures & encryption
│   │   │   ├── archive/               # ZIP handling
│   │   │   ├── streaming/             # Streaming APIs
│   │   │   ├── error.rs               # Error types
│   │   │   └── utils/                 # Utilities
│   │   ├── tests/                     # Integration tests
│   │   └── benches/                   # Benchmarks
│   │
│   ├── lib3mf-async/                  # Async runtime support
│   │   ├── Cargo.toml
│   │   └── src/
│   │
│   ├── lib3mf-cli/                    # CLI tools
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── commands/              # CLI commands
│   │       │   ├── validate.rs
│   │       │   ├── convert.rs
│   │       │   ├── inspect.rs
│   │       │   ├── repair.rs
│   │       │   └── benchmark.rs
│   │       └── utils/
│   │
│   ├── lib3mf-wasm/                   # WASM bindings
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   └── www/                       # JS wrapper & examples
│   │
│   └── lib3mf-converters/             # Format converters
│       ├── Cargo.toml
│       └── src/
│           ├── stl.rs
│           ├── obj.rs
│           └── common.rs
│
├── docs/
│   ├── architecture/                  # Architecture docs
│   ├── specifications/                # Spec-driven dev docs
│   ├── user-guide/                    # User documentation
│   ├── api-reference/                 # API docs
│   └── examples/                      # Example code
│
├── specs/                             # 3MF Specifications
│   ├── 3MF-Core-Spec.pdf
│   ├── materials-and-properties.pdf
│   ├── beam-lattice.pdf
│   ├── slice.pdf
│   ├── production.pdf
│   ├── volumetric.pdf
│   └── ... (all other extensions)
│
├── test-data/                         # Test 3MF files
│   ├── valid/
│   ├── invalid/
│   ├── edge-cases/
│   └── real-world/                    # From makerworld.com
│
└── examples/
    ├── basic-usage/
    ├── async-usage/
    ├── streaming/
    ├── validation/
    └── conversion/
```

------

## Technology Stack

### Core Dependencies

```toml
# XML Processing
quick-xml = "0.31"              # Fast, streaming XML parser
xmlschema = "0.3"               # XML Schema validation (if available)

# ZIP Archive Handling
zip = { version = "0.6", features = ["deflate", "time"] }

# Serialization & Deserialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"              # For metadata
serde_xml_rs = "0.6"            # XML serialization helper

# Error Handling
thiserror = "1.0"               # Library errors
anyhow = "1.0"                  # CLI errors

# Async Runtime (optional feature)
tokio = { version = "1.35", features = ["full"], optional = true }
async-trait = { version = "0.1", optional = true }

# Cryptography (for signatures & encryption)
ring = "0.17"                   # Crypto primitives
rsa = "0.9"                     # RSA support
x509-parser = "0.15"            # Certificate parsing
aes-gcm = "0.10"                # Encryption
sha2 = "0.10"                   # Hashing

# Geometry & Math
nalgebra = "0.32"               # Linear algebra
glam = "0.25"                   # Alternative SIMD math
parry3d = "0.13"                # Collision detection & geometry

# Utilities
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
url = "2.5"                     # URI handling
mime = "0.3"                    # MIME type handling
regex = "1.10"
base64 = "0.21"

# Validation & Testing
proptest = "1.4"                # Property-based testing
criterion = "0.5"               # Benchmarking
quickcheck = "1.0"              # QuickCheck-style testing

# CLI (lib3mf-cli)
clap = { version = "4.4", features = ["derive", "cargo"] }
colored = "2.1"                 # Terminal colors
indicatif = "0.17"              # Progress bars
console = "0.15"                # Terminal utilities

# WASM (lib3mf-wasm)
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = "0.3"

# Logging & Tracing
tracing = "0.1"                 # Structured logging
tracing-subscriber = "0.3"

# Development
cargo-fuzz = "0.11"             # Fuzzing
tarpaulin = "0.27"              # Code coverage
```

------

## Phase Breakdown

## Phase 0: Foundation & Setup

### Objectives

- Project scaffolding
- CI/CD pipeline
- Specification analysis
- Development environment setup
- Create Claude or Gemini Skills

### Deliverables

#### 0.1: Project Initialization

**Tasks:**

1. Create workspace structure
2. Initialize all crate manifests
3. Set up Git repository with proper `.gitignore`
4. Configure EditorConfig and Rustfmt
5. Set up Clippy lints (strict mode)

**Acceptance Criteria:**

- `cargo build --workspace` succeeds
- `cargo clippy --workspace -- -D warnings` passes
- `cargo fmt --check` passes

#### 0.2: CI/CD Pipeline

**Tasks:**

1. Create GitHub Actions workflows:
   - `ci.yml`: Build, test, lint on PR
   - `release.yml`: Publish on tag
   - `benchmark.yml`: Performance tracking
   - `security-audit.yml`: Dependency scanning
2. Configure code coverage with tarpaulin
3. Set up automatic changelog generation
4. Configure dependabot

**Acceptance Criteria:**

- All workflows execute successfully
- Code coverage reporting works
- Security scanning active

#### 0.3: Specification Analysis

**Tasks:**

1. Download all 3MF specifications and extensions
2. Create specification index document
3. Extract data models from specs
4. Identify all required XML schemas
5. Map extensions to implementation phases

**Deliverable:** `docs/specifications/spec-analysis.md`

#### 0.4: Claude and GeminiSkills Creation

**Tasks:**

1. Create `3mf-spec-expert` skill
2. Create `mesh-processing` skill
3. Create `xml-zip-patterns` skill
4. Create `crypto-security` skill

**Deliverable:** All skills in `skills/` directory

#### 0.5: Architecture Documentation

**Tasks:**

1. Create architecture decision records (ADRs)
2. Document data flow diagrams
3. Create API design specification
4. Write error handling strategy

**Deliverables:**

- `docs/architecture/decisions/`
- `docs/architecture/diagrams.md`
- `docs/architecture/api-design.md`

------

## Phase 1: Core Data Structures

### Objectives

- Implement all core 3MF data structures
- Type-safe Rust representations
- Serde serialization support
- Unit tests for all types

### Deliverables

#### 1.1: Base Model Types

**Specification:** `docs/specifications/phase1/01-base-model-types.md`

**Tasks:**

1. Implement `Model` struct
2. Implement `Metadata` types
3. Implement `Resource` trait and types
4. Implement `Build` and `BuildItem`
5. Implement coordinate systems and transformations

**Key Types:**

```rust
pub struct Model {
    pub unit: Unit,
    pub language: Option<String>,
    pub resources: ResourceCollection,
    pub build: Build,
    pub metadata: Vec<Metadata>,
}

pub enum Unit {
    Micron,
    Millimeter,
    Centimeter,
    Inch,
    Foot,
    Meter,
}

pub struct Build {
    pub items: Vec<BuildItem>,
}

pub struct BuildItem {
    pub object_id: ResourceId,
    pub transform: Option<Transform>,
    pub part_number: Option<String>,
}
```

**Acceptance Criteria:**

- All types compile
- Serde serialize/deserialize works
- Unit tests cover all types
- Documentation complete

#### 1.2: Mesh & Geometry Types

**Specification:** `docs/specifications/phase1/02-mesh-geometry.md`

**Tasks:**

1. Implement `Object` and `ObjectType`
2. Implement `Mesh` structure
3. Implement `Vertices` collection
4. Implement `Triangles` collection
5. Implement vertex/triangle validation
6. Add spatial indexing support

**Key Types:**

```rust
pub struct Object {
    pub id: ResourceId,
    pub name: Option<String>,
    pub object_type: ObjectType,
    pub thumbnail: Option<PathBuf>,
    pub pid: Option<u32>,
    pub pindex: Option<u32>,
}

pub enum ObjectType {
    Model(Mesh),
    Support(Mesh),
    SolidsSupport(Mesh),
    Other(Mesh),
}

pub struct Mesh {
    pub vertices: Vertices,
    pub triangles: Triangles,
}

pub struct Vertices {
    vertices: Vec<Vertex>,
}

pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct Triangles {
    triangles: Vec<Triangle>,
}

pub struct Triangle {
    pub v1: u32,
    pub v2: u32,
    pub v3: u32,
    pub p1: Option<u32>,
    pub p2: Option<u32>,
    pub p3: Option<u32>,
    pub pid: Option<u32>,
}
```

**Acceptance Criteria:**

- Mesh validation works (manifold checks)
- Efficient vertex/triangle access
- Memory layout optimized
- Benchmark shows acceptable performance

#### 1.3: Component & Assembly Types

**Specification:** `docs/specifications/phase1/03-components.md`

**Tasks:**

1. Implement `Component` type
2. Implement component graph validation
3. Implement transformation composition
4. Add cycle detection

**Acceptance Criteria:**

- Component trees validated
- No circular references allowed
- Transform math correct

#### 1.4: Resource Management

**Specification:** `docs/specifications/phase1/04-resources.md`

**Tasks:**

1. Implement `ResourceCollection`
2. Implement resource ID management
3. Implement resource lookup and indexing
4. Add resource dependency tracking

**Key Types:**

```rust
pub struct ResourceCollection {
    objects: HashMap<ResourceId, Object>,
    base_materials: HashMap<ResourceId, BaseMaterial>,
    // ... other resource types
}

pub struct ResourceId(u32);
```

**Acceptance Criteria:**

- Fast resource lookup
- ID uniqueness enforced
- Dependency resolution works

------

## Phase 2: ZIP Archive & File I/O

### Objectives

- ZIP archive reading/writing
- Content type detection
- Relationship parsing
- OPC (Open Packaging Conventions) compliance

### Deliverables

#### 2.1: Archive Abstraction

**Specification:** `docs/specifications/phase2/01-archive-abstraction.md`

**Tasks:**

1. Create `Archive` trait
2. Implement `ZipArchive` reader
3. Implement `ZipArchive` writer
4. Add auto-detection for ZIP files (magic bytes)
5. Handle corrupted archives gracefully

**Key Types:**

```rust
pub trait Archive {
    fn read_entry(&mut self, path: &str) -> Result<Vec<u8>>;
    fn write_entry(&mut self, path: &str, data: &[u8]) -> Result<()>;
    fn list_entries(&self) -> Vec<String>;
    fn entry_exists(&self, path: &str) -> bool;
}

pub struct ZipArchive<R: Read + Seek> {
    archive: zip::ZipArchive<R>,
}
```

**Acceptance Criteria:**

- Reads valid ZIP archives
- Writes valid ZIP archives
- Detects ZIP by magic bytes (`50 4B 03 04`)
- Handles .3mf extension

#### 2.2: OPC Relationships

**Specification:** `docs/specifications/phase2/02-opc-relationships.md`

**Tasks:**

1. Parse `_rels/.rels`
2. Parse part-specific relationships
3. Implement relationship types
4. Validate relationship targets

**Key Types:**

```rust
pub struct Relationship {
    pub id: String,
    pub relationship_type: RelationshipType,
    pub target: String,
}

pub enum RelationshipType {
    StartPart,
    Thumbnail,
    PrintTicket,
    Metadata,
    Custom(String),
}
```

**Acceptance Criteria:**

- All relationship types parsed
- Relationship validation works
- Required relationships enforced

#### 2.3: Content Types

**Specification:** `docs/specifications/phase2/03-content-types.md`

**Tasks:**

1. Parse `[Content_Types].xml`
2. Implement MIME type handling
3. Validate content types

**Acceptance Criteria:**

- Content types correctly identified
- Unknown types handled

------

## Phase 3: XML Parsing

### Objectives

- Parse all 3MF XML formats
- Streaming XML support
- Namespace handling
- Error recovery

### Deliverables

#### 3.1: XML Parser Infrastructure

**Specification:** `docs/specifications/phase3/01-xml-parser.md`

**Tasks:**

1. Create XML event streaming wrapper
2. Implement namespace context
3. Create attribute parsing helpers
4. Add error context tracking

**Key Types:**

```rust
pub struct XmlParser<R: BufRead> {
    reader: quick_xml::Reader<R>,
    namespace_stack: Vec<HashMap<String, String>>,
}

pub struct ParseContext {
    pub current_path: Vec<String>,
    pub line_number: usize,
    pub namespaces: HashMap<String, String>,
}
```

#### 3.2: Model XML Parser

**Specification:** `docs/specifications/phase3/02-model-parser.md`

**Tasks:**

1. Parse `<model>` root element
2. Parse `<resources>` section
3. Parse `<build>` section
4. Parse `<metadata>` elements
5. Handle all core spec attributes

**Acceptance Criteria:**

- Parses valid 3MF model files
- Provides detailed error messages
- Handles malformed XML gracefully

#### 3.3: Mesh Parser

**Specification:** `docs/specifications/phase3/03-mesh-parser.md`

**Tasks:**

1. Parse `<object>` elements
2. Parse `<mesh>` structure
3. Parse `<vertices>` efficiently
4. Parse `<triangles>` efficiently
5. Validate during parsing

**Performance Target:**

- Parse 100K triangles in < 100ms

#### 3.4: Component Parser

**Specification:** `docs/specifications/phase3/04-component-parser.md`

**Tasks:**

1. Parse `<components>` elements
2. Parse transformation matrices
3. Build component graph

------

## Phase 4: XML Writing

### Objectives

- Generate valid 3MF XML
- Pretty-printing support
- Namespace management
- Efficient writing

### Deliverables

#### 4.1: XML Writer Infrastructure

**Specification:** `docs/specifications/phase4/01-xml-writer.md`

**Tasks:**

1. Create XML event writer wrapper
2. Implement namespace tracking
3. Add indentation support
4. Optimize for large meshes

#### 4.2: Model XML Writer

**Specification:** `docs/specifications/phase4/02-model-writer.md`

**Tasks:**

1. Write `<model>` element
2. Write `<resources>`
3. Write `<build>`
4. Write `<metadata>`

#### 4.3: Mesh Writer

**Specification:** `docs/specifications/phase4/03-mesh-writer.md`

**Tasks:**

1. Write `<object>` elements
2. Write `<mesh>` efficiently
3. Write vertices in batches
4. Write triangles in batches

**Performance Target:**

- Write 100K triangles in < 100ms

------

## Phase 5: Validation

### Objectives

- Full spec compliance validation
- Semantic validation
- Geometry validation
- Performance validation

### Deliverables

#### 5.1: Schema Validation

**Specification:** `docs/specifications/phase5/01-schema-validation.md`

**Tasks:**

1. XML Schema validation
2. Namespace validation
3. Required element checking
4. Attribute validation

#### 5.2: Semantic Validation

**Specification:** `docs/specifications/phase5/02-semantic-validation.md`

**Tasks:**

1. Resource ID validation
2. Reference validation
3. Unit consistency
4. Transform validity
5. Component cycle detection

#### 5.3: Geometry Validation

**Specification:** `docs/specifications/phase5/03-geometry-validation.md`

**Tasks:**

1. Manifold mesh checking
2. Non-degenerate triangle validation
3. Orientation consistency
4. Vertex uniqueness
5. Watertight mesh validation

**Validation Levels:**

```rust
pub enum ValidationLevel {
    Minimal,      // Basic structure only
    Standard,     // Spec compliance
    Strict,       // Full semantic validation
    Paranoid,     // Everything including geometry
}
```

#### 5.4: Validation Reporter

**Specification:** `docs/specifications/phase5/04-validation-reporter.md`

**Tasks:**

1. Create validation result types
2. Implement warning/error categorization
3. Add fix suggestions
4. Create JSON/human-readable output

------

## Phase 6: Extensions - Materials & Properties

### Objectives

- Full Materials & Properties extension support
- Base materials
- Color groups
- Texture coordinates
- Property groups

### Deliverables

#### 6.1: Base Materials

**Specification:** `docs/specifications/phase6/01-base-materials.md`

**Tasks:**

1. Implement `BaseMaterial` type
2. Parse `<basematerials>` groups
3. Write base materials
4. Validate material references

**Key Types:**

```rust
pub struct BaseMaterial {
    pub name: String,
    pub display_color: Color,
}

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
```

#### 6.2: Color Groups

**Specification:** `docs/specifications/phase6/02-color-groups.md`

**Tasks:**

1. Implement `ColorGroup`
2. Support per-vertex colors
3. Support per-triangle colors
4. Color interpolation

#### 6.3: Texture Coordinates

**Specification:** `docs/specifications/phase6/03-texture-coords.md`

**Tasks:**

1. Implement `Texture2DGroup`
2. Parse UV coordinates
3. Handle texture references
4. Support embedded textures

#### 6.4: Composite Materials

**Specification:** `docs/specifications/phase6/04-composite-materials.md`

**Tasks:**

1. Implement `CompositeMaterials`
2. Parse material mixing
3. Validate composite references

------

## Phase 7: Extensions - Production

### Objectives

- Production extension support
- Build item references
- Production paths
- UUID tracking

### Deliverables

#### 7.1: Production Items

**Specification:** `docs/specifications/phase7/01-production-items.md`

**Tasks:**

1. Implement production item tracking
2. Parse UUID references
3. Handle production paths

------

## Phase 8: Extensions - Beam Lattice

### Objectives

- Beam lattice support
- Beam definitions
- Ball references
- Lattice validation

### Deliverables

#### 8.1: Beam Structures

**Specification:** `docs/specifications/phase8/01-beam-structures.md`

**Tasks:**

1. Implement `BeamLattice` type
2. Parse beam definitions
3. Validate beam connectivity

------

## Phase 9: Extensions - Slice

### Objectives

- Slice extension support
- Slice stack parsing
- Polygon validation

### Deliverables

#### 9.1: Slice Stacks

**Specification:** `docs/specifications/phase9/01-slice-stacks.md`

**Tasks:**

1. Implement slice stack structures
2. Parse slice polygons
3. Validate slice data

------

## Phase 10: Extensions - Volumetric

### Objectives

- Volumetric data support
- Voxel grids
- Implicit functions

### Deliverables

#### 10.1: Volumetric Data

**Specification:** `docs/specifications/phase10/01-volumetric.md`

**Tasks:**

1. Implement volumetric structures
2. Parse voxel data efficiently
3. Support compression

------

## Phase 11: Extensions - Remaining Extensions

### Objectives

- Implement all remaining official extensions
- Third-party extension framework

### Extensions to Implement:

1. **Secure Content** (encryption/signatures)
2. **Custom Metadata**
3. **Any other official 3MF extensions**

------

## Phase 12: Cryptography - Signatures & Encryption (Week 19)

### Objectives

- Digital signature support
- Content encryption
- Certificate management

### Deliverables

#### 12.1: Digital Signatures

**Specification:** `docs/specifications/phase12/01-signatures.md`

**Tasks:**

1. Implement XML-DSIG signing
2. Parse signature elements
3. Verify signatures
4. Certificate chain validation

**Key Types:**

```rust
pub struct Signature {
    pub algorithm: SignatureAlgorithm,
    pub certificate: Certificate,
    pub signature_value: Vec<u8>,
}

pub enum SignatureAlgorithm {
    RsaSha256,
    RsaSha512,
    EcdsaSha256,
}
```

#### 12.2: Encryption

**Specification:** `docs/specifications/phase12/02-encryption.md`

**Tasks:**

1. Implement AES-GCM encryption
2. Parse encrypted elements
3. Key management
4. Decrypt on read, encrypt on write

------

## Phase 13: Async Support

### Objectives

- Async I/O for all operations
- Tokio runtime integration
- Async streaming

### Deliverables

#### 13.1: Async Archive I/O

**Specification:** `docs/specifications/phase13/01-async-io.md`

**Tasks:**

1. Create async `Archive` trait
2. Implement async ZIP reading
3. Implement async ZIP writing
4. Add async streaming parser

**Key Types:**

```rust
#[async_trait]
pub trait AsyncArchive {
    async fn read_entry(&mut self, path: &str) -> Result<Vec<u8>>;
    async fn write_entry(&mut self, path: &str, data: &[u8]) -> Result<()>;
}
```

#### 13.2: Async Parser

**Specification:** `docs/specifications/phase13/02-async-parser.md`

**Tasks:**

1. Async XML parsing
2. Async resource loading
3. Progress reporting

------

## Phase 14: Streaming API

### Objectives

- Memory-efficient streaming
- Large file support
- Incremental parsing

### Deliverables

#### 14.1: Streaming Parser

**Specification:** `docs/specifications/phase14/01-streaming-parser.md`

**Tasks:**

1. Event-based parsing API
2. Chunk-based mesh loading
3. Memory bounds enforcement

**API Design:**

```rust
pub trait ModelVisitor {
    fn visit_metadata(&mut self, metadata: &Metadata) -> Result<()>;
    fn visit_object_start(&mut self, id: ResourceId) -> Result<()>;
    fn visit_vertices(&mut self, vertices: &[Vertex]) -> Result<()>;
    fn visit_triangles(&mut self, triangles: &[Triangle]) -> Result<()>;
    fn visit_object_end(&mut self) -> Result<()>;
}

pub fn parse_streaming<R, V>(reader: R, visitor: V) -> Result<()>
where
    R: Read,
    V: ModelVisitor;
```

------

## Phase 15: Format Converters

### Objectives

- STL ↔ 3MF conversion
- OBJ support
- PLY support (optional)

### Deliverables

#### 15.1: STL Converter

**Specification:** `docs/specifications/phase15/01-stl-converter.md`

**Tasks:**

1. Parse ASCII STL
2. Parse binary STL
3. Write STL from 3MF
4. Handle units conversion
5. Preserve metadata where possible

#### 15.2: OBJ Converter

**Specification:** `docs/specifications/phase15/02-obj-converter.md`

**Tasks:**

1. Parse OBJ files
2. Convert to 3MF mesh
3. Handle materials (MTL files)

------

## Phase 16: CLI Tools

### Objectives

- Full-featured CLI
- User-friendly interface
- Comprehensive documentation

### Deliverables

#### 16.1: CLI Framework

**Specification:** `docs/specifications/phase16/01-cli-framework.md`

**Commands:**

```bash
lib3mf validate <file>              # Validate 3MF file
lib3mf inspect <file>               # Inspect contents
lib3mf convert <input> <output>     # Convert formats
lib3mf repair <file>                # Fix common issues
lib3mf extract <file> <dir>         # Extract archive
lib3mf create <dir> <output>        # Create 3MF from parts
lib3mf sign <file>                  # Add digital signature
lib3mf encrypt <file>               # Encrypt 3MF
lib3mf benchmark <file>             # Performance test
lib3mf diff <file1> <file2>         # Compare files
```

#### 16.2: Validate Command

**Specification:** `docs/specifications/phase16/02-validate-cmd.md`

**Features:**

- Multiple validation levels
- JSON/YAML/human output
- Color-coded results
- Fix suggestions

#### 16.3: Inspect Command

**Specification:** `docs/specifications/phase16/03-inspect-cmd.md`

**Features:**

- Show file structure
- List resources
- Display metadata
- Show statistics (triangle count, etc.)
- Preview thumbnails

#### 16.4: Repair Command

**Specification:** `docs/specifications/phase16/04-repair-cmd.md`

**Fixes:**

- Fix manifold issues
- Remove duplicate vertices
- Fix triangle orientation
- Repair resource references
- Update content types

------

## Phase 17: WASM Bindings

### Objectives

- Browser-compatible WASM module
- JavaScript/TypeScript bindings
- NPM package

### Deliverables

#### 17.1: WASM Core

**Specification:** `docs/specifications/phase17/01-wasm-core.md`

**Tasks:**

1. Implement wasm-bindgen bindings
2. Create JS-friendly API
3. Handle async operations
4. Optimize binary size

**API Example:**

```javascript
import init, { Model } from 'lib3mf-wasm';

await init();

// Parse 3MF
const model = await Model.from_bytes(fileBytes);

// Inspect
console.log(`Objects: ${model.object_count()}`);
console.log(`Triangles: ${model.triangle_count()}`);

// Convert to JSON
const json = model.to_json();

// Validate
const result = model.validate();
```

#### 17.2: TypeScript Definitions

**Specification:** `docs/specifications/phase17/02-typescript.md`

**Tasks:**

1. Generate `.d.ts` files
2. Document all APIs
3. Add JSDoc comments

#### 17.3: NPM Package

**Specification:** `docs/specifications/phase17/03-npm-package.md`

**Tasks:**

1. Create package.json
2. Bundle WASM + JS
3. Publish to NPM

------

## Phase 18: Testing & Quality

### Objectives

- Comprehensive test coverage
- Property-based testing
- Fuzzing
- Performance benchmarks

### Deliverables

#### 18.1: Unit Tests

**Specification:** `docs/specifications/phase18/01-unit-tests.md`

**Coverage Target:** 90%+

**Test Categories:**

1. Data structure tests
2. Parser tests
3. Writer tests
4. Validator tests
5. Extension tests
6. Crypto tests

#### 18.2: Integration Tests

**Specification:** `docs/specifications/phase18/02-integration-tests.md`

**Test Cases:**

1. Round-trip tests (read → write → read)
2. Real-world file tests (makerworld.com)
3. Malformed file handling
4. Large file tests (100MB+)
5. Concurrent access tests

#### 18.3: Property-Based Tests

**Specification:** `docs/specifications/phase18/03-property-tests.md`

**Properties to Test:**

1. Round-trip identity: `parse(write(model)) == model`
2. Validation idempotence: `validate(validate(x)) == validate(x)`
3. Transform composition associativity
4. Mesh validity preservation

**Example:**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_preserves_model(model: Model) {
        let bytes = write_to_bytes(&model)?;
        let parsed = parse_from_bytes(&bytes)?;
        prop_assert_eq!(model, parsed);
    }
}
```

#### 18.4: Fuzz Testing

**Specification:** `docs/specifications/phase18/04-fuzzing.md`

**Fuzz Targets:**

1. ZIP archive parsing
2. XML parsing
3. Mesh parsing
4. Crypto operations

**Setup:**

```bash
cargo fuzz run parse_3mf
cargo fuzz run parse_xml
cargo fuzz run parse_mesh
```

#### 18.5: Benchmarks

**Specification:** `docs/specifications/phase18/05-benchmarks.md`

**Benchmark Suite:**

1. Parse performance (various file sizes)
2. Write performance
3. Validation performance
4. Memory usage
5. Streaming vs. non-streaming

**Benchmark Targets:**

- Parse 1MB file: < 50ms
- Parse 10MB file: < 500ms
- Parse 100MB file: < 5s
- Memory: < 2x file size

------

## Phase 19: Documentation

### Objectives

- Complete API documentation
- User guide
- Developer guide
- Examples

### Deliverables

#### 19.1: API Documentation

**Specification:** `docs/specifications/phase19/01-api-docs.md`

**Tasks:**

1. Write rustdoc for all public APIs
2. Add code examples to docs
3. Document all error types
4. Create module-level documentation

**Quality Bar:**

- All public items documented
- Examples for common use cases
- Links between related items

#### 19.2: User Guide

**Specification:** `docs/specifications/phase19/02-user-guide.md`

**Chapters:**

1. Getting Started
2. Reading 3MF Files
3. Writing 3MF Files
4. Validation
5. Format Conversion
6. Working with Extensions
7. Async API
8. Streaming Large Files
9. Digital Signatures
10. Encryption
11. CLI Usage
12. WASM Usage

#### 19.3: Developer Guide

**Specification:** `docs/specifications/phase19/03-developer-guide.md`

**Topics:**

1. Architecture Overview
2. Adding New Extensions
3. Custom Validators
4. Performance Optimization
5. Contributing Guidelines

#### 19.4: Examples

**Specification:** `docs/specifications/phase19/04-examples.md`

**Examples to Create:**

1. `basic-read-write` - Simple 3MF I/O
2. `mesh-creation` - Create mesh programmatically
3. `validation` - Validate and fix files
4. `async-usage` - Async file operations
5. `streaming` - Process large files
6. `materials` - Work with materials
7. `signatures` - Sign and verify
8. `cli-integration` - Use as library in CLI apps
9. `wasm-browser` - Browser usage
10. `stl-converter` - Format conversion

------

## Phase 20: Performance Optimization

### Objectives

- Profile and optimize hot paths
- Memory usage optimization
- SIMD where applicable

### Deliverables

#### 20.1: Profiling

**Specification:** `docs/specifications/phase20/01-profiling.md`

**Tasks:**

1. CPU profiling with `perf`/`Instruments`
2. Memory profiling with `valgrind`/`heaptrack`
3. Identify bottlenecks
4. Create optimization plan

#### 20.2: Parser Optimization

**Specification:** `docs/specifications/phase20/02-parser-optimization.md`

**Optimizations:**

1. Zero-copy parsing where possible
2. Optimize hot loops
3. Reduce allocations
4. Parallel parsing (optional)

#### 20.3: Memory Optimization

**Specification:** `docs/specifications/phase20/03-memory-optimization.md`

**Optimizations:**

1. Compact data structures
2. Lazy loading
3. Resource pooling
4. Memory mapping for large files

#### 20.4: SIMD Optimization

**Specification:** `docs/specifications/phase20/04-simd.md`

**Targets:**

1. Vertex transformations
2. Mesh validation
3. Compression/decompression

------

## Phase 21: Release Preparation

### Objectives

- Final QA
- Documentation review
- Release automation
- Marketing materials

### Deliverables

#### 21.1: Release Checklist

**Specification:** `docs/specifications/phase21/01-release-checklist.md`

**Items:**

- [ ] All tests passing
- [ ] 90%+ code coverage
- [ ] No clippy warnings
- [ ] Documentation complete
- [ ] Examples working
- [ ] Benchmarks run
- [ ] Security audit passed
- [ ] CHANGELOG updated
- [ ] Version numbers set
- [ ] Tags created

#### 21.2: Crates.io Publication

**Tasks:**

1. Verify crate metadata
2. Test local publication
3. Publish to crates.io
4. Verify published crates

#### 21.3: NPM Publication (WASM)

**Tasks:**

1. Build WASM package
2. Test in browsers
3. Publish to NPM

#### 21.4: Announcement

**Tasks:**

1. Write release blog post
2. Create announcement tweets
3. Post to Reddit/HN
4. Notify 3MF consortium

------

## Testing Strategy

### Test Pyramid

```
         ╱ ╲
        ╱ E2E╲             5%  - End-to-end (CLI, real files)
       ╱───────╲
      ╱  Integ  ╲          15% - Integration tests
     ╱───────────╲
    ╱    Unit     ╲        80% - Unit tests
   ╱───────────────╲
```

### Test Categories

#### 1. Unit Tests

- Every public function
- Edge cases
- Error conditions
- Data structure invariants

#### 2. Integration Tests

- Read/write round-trips
- Validation workflows
- Format conversions
- Extension interactions

#### 3. Property-Based Tests

- Invariant checking
- Fuzzing inputs
- Randomized testing

#### 4. Performance Tests

- Benchmarks
- Regression detection
- Memory profiling

#### 5. Real-World Tests

- Files from makerworld.com
- Various 3MF producers
- Edge cases from community

### CI/CD Testing

**On Every PR:**

```yaml
- cargo test --all-features
- cargo clippy -- -D warnings
- cargo fmt --check
- cargo tarpaulin --out Lcov
- cargo audit
- cargo deny check
```

**On Main Branch:**

```yaml
- All PR checks
- cargo bench (save baseline)
- Integration test full suite
- Build documentation
- WASM build test
```

**On Release Tag:**

```yaml
- Full test suite
- Cross-platform builds
- Publish to crates.io
- Publish NPM package
- Deploy documentation
```

------

## Error Handling Strategy

### Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Lib3mfError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("XML parsing error at line {line}: {message}")]
    XmlParse { line: usize, message: String },
    
    #[error("Invalid 3MF structure: {0}")]
    InvalidStructure(String),
    
    #[error("Validation failed: {0}")]
    Validation(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(ResourceId),
    
    #[error("Invalid mesh: {0}")]
    InvalidMesh(String),
    
    #[error("Unsupported extension: {0}")]
    UnsupportedExtension(String),
    
    #[error("Cryptography error: {0}")]
    Crypto(String),
    
    #[error("Archive error: {0}")]
    Archive(String),
}

pub type Result<T> = std::result::Result<T, Lib3mfError>;
```

### Error Context

- Include file location in errors
- Provide helpful error messages
- Suggest fixes where possible
- Never panic in library code

------

## Security Considerations

### 1. Input Validation

- Validate all external inputs
- Bounds checking on indices
- Size limits on allocations
- ZIP bomb prevention

### 2. Cryptography

- Use well-audited crypto libraries
- Constant-time operations
- Secure key management
- Certificate validation

### 3. Dependencies

- Regular `cargo audit`
- Minimal dependency tree
- Review all dependencies
- Pin versions

### 4. Fuzzing

- Continuous fuzzing
- OSS-Fuzz integration (optional)
- Corpus management

------

## Performance Targets

### Parsing Benchmarks

| File Size | Target Time | Memory Usage |
| --------- | ----------- | ------------ |
| 1 MB      | < 50ms      | < 5 MB       |
| 10 MB     | < 500ms     | < 30 MB      |
| 100 MB    | < 5s        | < 200 MB     |

### Writing Benchmarks

| Triangle Count | Target Time |
| -------------- | ----------- |
| 10K            | < 10ms      |
| 100K           | < 100ms     |
| 1M             | < 1s        |

### Validation Benchmarks

| Operation | Target Time |
| --------- | ----------- |
| Schema    | < 50ms      |
| Semantic  | < 100ms     |
| Geometry  | < 500ms     |

------

## Claude and Gemini Skills

I will create the following Claude and Gemini Skills:

### 1. `3mf-spec-expert`

**Purpose:** Domain knowledge about 3MF specification

**Content:**

- All 3MF specifications (embedded)
- Extension details
- Common patterns
- Best practices

### 2. `mesh-processing`

**Purpose:** 3D geometry and mesh algorithms

**Content:**

- Manifold checking algorithms
- Normal computation
- Mesh repair techniques
- Spatial indexing
- Optimization strategies

### 3. `xml-zip-patterns`

**Purpose:** XML/ZIP handling patterns

**Content:**

- Efficient XML parsing
- Streaming techniques
- ZIP optimization
- OPC conventions
- Namespace handling

### 4. `crypto-security`

**Purpose:** Cryptography and security

**Content:**

- Digital signature patterns
- Encryption best practices
- Certificate handling
- Security audit checklist
- Common vulnerabilities

------

## Next Steps

Would you like me to:

1. **Create the first skill** (`3mf-spec-expert`) with the specification downloaded and embedded?
2. **Generate Phase 0 detailed specifications** - the foundation phase with all markdown documents for Claude or Gemini Code?
3. **Set up the initial project structure** with all the Cargo.toml files, CI/CD workflows, and basic scaffolding?
4. **Create a detailed roadmap document** with Gantt chart and milestone tracking?

Please let me know which you'd like to prioritize, or if you'd like me to proceed with all of them in sequence!