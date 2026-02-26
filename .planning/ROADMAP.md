# Roadmap: lib3mf-rs

## Milestones

- v0.2.0 Complete Implementation - Phases 1-11 (shipped 2026-02-25)
- v0.3.0 Writer Completeness & Roundtrip Fidelity - Phases 12-14 (shipped 2026-02-25)
- v0.4.0 Format Converters - Phases 18-19 (planned)
- v0.5.0 Advanced CLI & Tooling - Phases 22-25 (planned)
- v0.6.0 Production Hardening - Phases 26-29 (planned)

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

<details>
<summary>v0.2.0 Complete Implementation (Phases 1-11) - SHIPPED 2026-02-25</summary>

- [x] **Phase 1: Object Type Differentiation** - 3/3 plans
- [x] **Phase 2: Boolean Operations Extension** - 4/4 plans
- [x] **Phase 3: Displacement Extension** - 4/4 plans
- [x] **Phase 4: Comprehensive Fuzzing Infrastructure** - 4/4 plans
- [x] **Phase 5: Extension Test Coverage Enhancement** - 5/5 plans
- [x] **Phase 6: Feature Flag Optimization** - 3/3 plans
- [x] **Phase 7: Comprehensive Documentation and GitHub Pages** - 4/4 plans
- [x] **Phase 8: Document Remaining Crates** - 3/3 plans
- [x] **Phase 9: Competitive Differentiation and Market Positioning** - 4/4 plans
- [x] **Phase 10: Achieve 90%+ Conformance (100% MUSTPASS)** - 4/4 plans
- [x] **Phase 11: Bambu Lab 3MF Full Support** - 3/3 plans

</details>

<details>
<summary>v0.3.0 Writer Completeness & Roundtrip Fidelity (Phases 12-14) - SHIPPED 2026-02-25</summary>

- [x] **Phase 12: Beam Lattice Writer** (1/1 plans) — completed 2026-02-25
- [x] **Phase 13: Slice Extension Writer** (2/2 plans) — completed 2026-02-25
- [x] **Phase 14: Volumetric Extension Writer** (1/1 plans) — completed 2026-02-25

</details>

### v0.4.0 Format Converters (Planned)

**Milestone Goal:** Add ASCII STL format support to complete STL coverage (binary + ASCII read/write), and fix production extension component resolution for BambuStudio/OrcaSlicer real-world 3MF files.

- [ ] **Phase 18: ASCII STL Support** - Read/write ASCII STL format with auto-detection, face normals, multi-solid support, and tests
- [ ] **Phase 19: Production Extension Component Resolution** - Resolve cross-file component references, apply transforms, filter by ObjectType, handle unit conversion for real-world BambuStudio/OrcaSlicer 3MF files

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

### Phase 19: Production Extension Component Resolution
**Goal**: Consumers of lib3mf-core can correctly load BambuStudio/OrcaSlicer 3MF files that use production extension cross-file component references (`<component p:path="...">`), with proper transform application, ObjectType filtering, unit conversion, and build item printable flag support.

**Depends on**: Phase 18

**Requirements**: TBD

**Success Criteria** (what must be TRUE):
  1. `PartResolver` correctly resolves `Geometry::Components` with cross-file `path` references, loading sub-model files from the archive on demand
  2. Component transforms (`Mat4`) and BuildItem transforms are correctly composed and applied to vertices (`build_item.transform * component.transform * vertex`)
  3. Recursive component resolution works (component → component → mesh) with a depth limit to prevent infinite loops
  4. `ObjectType::Other` modifier volumes are filtered out — only `ObjectType::Model` objects contribute geometry
  5. `BuildItem.printable == Some(false)` items are skipped
  6. Unit conversion normalizes all vertex coordinates to millimeters based on `model.unit`
  7. Real BambuStudio 3MF test files (Cube_PLA, 3DBenchy_PLA, SimplePyramid) parse successfully with correct vertex/triangle counts
  8. Existing tests continue to pass — no regressions in non-production-extension 3MF handling

**Plans:** TBD

Plans:
- [ ] 19-01-PLAN.md -- TBD

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

**Plans:** TBD

### Phase 23: Split Command
**Goal**: Users can extract individual objects or plates from a 3MF file into separate output files.

**Depends on**: Phase 22

**Requirements**: TBD (defined during milestone activation)

**Plans:** TBD

### Phase 24: Batch Processing
**Goal**: Users can process directories of 3MF files with configurable operations (validate, stats, convert) in a single command.

**Depends on**: Phase 23

**Requirements**: TBD (defined during milestone activation)

**Plans:** TBD

### Phase 25: OBJ Materials Import
**Goal**: OBJ converter parses .mtl material library files during import, preserving material definitions in the 3MF model.

**Depends on**: Phase 24

**Requirements**: TBD (defined during milestone activation)

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

**Plans:** TBD

### Phase 27: Performance Regression CI
**Goal**: Automated benchmark tracking per commit with alerts when performance regresses beyond threshold.

**Depends on**: Phase 26

**Requirements**: TBD (defined during milestone activation)

**Plans:** TBD

### Phase 28: crates.io Publication
**Goal**: All 5 workspace crates published to crates.io with proper metadata, dependency ordering, and publication workflow.

**Depends on**: Phase 27

**Requirements**: TBD (defined during milestone activation)

**Plans:** TBD

### Phase 29: Conformance CI
**Goal**: 3MF Consortium test suite runs in CI pipeline on every PR, preventing conformance regressions.

**Depends on**: Phase 28

**Requirements**: TBD (defined during milestone activation)

**Plans:** TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> ... -> 14 -> 18 -> 19 -> 22 -> 23 -> 24 -> 25 -> 26 -> 27 -> 28 -> 29

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
| 13. Slice Extension Writer | v0.3.0 | 2/2 | Complete | 2026-02-25 |
| 14. Volumetric Extension Writer | v0.3.0 | 1/1 | Complete | 2026-02-25 |
| 18. ASCII STL Support | v0.4.0 | 0/TBD | Not started | - |
| 19. Production Extension Component Resolution | v0.4.0 | 0/TBD | Not started | - |
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
*Last updated: 2026-02-26 (Phase 19 added to v0.4.0)*
