# Roadmap: lib3mf-rs

## Milestones

- v0.2.0 Complete Implementation - Phases 1-11 (shipped 2026-02-25)
- v0.3.0 Writer Completeness & Roundtrip Fidelity - Phases 12-14 (in progress)
- v0.4.0 Format Converters - Phase 18 (planned)
- v0.5.0 Advanced CLI & Tooling - Phases 22-25 (planned)
- v0.6.0 Production Hardening - Phases 26-29 (planned)

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

<details>
<summary>v0.2.0 Complete Implementation (Phases 1-11) - SHIPPED 2026-02-25</summary>

- [x] **Phase 1: Object Type Differentiation** - Implement proper object type handling (model/support/solidsupport/surface/other)
- [x] **Phase 2: Boolean Operations Extension** - Add boolean shape support (union/difference/intersection)
- [x] **Phase 3: Displacement Extension** - Implement displacement mesh with texture-driven surface modification
- [x] **Phase 4: Comprehensive Fuzzing Infrastructure** - Design and implement production-grade fuzzing strategy with multiple targets, CI/CD integration, and automated security testing
- [x] **Phase 5: Extension Test Coverage Enhancement** - Expand test coverage for Secure Content, Beam Lattice, Slice, and Volumetric extensions to achieve 80%+ coverage
- [x] **Phase 6: Feature Flag Optimization** - Implement cargo feature flags to make crypto and parallel dependencies optional, reducing default dependency footprint from 338 to ~60 crates
- [x] **Phase 7: Comprehensive Documentation and GitHub Pages** - Create production-quality documentation with mdBook narrative guides and enhanced rustdoc API references, published to GitHub Pages with automated workflows
- [x] **Phase 8: Document Remaining Crates** - Create comprehensive rustdoc API documentation for lib3mf-async, lib3mf-cli, lib3mf-converters, and lib3mf-wasm with examples and usage patterns
- [x] **Phase 9: Competitive Differentiation and Market Positioning** - Analyze competitor implementations, optimize branding/SEO, create comparison documentation, implement 3MF Consortium test suite validation, establish performance benchmarks, and execute content marketing strategy for community visibility
- [x] **Phase 10: Achieve 90%+ Conformance (100% MUSTPASS)** - Systematically identify and fix validation gaps for 90% overall conformance with 100% MUSTPASS parsing
- [x] **Phase 11: Bambu Lab 3MF Full Support** - Comprehensive parsing and extraction of Bambu Studio/OrcaSlicer vendor data

### Phase 1: Object Type Differentiation
**Goal**: Object types are properly parsed, stored, written, and validated throughout the library.
**Plans**: 3/3 complete

Plans:
- [x] 01-01-PLAN.md -- Core model: ObjectType enum, Object struct, parser, writer
- [x] 01-02-PLAN.md -- Type-specific validation rules for geometry and build items
- [x] 01-03-PLAN.md -- CLI stats display, unit tests, documentation

### Phase 2: Boolean Operations Extension
**Goal**: Boolean shape operations (union, difference, intersection) are fully supported for reading, writing, and validation.
**Plans**: 4/4 complete

Plans:
- [x] 02-01-PLAN.md -- Data structures (BooleanShape, BooleanOperation) and parser
- [x] 02-02-PLAN.md -- Writer with namespace declarations and XML output
- [x] 02-03-PLAN.md -- Validation (reference integrity, cycle detection, transforms)
- [x] 02-04-PLAN.md -- Unit tests, integration tests, and code example

### Phase 3: Displacement Extension
**Goal**: Displacement mesh with texture-driven surface modification is fully implemented for reading, writing, and validation.
**Plans**: 4/4 complete

Plans:
- [x] 03-01-PLAN.md -- Data structures (DisplacementMesh, Displacement2D) and parser
- [x] 03-02-PLAN.md -- Writer with namespace declarations and XML output
- [x] 03-03-PLAN.md -- Validation (normal vectors, texture references, PNG support)
- [x] 03-04-PLAN.md -- CLI stats, tests, and code example

### Phase 4: Comprehensive Fuzzing Infrastructure
**Goal**: Production-grade fuzzing infrastructure covering all parser entry points with CI/CD integration.
**Plans**: 4/4 complete

Plans:
- [x] 04-01-PLAN.md -- Fuzz target infrastructure (7 targets, workspace config, Cargo.toml)
- [x] 04-02-PLAN.md -- Corpus seeding and dictionary files
- [x] 04-03-PLAN.md -- GitHub Actions CI workflow with crash triage
- [x] 04-04-PLAN.md -- Documentation (CLAUDE.md, fuzz/README.md)

### Phase 5: Extension Test Coverage Enhancement
**Goal**: Achieve 80%+ test coverage for all extensions.
**Plans**: 5/5 complete

Plans:
- [x] 05-01-PLAN.md -- Secure Content comprehensive test suite
- [x] 05-02-PLAN.md -- Beam Lattice and Slice extension tests
- [x] 05-03-PLAN.md -- Volumetric extension tests and example code
- [x] 05-04-PLAN.md -- Cross-extension integration tests and coverage measurement
- [x] 05-05-PLAN.md -- Gap closure: Error path tests for 80%+ coverage

### Phase 6: Feature Flag Optimization
**Goal**: Make crypto and parallel dependencies optional via cargo feature flags.
**Plans**: 3/3 complete

Plans:
- [x] 06-01-PLAN.md -- Core feature flags: Cargo.toml definitions, cfg gates
- [x] 06-02-PLAN.md -- Downstream crates, test gating, feature combinations
- [x] 06-03-PLAN.md -- CI workflow feature matrix, QA suite, documentation

### Phase 7: Comprehensive Documentation and GitHub Pages
**Goal**: Production-quality documentation with mdBook guides and rustdoc API references.
**Plans**: 4/4 complete

Plans:
- [x] 07-01-PLAN.md -- Rustdoc: crate-level and module-level documentation
- [x] 07-02-PLAN.md -- Rustdoc: key public types documentation
- [x] 07-03-PLAN.md -- mdBook narrative guide (8 chapters)
- [x] 07-04-PLAN.md -- GitHub Actions docs workflow, landing page, README

### Phase 8: Document Remaining Crates
**Goal**: All workspace crates have comprehensive rustdoc API documentation.
**Plans**: 3/3 complete

Plans:
- [x] 08-01-PLAN.md -- Rustdoc for lib3mf-converters and lib3mf-async
- [x] 08-02-PLAN.md -- Rustdoc for lib3mf-wasm and lib3mf-cli
- [x] 08-03-PLAN.md -- Integration: workspace doc verification, mdBook update

### Phase 9: Competitive Differentiation and Market Positioning
**Goal**: Establish lib3mf-rs as the enterprise-grade Rust 3MF implementation.
**Plans**: 4/4 complete

Plans:
- [x] 09-01-PLAN.md -- Competitive analysis and 3MF Consortium test suite
- [x] 09-02-PLAN.md -- Performance benchmark suite and comparison metrics
- [x] 09-03-PLAN.md -- Branding optimization: Cargo.toml SEO, comparison docs, README
- [x] 09-04-PLAN.md -- Content marketing execution and community engagement

### Phase 10: Achieve 90%+ Conformance (100% MUSTPASS)
**Goal**: 90%+ overall conformance with 100% MUSTPASS and maximum practical MUSTFAIL detection.
**Plans**: 4/4 complete

Plans:
- [x] 10-01-PLAN.md -- OPC/archive layer validation
- [x] 10-02-PLAN.md -- Parser structure validation (integrated into 10-01/10-03/10-04)
- [x] 10-03-PLAN.md -- Texture2D resource parsing with path/contenttype validation
- [x] 10-04-PLAN.md -- Finalization: remaining gaps, documentation, state updates

### Phase 11: Bambu Lab 3MF Full Support
**Goal**: Comprehensive parsing and extraction of Bambu Studio/OrcaSlicer vendor data.
**Plans**: 3/3 complete

Plans:
- [x] 11-01-PLAN.md -- Data model foundation: enriched VendorData structs, printable attribute
- [x] 11-02-PLAN.md -- Bambu config parsers: slice_info, model_settings, project_settings
- [x] 11-03-PLAN.md -- Stats integration, CLI display enrichment, integration tests

</details>

### v0.3.0 Writer Completeness & Roundtrip Fidelity (In Progress)

**Milestone Goal:** Complete all extension writers (beam lattice, slice, volumetric) to enable full roundtrip fidelity for every 3MF extension the library parses. Fix model_writer object attribute architecture to support extension-specific attributes (slicestackid, volumetricstackid).

- [x] **Phase 12: Beam Lattice Writer** - Serialize BeamLattice data back to XML with namespace declarations, beam/beamset elements, and full roundtrip fidelity
- [ ] **Phase 13: Slice Extension Writer** - Serialize slice stacks with polygons, contours, external references, and refactor model_writer for slicestackid attribute
- [ ] **Phase 14: Volumetric Extension Writer** - Serialize volumetric stacks with layers and external references, leveraging model_writer refactor from Phase 13

## Phase Details

### Phase 12: Beam Lattice Writer
**Goal**: Users can write 3MF files containing beam lattice data and get identical structural output when roundtripping through parse-write-parse.

**Depends on**: Phase 11 (continues from Milestone 1)

**Requirements**: BLW-01, BLW-02, BLW-03, BLW-04, BLW-05, BLW-06, BLW-07, BLW-08

**Success Criteria** (what must be TRUE):
  1. A 3MF file containing beam lattice data can be parsed, written back to a new file, and re-parsed with structurally identical beam lattice content (beams, radii, cap modes, beam sets all preserved)
  2. The beam lattice XML namespace (`http://schemas.microsoft.com/3dmanufacturing/beamlattice/2017/02`) appears on the model element when beam lattice objects are present
  3. All CapMode and ClippingMode enum variants produce correct XML attribute strings and survive roundtrip without data loss
  4. The CLI `copy` command produces output files where beam lattice data is intact and can be re-inspected with `stats`

**Plans:** 1/1 complete

Plans:
- [x] 12-01-PLAN.md -- Beam lattice writer implementation, integration, and roundtrip tests

### Phase 13: Slice Extension Writer
**Goal**: Users can write 3MF files containing slice stack data (2D geometry slices, polygons, external references) and the model_writer correctly emits slicestackid on object elements.

**Depends on**: Phase 12

**Requirements**: SLW-01, SLW-02, SLW-03, SLW-04, SLW-05, SLW-06, SLW-07, SLW-08, SLW-09, SLW-10

**Success Criteria** (what must be TRUE):
  1. A 3MF file containing slice stack data (vertices, polygons, segments, slicerefs) can be parsed, written, and re-parsed with structurally identical slice content
  2. The slice XML namespace appears on the model element, and slicestackid attribute is correctly emitted on object elements that reference slice stacks (requires model_writer refactor)
  3. ResourceCollection exposes an iter_slice_stacks() method that writer code can use to iterate over all slice stack resources
  4. External slice references (sliceref elements) are serialized and roundtripped correctly, preserving path and slicestackid attributes
  5. Polygon segments with optional property attributes (p1, p2, pid) are serialized without data loss

**Plans:** TBD

Plans:
- [ ] 13-01-PLAN.md -- TBD
- [ ] 13-02-PLAN.md -- TBD

### Phase 14: Volumetric Extension Writer
**Goal**: Users can write 3MF files containing volumetric stack data (layers, external references) and the model_writer correctly emits volumetricstackid on object elements.

**Depends on**: Phase 13 (leverages model_writer object attribute refactor from SLW-08)

**Requirements**: VLW-01, VLW-02, VLW-03, VLW-04, VLW-05, VLW-06, VLW-07

**Success Criteria** (what must be TRUE):
  1. A 3MF file containing volumetric stack data (layers, refs) can be parsed, written, and re-parsed with structurally identical volumetric content
  2. The volumetric XML namespace appears on the model element, and volumetricstackid attribute is correctly emitted on object elements that reference volumetric stacks (reuses model_writer refactor from Phase 13)
  3. ResourceCollection exposes an iter_volumetric_stacks() method that writer code can use to iterate over all volumetric stack resources
  4. All volumetric layer and ref attributes (z, path, stackid) are serialized and roundtripped without data loss

**Plans:** TBD

Plans:
- [ ] 14-01-PLAN.md -- TBD
- [ ] 14-02-PLAN.md -- TBD

### v0.4.0 Format Converters (Planned)

**Milestone Goal:** Add ASCII STL format support to complete STL coverage (binary + ASCII read/write). Currently only binary STL is supported.

- [ ] **Phase 18: ASCII STL Support** - Read/write ASCII STL format with auto-detection, face normals, multi-solid support, and tests

### Phase 18: ASCII STL Support
**Goal**: Users can read and write ASCII STL files with auto-detection between binary and ASCII formats, completing full STL format coverage.

**Depends on**: Phase 14

**Requirements**: ASTL-01 through ASTL-16

**Success Criteria** (what must be TRUE):
  1. StlImporter::read() auto-detects binary vs ASCII format and parses both correctly without caller intervention
  2. ASCII STL files with varying whitespace, casing, and multiple solids parse into correct Model structures with deduplicated vertices
  3. ASCII STL writer emits computed face normals (not zero normals) and produces valid ASCII STL files that other tools can import
  4. Roundtrip test passes: ASCII STL -> Model -> ASCII STL -> Model with structurally identical geometry
  5. README documentation bug fixed (currently incorrectly claims ASCII STL support exists)

**Plans:** TBD

Plans:
- [ ] 18-01-PLAN.md -- TBD
- [ ] 18-02-PLAN.md -- TBD

### v0.5.0 Advanced CLI & Tooling (Planned)

**Milestone Goal:** Add advanced CLI commands for common 3MF workflows: merging files, splitting objects, batch processing, and OBJ material import.

- [ ] **Phase 22: Merge Command** - Combine multiple 3MF files into a single output file
- [ ] **Phase 23: Split Command** - Extract individual objects/plates to separate 3MF files
- [ ] **Phase 24: Batch Processing** - Process directories of 3MF files with configurable operations
- [ ] **Phase 25: OBJ Materials Import** - Parse .mtl files during OBJ import for material support

### Phase 22: Merge Command
**Goal**: Users can combine multiple 3MF files into a single file, merging resources, build items, and metadata.

**Depends on**: Phase 18

**Requirements**: TBD (defined during milestone activation)

**Success Criteria** (what must be TRUE):
  1. CLI `merge` command accepts 2+ input 3MF files and produces a single combined output
  2. Resource IDs are remapped to avoid conflicts between source files
  3. Build items from all inputs are preserved with correct resource references
  4. Metadata and relationships are merged without data loss

**Plans:** TBD

### Phase 23: Split Command
**Goal**: Users can extract individual objects or plates from a 3MF file into separate output files.

**Depends on**: Phase 22

**Requirements**: TBD (defined during milestone activation)

**Success Criteria** (what must be TRUE):
  1. CLI `split` command extracts specified objects to separate 3MF files
  2. Each output file is a valid standalone 3MF with correct resource references
  3. Plate-based splitting supported for multi-plate Bambu files
  4. Materials and textures referenced by extracted objects are included in output

**Plans:** TBD

### Phase 24: Batch Processing
**Goal**: Users can process directories of 3MF files with configurable operations (validate, stats, convert) in a single command.

**Depends on**: Phase 23

**Requirements**: TBD (defined during milestone activation)

**Success Criteria** (what must be TRUE):
  1. CLI `batch` command processes all 3MF files in a directory
  2. Configurable operations: validate, stats, convert (to STL/OBJ)
  3. Summary report with per-file results and aggregated statistics
  4. Parallel processing support for large directories

**Plans:** TBD

### Phase 25: OBJ Materials Import
**Goal**: OBJ converter parses .mtl material library files during import, preserving material definitions in the 3MF model.

**Depends on**: Phase 24

**Requirements**: TBD (defined during milestone activation)

**Success Criteria** (what must be TRUE):
  1. OBJ importer reads .mtl files referenced by `mtllib` directive
  2. Material definitions (Kd, Ka, Ks, map_Kd) mapped to 3MF BaseMaterial or ColorGroup
  3. Per-face material assignments (`usemtl`) preserved as triangle properties
  4. Missing .mtl files produce warning but don't fail import

**Plans:** TBD

### v0.6.0 Production Hardening (Planned)

**Milestone Goal:** Prepare for production deployment with streaming parser extensions, performance regression CI, crates.io publication, and conformance CI.

- [ ] **Phase 26: Streaming Parser for Extensions** - Add SAX-mode parsing for boolean, displacement, and volumetric extensions to handle GB+ files
- [ ] **Phase 27: Performance Regression CI** - Automated benchmark tracking per commit with regression alerts
- [ ] **Phase 28: crates.io Publication** - Publish all 5 workspace crates with proper metadata and dependency structure
- [ ] **Phase 29: Conformance CI** - Run 3MF Consortium test suite in CI pipeline to prevent conformance regressions

### Phase 26: Streaming Parser for Extensions
**Goal**: Extensions (boolean, displacement, volumetric) work in SAX streaming mode for GB+ files, matching core mesh streaming support.

**Depends on**: Phase 25

**Requirements**: TBD (defined during milestone activation)

**Success Criteria** (what must be TRUE):
  1. ModelVisitor trait extended with callbacks for boolean, displacement, and volumetric elements
  2. Streaming parser invokes extension callbacks without accumulating extension data in memory
  3. GB+ files with extensions parse in constant memory using streaming mode
  4. All existing DOM-mode extension tests pass unchanged

**Plans:** TBD

### Phase 27: Performance Regression CI
**Goal**: Automated benchmark tracking per commit with alerts when performance regresses beyond threshold.

**Depends on**: Phase 26

**Requirements**: TBD (defined during milestone activation)

**Success Criteria** (what must be TRUE):
  1. GitHub Actions workflow runs benchmarks on every PR and push to main
  2. Results compared against baseline with configurable regression threshold (default 5%)
  3. PR comments show benchmark comparison table with deltas
  4. CI fails or warns when regression exceeds threshold

**Plans:** TBD

### Phase 28: crates.io Publication
**Goal**: All 5 workspace crates published to crates.io with proper metadata, dependency ordering, and publication workflow.

**Depends on**: Phase 27

**Requirements**: TBD (defined during milestone activation)

**Success Criteria** (what must be TRUE):
  1. All 5 crates (lib3mf-core, lib3mf-cli, lib3mf-converters, lib3mf-async, lib3mf-wasm) published to crates.io
  2. Dependency ordering correct (core first, then dependents)
  3. GitHub Actions release workflow automates publication on git tags
  4. All Cargo.toml metadata complete (description, license, repository, documentation, keywords, categories)

**Plans:** TBD

### Phase 29: Conformance CI
**Goal**: 3MF Consortium test suite runs in CI pipeline on every PR, preventing conformance regressions.

**Depends on**: Phase 28

**Requirements**: TBD (defined during milestone activation)

**Success Criteria** (what must be TRUE):
  1. GitHub Actions workflow runs conformance tests on every PR
  2. Tests cover both MUSTPASS (valid files parse) and MUSTFAIL (invalid files rejected)
  3. Conformance percentage tracked and reported in PR comments
  4. CI fails if conformance drops below 90% threshold

**Plans:** TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> ... -> 14 -> 18 -> 22 -> 23 -> 24 -> 25 -> 26 -> 27 -> 28 -> 29

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Object Type Differentiation | v0.2.0 | 3/3 | Complete | 2026-02-02 |
| 2. Boolean Operations Extension | v0.2.0 | 4/4 | Complete | 2026-02-02 |
| 3. Displacement Extension | v0.2.0 | 4/4 | Complete | 2026-02-03 |
| 4. Comprehensive Fuzzing Infrastructure | v0.2.0 | 4/4 | Complete | 2026-02-03 |
| 5. Extension Test Coverage Enhancement | v0.2.0 | 5/5 | Complete | 2026-02-03 |
| 6. Feature Flag Optimization | v0.2.0 | 3/3 | Complete | 2026-02-04 |
| 7. Comprehensive Documentation and GitHub Pages | v0.2.0 | 4/4 | Complete | 2026-02-04 |
| 8. Document Remaining Crates | v0.2.0 | 3/3 | Complete | 2026-02-04 |
| 9. Competitive Differentiation and Market Positioning | v0.2.0 | 4/4 | Complete | 2026-02-07 |
| 10. Achieve 90%+ Conformance (100% MUSTPASS) | v0.2.0 | 4/4 | Complete | 2026-02-07 |
| 11. Bambu Lab 3MF Full Support | v0.2.0 | 3/3 | Complete | 2026-02-25 |
| 12. Beam Lattice Writer | v0.3.0 | 1/1 | Complete | 2026-02-25 |
| 13. Slice Extension Writer | v0.3.0 | 0/TBD | Not started | - |
| 14. Volumetric Extension Writer | v0.3.0 | 0/TBD | Not started | - |
| 18. ASCII STL Support | v0.4.0 | 0/TBD | Not started | - |
| 22. Merge Command | v0.5.0 | 0/TBD | Not started | - |
| 23. Split Command | v0.5.0 | 0/TBD | Not started | - |
| 24. Batch Processing | v0.5.0 | 0/TBD | Not started | - |
| 25. OBJ Materials Import | v0.5.0 | 0/TBD | Not started | - |
| 26. Streaming Parser for Extensions | v0.6.0 | 0/TBD | Not started | - |
| 27. Performance Regression CI | v0.6.0 | 0/TBD | Not started | - |
| 28. crates.io Publication | v0.6.0 | 0/TBD | Not started | - |
| 29. Conformance CI | v0.6.0 | 0/TBD | Not started | - |

---
*Roadmap created: 2026-02-02*
*Last updated: 2026-02-25 (Phase 12 complete: beam lattice writer with roundtrip fidelity)*
