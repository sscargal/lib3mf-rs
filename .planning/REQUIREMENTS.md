# Requirements: lib3mf-rs

**Defined:** 2026-02-02
**Updated:** 2026-02-25 (Milestone 4 requirements added)
**Core Value:** Complete, production-ready Rust implementation of all 3MF specifications that can be trusted for critical manufacturing workflows

## Milestone 1 Requirements (v0.2.0) — COMPLETE

All 58 requirements shipped across phases 1-11. See MILESTONES.md for details.

## Milestone 2 Requirements (v0.3.0) — Writer Completeness & Roundtrip Fidelity

Requirements for completing all extension writers to enable full roundtrip fidelity.

### Beam Lattice Writer

- [x] **BLW-01**: Writer emits beam lattice namespace declaration on model element
- [x] **BLW-02**: Writer serializes beamlattice element with minlength, precision, and clipping attributes within mesh
- [x] **BLW-03**: Writer serializes beam elements with v1/v2/r1/r2 and optional cap/p1/p2 attributes
- [x] **BLW-04**: Writer serializes beam set elements with optional name/identifier and ref index children
- [x] **BLW-05**: Writer correctly maps all CapMode values (Sphere/Hemisphere/Butt) to XML attribute strings
- [x] **BLW-06**: Writer correctly maps all ClippingMode values (None/Inside/Outside) to XML attribute strings
- [x] **BLW-07**: Roundtrip test passes for beam lattice models (parse -> write -> parse -> compare)
- [x] **BLW-08**: CLI copy command preserves beam lattice data in output file

### Slice Extension Writer

- [ ] **SLW-01**: Writer emits slice namespace declaration on model element
- [ ] **SLW-02**: Writer serializes slicestack resource elements with id and zbottom attributes
- [ ] **SLW-03**: Writer serializes slice elements with ztop attribute within slicestack
- [ ] **SLW-04**: Writer serializes 2D vertex elements (x, y) within each slice
- [ ] **SLW-05**: Writer serializes polygon elements with start attribute and segment children
- [ ] **SLW-06**: Writer serializes segment elements with v2 and optional p1/p2/pid attributes
- [ ] **SLW-07**: Writer serializes sliceref elements for external slice references
- [ ] **SLW-08**: Writer emits slicestackid attribute on object element for SliceStack geometry (model_writer refactor)
- [ ] **SLW-09**: ResourceCollection provides iter_slice_stacks() method for writer iteration
- [ ] **SLW-10**: Roundtrip test passes for slice extension models (parse -> write -> parse -> compare)

### Volumetric Extension Writer

- [ ] **VLW-01**: Writer emits volumetric namespace declaration on model element
- [ ] **VLW-02**: Writer serializes volumetricstack resource elements with id attribute
- [ ] **VLW-03**: Writer serializes volumetric layer elements with z and path attributes
- [ ] **VLW-04**: Writer serializes volumetric ref elements with stackid and path attributes
- [ ] **VLW-05**: Writer emits volumetricstackid attribute on object element for VolumetricStack geometry (shared refactor)
- [ ] **VLW-06**: ResourceCollection provides iter_volumetric_stacks() method for writer iteration
- [ ] **VLW-07**: Roundtrip test passes for volumetric extension models (parse -> write -> parse -> compare)

## Milestone 4 Requirements (v0.4.0) — Format Converters

Requirements for ASCII STL format support in lib3mf-converters.

### ASCII STL Reader

- [ ] **ASTL-01**: Reader auto-detects binary vs ASCII STL format from file content
- [ ] **ASTL-02**: Reader parses solid/endsolid keywords with optional solid name
- [ ] **ASTL-03**: Reader parses facet normal vectors (nx, ny, nz)
- [ ] **ASTL-04**: Reader parses vertex coordinates within outer loop/endloop blocks
- [ ] **ASTL-05**: Reader deduplicates vertices using bitwise float comparison (matching binary STL behavior)
- [ ] **ASTL-06**: Reader handles whitespace variations and case-insensitive keywords
- [ ] **ASTL-07**: Reader supports multiple solids in a single ASCII STL file (multi-object import)

### ASCII STL Writer

- [ ] **ASTL-08**: Writer serializes geometry to standard ASCII STL format with solid/facet/vertex structure
- [ ] **ASTL-09**: Writer computes and emits face normals from triangle vertices (not zero normals)
- [ ] **ASTL-10**: Writer applies build item transforms to vertex coordinates (matching binary writer)
- [ ] **ASTL-11**: Writer supports PartResolver for multi-part 3MF files (matching binary writer)

### Integration & Testing

- [ ] **ASTL-12**: Unit tests cover ASCII STL read with various formatting (whitespace, case, multi-solid)
- [ ] **ASTL-13**: Unit tests cover ASCII STL write with normal computation verification
- [ ] **ASTL-14**: Roundtrip test passes (ASCII STL -> Model -> ASCII STL -> Model -> compare)
- [ ] **ASTL-15**: Fix README documentation bug (currently incorrectly claims ASCII STL support exists)
- [ ] **ASTL-16**: CLI convert command works with ASCII STL files (auto-detected on input, selectable on output)

## Future Requirements

Tracked but not in current milestone.

- Part-level thumbnail relationships in writer (package_writer.rs:103)
- Certificate chain validation in crypto verify (commands.rs:1617)
- PNG validation for displacement textures (displacement.rs:192-199)
- Deeper hash comparison for model diff (diff.rs:120)
- README version references kept current with releases

## Out of Scope

| Feature | Reason |
|---------|--------|
| Boolean geometry evaluation | CSG processing handled by CAD/slicer software |
| Displacement surface subdivision | Mesh tessellation handled by renderers |
| Texture sampling operations | Graphics pipeline handled by rendering engines |
| G-code generation/evaluation | Slicer responsibility |
| Bambu config round-trip writing | Read-only vendor data extraction |
| Slice geometry computation | Slicer/renderer responsibility; writer only serializes existing data |
| Volumetric voxel generation | Application responsibility; writer only serializes existing data |
| STL color extensions | Non-standard vendor extensions (VisCAM, SolidView) not widely supported |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| BLW-01 | Phase 12 | Complete |
| BLW-02 | Phase 12 | Complete |
| BLW-03 | Phase 12 | Complete |
| BLW-04 | Phase 12 | Complete |
| BLW-05 | Phase 12 | Complete |
| BLW-06 | Phase 12 | Complete |
| BLW-07 | Phase 12 | Complete |
| BLW-08 | Phase 12 | Complete |
| SLW-01 | Phase 13 | Pending |
| SLW-02 | Phase 13 | Pending |
| SLW-03 | Phase 13 | Pending |
| SLW-04 | Phase 13 | Pending |
| SLW-05 | Phase 13 | Pending |
| SLW-06 | Phase 13 | Pending |
| SLW-07 | Phase 13 | Pending |
| SLW-08 | Phase 13 | Pending |
| SLW-09 | Phase 13 | Pending |
| SLW-10 | Phase 13 | Pending |
| VLW-01 | Phase 14 | Pending |
| VLW-02 | Phase 14 | Pending |
| VLW-03 | Phase 14 | Pending |
| VLW-04 | Phase 14 | Pending |
| VLW-05 | Phase 14 | Pending |
| VLW-06 | Phase 14 | Pending |
| VLW-07 | Phase 14 | Pending |
| ASTL-01 | Phase 18 | Pending |
| ASTL-02 | Phase 18 | Pending |
| ASTL-03 | Phase 18 | Pending |
| ASTL-04 | Phase 18 | Pending |
| ASTL-05 | Phase 18 | Pending |
| ASTL-06 | Phase 18 | Pending |
| ASTL-07 | Phase 18 | Pending |
| ASTL-08 | Phase 18 | Pending |
| ASTL-09 | Phase 18 | Pending |
| ASTL-10 | Phase 18 | Pending |
| ASTL-11 | Phase 18 | Pending |
| ASTL-12 | Phase 18 | Pending |
| ASTL-13 | Phase 18 | Pending |
| ASTL-14 | Phase 18 | Pending |
| ASTL-15 | Phase 18 | Pending |
| ASTL-16 | Phase 18 | Pending |

**Coverage:**
- Milestone 2 requirements: 25 total
- Milestone 4 requirements: 16 total
- Total mapped: 41
- Unmapped: 0

**Phase Summary:**
- Phase 12: Beam Lattice Writer — 8 requirements
- Phase 13: Slice Extension Writer — 10 requirements
- Phase 14: Volumetric Extension Writer — 7 requirements
- Phase 18: ASCII STL Support — 16 requirements

---
*Requirements defined: 2026-02-02*
*Last updated: 2026-02-25 after Milestone 4 definition*
