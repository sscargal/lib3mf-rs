# Roadmap: lib3mf-rs

## Milestones

- v0.2.0 Complete Implementation - Phases 1-11 (shipped 2026-02-25)
- v0.3.0 Writer Completeness & Roundtrip Fidelity - Phases 12-14 (shipped 2026-02-25)
- v0.4.0 Format Converters - Phases 18-19 (shipped 2026-02-26)
- v0.5.0 Advanced CLI & Tooling - Phases 22-26 (planned)
- v0.6.0 Production Hardening - Phases 27-30 (planned)

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

<details>
<summary>v0.4.0 Format Converters (Phases 18-19) - SHIPPED 2026-02-26</summary>

**Milestone Goal:** Add ASCII STL format support to complete STL coverage (binary + ASCII read/write), and fix production extension component resolution for BambuStudio/OrcaSlicer real-world 3MF files.

- [x] **Phase 18: ASCII STL Support** (2/2 plans) — completed 2026-02-26
- [x] **Phase 19: Production Extension Component Resolution** (2/2 plans) — completed 2026-02-26

</details>

### v0.5.0 Advanced CLI & Tooling (Planned)

**Milestone Goal:** Add advanced CLI commands for common 3MF workflows: merging files, splitting objects, batch processing, and OBJ material import.

- [ ] **Phase 22: Merge Command** - Combine multiple 3MF files into a single output file
- [ ] **Phase 23: Split Command** - Extract individual objects/plates to separate 3MF files
- [ ] **Phase 24: Batch Processing** - Process directories of 3MF files with configurable operations
- [ ] **Phase 25: OBJ Materials Import** - Parse .mtl files during OBJ import for material support
- [ ] **Phase 26: Close Test Coverage Gaps** - Add missing tests for v0.3.0/v0.4.0 features (binary STL writer, CLI --ascii, multi-object export, resolver edge cases)

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

### Phase 26: Close Test Coverage Gaps
**Goal**: Fill critical test gaps for v0.3.0/v0.4.0 features — binary STL writer tests, binary STL roundtrip, CLI --ascii flag tests, multi-object STL export, vendor namespace roundtrip, and resolver edge cases.

**Depends on**: Phase 25

**Requirements**:
- Binary STL writer (BinaryStlExporter) unit tests: write output validation, normal computation
- Binary STL roundtrip tests: write→read→compare cycle
- CLI `--ascii` flag integration tests: end-to-end 3mf→ASCII STL conversion
- Multi-object STL export tests: multiple meshes in single export
- ASCII STL with real .3mf files: end-to-end 3mf→stl using Benchy/test models
- Vendor namespace (`extra_namespaces`) roundtrip test
- Beam lattice `radius` attribute roundtrip test
- resolve_meshes modifier volume (ObjectType::Other) filtering test
- QA test suite additions for new test scenarios
- Real-file integration tests in QA suite (tmp/models/ dynamic discovery)

**Plans:** 4 plans
Plans:
- [ ] 26-01-PLAN.md — Binary STL writer unit tests (write validation, roundtrip, multi-object)
- [ ] 26-02-PLAN.md — Core roundtrip tests (vendor namespace, beam lattice radius)
- [ ] 26-03-PLAN.md — CLI convert integration tests and QA script --ascii coverage
- [ ] 26-04-PLAN.md — Real-file integration tests in QA suite (--no-cleanup flag, tmp/models/ discovery)

### v0.6.0 Production Hardening (Planned)

**Milestone Goal:** Prepare for production deployment with streaming parser extensions, performance regression CI, crates.io publication, and conformance CI.

- [ ] **Phase 27: Streaming Parser for Extensions** - Add SAX-mode parsing for boolean, displacement, and volumetric extensions to handle GB+ files
- [ ] **Phase 28: Performance Regression CI** - Automated benchmark tracking per commit with regression alerts
- [ ] **Phase 29: crates.io Publication** - Publish all 5 workspace crates with proper metadata and dependency structure
- [ ] **Phase 30: Conformance CI** - Run 3MF Consortium test suite in CI pipeline to prevent conformance regressions

### Phase 27: Streaming Parser for Extensions
**Goal**: Extensions (boolean, displacement, volumetric) work in SAX streaming mode for GB+ files, matching core mesh streaming support.

**Depends on**: Phase 26

**Requirements**: TBD (defined during milestone activation)

**Plans:** TBD

### Phase 28: Performance Regression CI
**Goal**: Automated benchmark tracking per commit with alerts when performance regresses beyond threshold.

**Depends on**: Phase 27

**Requirements**: TBD (defined during milestone activation)

**Plans:** TBD

### Phase 29: crates.io Publication
**Goal**: All 5 workspace crates published to crates.io with proper metadata, dependency ordering, and publication workflow.

**Depends on**: Phase 28

**Requirements**: TBD (defined during milestone activation)

**Plans:** TBD

### Phase 30: Conformance CI
**Goal**: 3MF Consortium test suite runs in CI pipeline on every PR, preventing conformance regressions.

**Depends on**: Phase 29

**Requirements**: TBD (defined during milestone activation)

**Plans:** TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> ... -> 14 -> 18 -> 19 -> 22 -> 23 -> 24 -> 25 -> 26 -> 27 -> 28 -> 29 -> 30

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
| 18. ASCII STL Support | v0.4.0 | 2/2 | Complete | 2026-02-26 |
| 19. Production Extension Component Resolution | v0.4.0 | 2/2 | Complete | 2026-02-26 |
| 22. Merge Command | v0.5.0 | 0/TBD | Not started | - |
| 23. Split Command | v0.5.0 | 0/TBD | Not started | - |
| 24. Batch Processing | v0.5.0 | 0/TBD | Not started | - |
| 25. OBJ Materials Import | v0.5.0 | 0/TBD | Not started | - |
| 26. Close Test Coverage Gaps | v0.5.0 | 0/4 | Not started | - |
| 27. Streaming Parser for Extensions | v0.6.0 | 0/TBD | Not started | - |
| 28. Performance Regression CI | v0.6.0 | 0/TBD | Not started | - |
| 29. crates.io Publication | v0.6.0 | 0/TBD | Not started | - |
| 30. Conformance CI | v0.6.0 | 0/TBD | Not started | - |

---
*Roadmap created: 2026-02-02*
*Last updated: 2026-02-26 (Phase 26 planned: 4 plans in 2 waves)*
