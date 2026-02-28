# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-26)

**Core value:** Complete, production-ready Rust implementation of all 3MF specifications that can be trusted for critical manufacturing workflows
**Current focus:** v0.5.0 milestone SHIPPED — all 5 phases complete, Phase 25 verified (8/8 must-haves)

## Current Position

Phase: 27-streaming-parser-for-extensions (in progress)
Plan: 1/1 complete
Status: Phase 27 plan 01 complete — all tasks done, 9/9 tests pass
Last activity: 2026-02-28 — Completed 27-01-PLAN.md

Progress: v0.5.0: 5/5 phases complete (Phase 22, Phase 23, Phase 24, Phase 25, Phase 26 all complete)
Progress: v0.6.0: Phase 31 complete, Phase 27 plan 01 complete (2/N)

Progress bar: ██████████████████████████░░░░░░░░░░░░░░░░ (62 of 100 plans)

**Last Milestone:** v0.5.0 Advanced CLI & Tooling — SHIPPED 2026-02-27
**Current Milestone:** v0.6.0 (in progress)

## Performance Metrics

**Velocity:**
- Total plans completed: 39 (v0.2.0) + 4 (v0.3.0) + 4 (v0.4.0) + 4 (phase 26) + 3 (phase 22) + 2 (phase 23) + 2 (phase 24) + 2 (phase 25) = 60
- Average duration: 13.2 minutes (v0.2.0), ~10 minutes (v0.3.0), ~4-5 min (v0.4.0), ~2-4 min (phase 26), ~4 min (22-01, 22-02, 22-03), 3 min (23-01), 8 min (23-02), 7 min (24-01), 5 min (25-01), 11 min (25-02)

## Accumulated Context

### Phase 27 Streaming Parser for Extensions Deliverables

- **Plan 01:** 8 new ModelVisitor trait methods with default no-ops (on_start_beam_lattice, on_beam, on_end_beam_lattice, on_start_displacement_mesh, on_displacement_vertex, on_displacement_triangle, on_displacement_normal, on_end_displacement_mesh). 6 new streaming parse functions (parse_beam_lattice_streaming, parse_beams_streaming, parse_displacement_mesh_streaming, parse_displacement_vertices_streaming, parse_displacement_triangles_streaming, parse_displacement_normals_streaming). parse_mesh_streaming updated to accept object_id and dispatch beamlattice. parse_object_content_streaming updated to dispatch displacementmesh. BeamSets and disp2dgroups skipped via read_to_end. All extension element matching uses local_name(). streaming_stats.rs updated with 6 extension fields and callbacks. streaming_extension_test.rs with 9 integration tests.

### Phase 31 QA Suite Subcommand Coverage Deliverables

- **Plan 01:** skip_test() helper (yellow [SKIP] output + SKIP_COUNT tracking). create_minimal_3mf() helper (OPC container tetrahedron, 3 params: file, name, x_offset). MERGE_A/MERGE_B synthetic files generated unconditionally. 5 end-to-end merge tests (2-file merge, --single-plate, --force, nonexistent files, single-input failure). else skip_test branches on all 6 split conditional guards and all 11 batch conditional guards. Updated coverage summary line for Merge Command. OBJ materials section unchanged.

### Phase 25 OBJ Materials Import Deliverables

- **Plan 01:** MTL parser module (mtl.rs) with parse_mtl/parse_mtl_file, MtlMaterial struct, 12 unit tests. OBJ importer rewritten with two-pass architecture (parse_obj -> ObjIntermediate -> build_model), read_from_path() for material-aware import, read() backward-compat path, group splitting via g/o directives, per-triangle pid/p1/p2/p3 material assignment via BaseMaterialsGroup, vertex remapping per group, 11 unit tests.
- **Plan 02:** CLI open_model() and batch process_obj_file() wired to read_from_path(). 10 integration tests in obj_materials_tests.rs (479 lines) covering convert, stats, validate, batch with materials. QA suite OBJ Materials Import section with 5 scenarios. Fixed batch OBJ stats hardcoded base_materials_count: 0.

### Phase 26 Deliverables

- Binary STL writer: 3 unit tests (byte-level format, roundtrip, multi-object) in stl.rs
- Core roundtrip: vendor namespace + beam lattice radius integration tests
- CLI convert: 3 integration tests (ASCII, binary, Benchy real-file)
- QA suite: --ascii section, --no-cleanup flag, real-file integration tests (tmp/models/ discovery)

### Phase 22 Merge Command Deliverables

- **Plan 01:** Complete merge engine — iter_texture_2d(), remap_model (all 20 ID fields), load_full, merge_attachments, merge_metadata, check_secure_content, merge_relationships, merge_extra_namespaces
- **Plan 02:** CLI subcommand wired up — run() pipeline: glob expand → load → validate → remap → merge → placement → atomic write → summary; resolve_output_path() with auto-increment; transfer_resources(); check_build_item_overlaps(); apply_single_plate_placement() with 10mm grid spacing; 1045 lines total in merge.rs
- **Plan 03:** 14 integration tests in crates/lib3mf-cli/tests/merge_tests.rs; QA suite merge section; CLI subprocess test pattern established

### Phase 23 Split Command Deliverables

- **Plan 01:** 863-line core split engine — DependencyCollector (transitive resource graph walk with cycle detection), build_compact_remap (sequential IDs 1-N), build_split_model (filters all 10 resource types, full ID remap), output naming with collision handling, ByItem/ByObject modes, --select/--dry-run/--preserve-transforms/--force
- **Plan 02:** CLI subcommand wired with all 9 flags, 16 integration tests (858 lines) in split_tests.rs, QA suite split section with 7 scenarios. Fixed latent bug: model_writer.rs obj.pid/obj.pindex never written to XML.

### Phase 24 Batch Processing Deliverables

- **Plan 01:** 1101-line batch engine — DetectedFileType enum, detect_file_type() (magic bytes: PK ZIP, "solid" ASCII STL, extension fallback), discover_files() (glob expand + walkdir + canonical dedup), BatchOps/BatchConfig structs, process_file() dispatching to 3MF/STL/OBJ handlers, run_validate_op() calling model.validate() directly, run_stats_op() calling model.compute_stats() directly, run_list_op() calling archiver.list_entries() directly, run() pipeline with rayon parallel, JSON Lines output, 100+ file warning, error accumulation, 11 unit tests.
- **Plan 02:** CLI wiring — Commands::Batch with 15 flags wired to batch::run(), exit code 0/1 convention, 20 integration tests in batch_tests.rs, 15 QA scenarios in qa_test_suite.sh. Fixed QA auto-discovery false failures (Rule 1 bug).

### Pending Todos

1. Support part-level thumbnail relationships (writer) -- package_writer.rs:103
2. Validate certificate chain in crypto sign/verify (crypto) -- commands.rs:1617
3. PNG validation for displacement textures (validation) -- displacement.rs:192-199
4. Deeper hash comparison for model diff (general) -- diff.rs:120

### Roadmap Evolution

- Phase 26 added: Close Test Coverage Gaps (v0.3.0/v0.4.0 features — binary STL writer, CLI --ascii, resolver edge cases)
- v0.6.0 phases renumbered: 26→27, 27→28, 28→29, 29→30
- Phase 31 added: QA Suite Subcommand Coverage (update qa_test_suite.sh for new subcommands, graceful skip on missing files)

### Phase 25 OBJ Materials Import Decisions

| Decision | Rationale |
|---|---|
| Separate mtl.rs module | Keeps MTL parsing isolated and independently testable |
| Dual entry points (read vs read_from_path) | read() preserves exact backward compat; read_from_path() enables full features |
| BaseMaterialsGroup gets ID 1, Objects get IDs 2+ | Simple sequential ID assignment, materials come first in resource order |
| Single BaseMaterialsGroup for all materials | Avoids ID bloat, correct 3MF pattern per research |
| Batch OBJ stats counts dynamically | With read_from_path, model contains actual materials; hardcoded 0 was a bug |

### Phase 22 Merge Command Decisions

| Decision | Rationale |
|---|---|
| Collect-remap-rebuild for ResourceCollection | std::mem::take to drain, mutate owned copies, rebuild via add_* methods (private HashMap fields) |
| Error on secure content | If resources.key_store.is_some(), bail before any merging |
| Attachment dedup | Byte-identical content reuses path; different content gets .{file_index} suffix |
| Metadata merge | Semicolon concatenation for duplicate keys |
| Atomic write | Write to .tmp then rename to prevent partial output on failure |
| Grid placement | ceil(sqrt(n)) columns with per-cell size tracking and 10mm spacing |
| Auto-increment suffix | .1/.2/... appended to full path including extension |
| Test via CLI subprocess | merge functions are pub(crate), not accessible from integration tests directly |

### Phase 23 Split Command Decisions

| Decision | Rationale |
|---|---|
| Compact ID remap (1-N sequential) | Split creates fresh ID spaces per output file — no offset arithmetic, just sort needed IDs and assign 1, 2, 3... |
| Two-phase trace-then-write | Trace ALL deps before creating output dir — prevents partial empty directory on failure |
| Reuse merge::load_full + check_secure_content | Identical loading/checking pattern, consistent error messages |
| ByObject filters by can_be_in_build() | Exclude ObjectType::Other per spec — cannot appear in build items |
| Metadata provenance keys | Copy source metadata + add Source and SourceObject keys |
| Fix model_writer pid/pindex (Rule 1 bug) | Object pid/pindex were never written to XML — material references silently dropped on all roundtrips |

### Phase 24 Batch Decisions

| Decision | Rationale |
|---|---|
| Direct API calls only (no commands::*) | commands::validate/stats/list call process::exit() — incompatible with accumulation |
| BatchConfig struct for run() | Avoids clippy too_many_arguments (>7 params) — idiomatic Rust grouping |
| Rayon ThreadPool with num_threads=jobs | Deterministic output order (sort by index after parallel collect) |
| JSON Lines (NDJL) output | Streaming-friendly, pipeable to jq, specified in plan |
| Extension fallback for binary STL | Binary STL has no reliable short magic header; .stl extension is definitive |
| batch::run() returns Ok(bool) | true=all succeeded; main.rs calls std::process::exit(1) when !all_succeeded |
| QA auto-discovery batch skip | batch added to special-case list (requires positional inputs) to prevent false failures |

### Phase 27 Streaming Parser Decisions

| Decision | Rationale |
|---|---|
| Distinct names on_displacement_vertex etc. | Displacement mesh vertices have different semantics from regular mesh vertices; reusing on_vertex would prevent distinguishing context |
| BeamSets skipped via read_to_end | BeamSet refs are index-based, requiring all beams known first — violates one-pass streaming |
| disp2dgroups skipped via read_to_end | Two-level nesting, small secondary data; use DOM mode for gradients |
| local_name() for extension elements | Namespace-prefixed elements like <bl:beamlattice> would fail with name() |
| object_id threaded into parse_mesh_streaming | on_start_beam_lattice needs context to match on_start_mesh pattern |

### Phase 31 QA Suite Decisions

| Decision | Rationale |
|---|---|
| Synthetic 3MF files generated unconditionally | Merge tests always run, no dependency on models/Benchy.3mf |
| x_offset arithmetic for vertex positioning | Separates ObjectA and ObjectB spatially in merged output |
| OBJ materials section unchanged | Already generates inputs inline, no Benchy dependency |
| skip_test() increments SKIP_COUNT | SKIP_COUNT in summary accurately reflects all skipped conditional tests |

### Blockers/Concerns

None currently.

## Session Continuity

Last session: 2026-02-28
Stopped at: Completed 27-01-PLAN.md — streaming extension callbacks for beam lattice and displacement mesh
Resume file: None
Next: Plan or execute remaining Phase 27 plans (if any), or move to next v0.6.0 phase
