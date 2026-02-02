# Architecture

**Analysis Date:** 2026-02-02

## Pattern Overview

**Overall:** Layered Archive-Parser-Model-Validator-Writer pipeline with immutable model semantics and trait-based abstraction for extensibility.

**Key Characteristics:**
- Linear pipeline architecture: ZIP/OPC container → XML parser → in-memory model → validation → serialization
- Immutable-by-default model design with explicit repair operations
- Trait-based abstractions decoupling parser from storage backend
- Extension-first design: materials, beam lattice, slices, volumetric, and secure content are first-class citizens
- Progressive validation system with four levels of strictness
- Two parsing modes: DOM (fast, in-memory) and SAX (streaming, constant memory)

## Layers

**Archive Layer:**
- Purpose: Opens and reads OPC (Open Packaging Convention) ZIP containers implementing the 3MF specification
- Location: `crates/lib3mf-core/src/archive/`
- Contains: `ZipArchiver` wrapper, OPC relationship/content type parsing, model path locator
- Depends on: `zip` crate for ZIP handling, `quick-xml` for OPC relationship parsing
- Used by: Parser layer to locate and read model XML and attachments

**Parser Layer:**
- Purpose: Converts XML event stream into in-memory `Model` structure; handles all extension-specific parsing
- Location: `crates/lib3mf-core/src/parser/`
- Contains: Modular parsers for each element type (mesh, materials, build, beam lattice, slice, volumetric, secure content)
- Depends on: Archive layer for raw XML, `quick-xml` for SAX events, `lexical-core` for float parsing
- Used by: Model creation workflow

**Model Layer:**
- Purpose: Central immutable data structures representing 3MF content
- Location: `crates/lib3mf-core/src/model/`
- Contains: Root `Model` struct, `ResourceCollection` registry, geometry types (`Mesh`, `Components`, `SliceStack`, `VolumetricStack`), material types, build instructions, metadata
- Depends on: None (pure data)
- Used by: All layers; consumers directly inspect/modify models

**Validation Layer:**
- Purpose: Progressive structural, semantic, and geometric checks at four strictness levels
- Location: `crates/lib3mf-core/src/validation/`
- Contains: Schema validation (attributes), semantic validation (reference integrity), geometry validation (manifoldness, self-intersection), BVH tree acceleration
- Depends on: Model layer
- Used by: CLI commands, examples, user code calling `model.validate(level)`

**Writer Layer:**
- Purpose: Serializes model back to XML and ZIP archive; mirrors parser structure
- Location: `crates/lib3mf-core/src/writer/`
- Contains: Model-to-XML serialization, OPC relationship generation, ZIP packaging, extension writing
- Depends on: Model layer, `quick-xml` for XML generation, `zip` crate for archive creation
- Used by: CLI copy/save commands, examples, user code calling `model.write(file)`

**Crypto/Security Layer:**
- Purpose: Digital signature verification and content encryption support for Secure Content Extension
- Location: `crates/lib3mf-core/src/crypto/`
- Contains: AES-GCM encryption/decryption, X.509 certificate parsing, XML-DSIG signature structures
- Depends on: `ring` and `x509-parser` crates
- Used by: Secure content parser and writer

**Utility Layer:**
- Purpose: Cross-cutting concerns: diff, canonicalization, hardware info, statistics
- Location: `crates/lib3mf-core/src/utils/`
- Contains: Model diffing, C14N canonicalization, hardware detection, statistics computation with optional parallelization
- Depends on: Model layer; optional `rayon` for parallel stats
- Used by: CLI commands, validation

## Data Flow

**Reading (Typical):**

1. `ZipArchiver::new(file)` opens 3MF ZIP container
2. `find_model_path()` locates `_rels/.rels` and resolves model XML path (e.g., `3D/3dmodel.model`)
3. `archiver.read_entry(model_path)` reads XML bytes
4. `parse_model(xml_reader)` deserializes XML → `Model` struct:
   - `XmlParser` wraps `quick-xml` event stream
   - `parse_resources()` dispatches to type-specific parsers (mesh, materials, etc.)
   - Extensions (beam lattice, slice) integrated directly into geometry/material parsing
5. `model.validate(level)` checks structure/semantics/geometry as needed
6. Consumer accesses `model.resources`, `model.build`, `model.attachments`

**Writing (Typical):**

1. User constructs or modifies `Model` with objects, materials, and build items
2. `model.write(file)` serializes:
   - `write_xml()` generates model XML via `XmlWriter`
   - Extension data written inline in object/material elements
   - `package_writer.rs` orchestrates ZIP creation and OPC relationships
   - Attachments (textures, thumbnails) written as additional archive entries
3. Output: Valid 3MF ZIP file

**State Management:**
- Models are immutable after parsing; mutation happens via explicit `repair()` operations on meshes
- `ResourceCollection` maintains global ID registry preventing duplicates
- `Attachments` stored as `HashMap<path, bytes>` in model for binary content
- `ExistingRelationships` preserved during round-trip for OPC compatibility

## Key Abstractions

**ArchiveReader Trait:**
- Purpose: Abstracts archive backend (ZIP or alternatives) from parser
- Location: `crates/lib3mf-core/src/archive/mod.rs`
- Pattern: Trait with `read_entry()`, `entry_exists()`, `list_entries()`
- Examples: `ZipArchiver<R: Read + Seek>` for ZIP files
- Used by: Parser to decouple from specific archive format

**Geometry Enum:**
- Purpose: Represents different geometric representations of an object
- Pattern: Sum type with variants: `Mesh`, `Components` (hierarchical assembly), `SliceStack` (layer data), `VolumetricStack` (voxel data)
- Allows a single object to use any geometric approach
- Parser/writer handle each variant differently

**ResourceId Newtype:**
- Purpose: Type-safe resource identifiers preventing ID mixing
- Pattern: `ResourceId(u32)` prevents accidental confusion with other u32 values
- Used in: `ResourceCollection`, object references, property references
- Enforces: Global namespace within model (all resource types share ID space)

**ResourceCollection Registry:**
- Purpose: Central resource storage with duplicate ID detection
- Pattern: Separate `HashMap<ResourceId, T>` for each type; centralized `add_*()` and `get_*()` methods
- Methods check `exists()` across all types before insertion
- Used by: Parser to add parsed resources, validation to resolve references

**MeshRepair Trait:**
- Purpose: Explicit mutation operations on geometry (stitching, hole-filling, orientation fixing)
- Location: `crates/lib3mf-core/src/model/repair.rs`
- Pattern: Operations return `RepairStats` showing what changed
- Examples: `stitch_vertices()`, `harmonize_orientation()`, `fill_holes()`
- Philosophy: Repairs are opt-in; immutable models don't silently change

**ModelVisitor (SAX Pattern):**
- Purpose: Callback-based streaming parser for large files
- Location: `crates/lib3mf-core/src/parser/visitor.rs`
- Pattern: User implements `ModelVisitor` trait with callbacks (`on_vertex()`, `on_triangle()`, etc.)
- Used by: `parse_model_streaming()` for O(1) memory with large meshes
- Alternative to: DOM parsing which loads entire model into memory

## Entry Points

**CLI Entry:**
- Location: `crates/lib3mf-cli/src/main.rs`
- Triggers: `cargo run -p lib3mf-cli -- <command>`
- Commands: `stats`, `list`, `validate`, `copy`, `diff`, `extract`, `repair`, `split`, `thumbnails`
- Responsibilities: Parse args, open file, call lib3mf-core, format output

**Library Entry:**
- Location: `crates/lib3mf-core/src/lib.rs`
- Re-exports: `Model`, `Geometry`, `Mesh`, `Object`, `ResourceId`, `parse_model`, `ValidationLevel`
- Typical usage:
  ```rust
  use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
  use lib3mf_core::parser::parse_model;
  let mut archiver = ZipArchiver::new(file)?;
  let path = find_model_path(&mut archiver)?;
  let data = archiver.read_entry(&path)?;
  let model = parse_model(std::io::Cursor::new(data))?;
  ```

**Converter Entry:**
- Location: `crates/lib3mf-converters/src/lib.rs`
- Modules: `stl`, `obj`
- Converts external formats (STL, OBJ) to `lib3mf_core::Model`
- Used by: CLI, converter examples

**Async Entry:**
- Location: `crates/lib3mf-async/src/lib.rs`
- Modules: `loader`, `archive`, `zip`
- Provides tokio-based async file loading and archive reading
- Alternative to: Sync parsing in lib3mf-core

**WebAssembly Entry:**
- Location: `crates/lib3mf-wasm/src/lib.rs`
- Wraps: Core functionality in `WasmModel` binding for browser usage
- Uses: `wasm-bindgen` for JS interoperability

## Error Handling

**Strategy:** Custom `Lib3mfError` enum with no panics in library code; errors expected and recoverable.

**Patterns:**
- `Lib3mfError` variants: `Io`, `Validation`, `ResourceNotFound`, `InvalidStructure`, `EncryptionError`
- Result type alias: `type Result<T> = std::result::Result<T, Lib3mfError>`
- Parser uses `.map_err(|_| Lib3mfError::Validation(...))` to provide context
- Validation returns `ValidationReport` struct containing all issues (non-fatal)
- Resource registry returns `Err` on duplicate ID (caught at add time, not later)

## Cross-Cutting Concerns

**Logging:** Not integrated; examples and CLI use `println!` for diagnostics

**Validation:** Progressive four-level system:
- `Minimal`: Basic XML structure and required attributes
- `Standard`: Reference integrity (object IDs exist, properties valid)
- `Strict`: Full spec compliance (metadata presence, unknown attributes rejected)
- `Paranoid`: Deep geometry (manifoldness, self-intersection via BVH)

**Authentication:** Secure Content Extension handles digital signatures and X.509 certificates

**Parallel Processing:** Feature-gated `parallel` enables `rayon` for mesh AABB computation and stats calculation; controlled via `#[cfg(feature = "parallel")]`

**Properties System:** Hierarchical material resolution:
- Object level: `object.pid` + `object.pindex` (defaults)
- Triangle level: `triangle.pid` or per-vertex `p1`, `p2`, `p3` (overrides)
- Resolution: Triangle → Vertex → Object → None

---

*Architecture analysis: 2026-02-02*
