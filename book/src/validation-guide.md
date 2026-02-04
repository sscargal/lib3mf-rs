# Validation Guide

lib3mf-rs provides a comprehensive 4-level progressive validation system to ensure 3MF file integrity and specification compliance.

## Overview

Validation checks 3MF files for errors ranging from basic structure issues to deep geometry problems. The system uses four levels that progressively increase in thoroughness:

1. **Minimal** — Basic structural checks (fastest)
2. **Standard** — Reference integrity and common issues (recommended)
3. **Strict** — Full specification compliance
4. **Paranoid** — Deep geometry analysis (slowest, most thorough)

## Quick Start

```rust
use lib3mf_core::Model;
use lib3mf_core::validation::ValidationLevel;

let model = Model::from_file("model.3mf")?;
let report = model.validate(ValidationLevel::Standard)?;

if report.has_errors() {
    eprintln!("Validation failed: {} errors", report.error_count());
    for error in report.errors() {
        eprintln!("  - {}", error.message);
    }
} else {
    println!("Validation passed!");
}
```

## Validation Levels

### Minimal — Quick Structural Check

**What it checks:**
- XML is well-formed
- Required elements present (`<model>`, `<resources>`, `<build>`)
- Resource IDs are valid integers
- ZIP archive is readable
- Basic OPC structure exists

**What it skips:**
- Reference integrity (broken links)
- Geometry validation
- Spec compliance details

**Use when:**
- CI/CD pipelines (fast feedback)
- Quick file format detection
- Pre-validation before deeper checks
- Processing untrusted files (fail fast)

**Performance:** <10ms for most files

**Example:**

```rust
let report = model.validate(ValidationLevel::Minimal)?;
```

### Standard — Reference Integrity (Recommended)

**What it checks:**
- Everything from Minimal level
- All resource references valid (objects, materials, textures)
- Build items reference existing objects
- Triangle vertex indices within bounds
- Property references valid
- Material/texture paths exist
- Component hierarchies valid

**What it skips:**
- Geometry quality checks (manifoldness, self-intersection)
- Strict spec compliance (ranges, optional attributes)
- Deep validation of extension data

**Use when:**
- Production file processing (default level)
- Pre-print validation
- General file integrity checks
- Most applications

**Performance:** ~100ms for 100K triangles

**Example:**

```rust
let report = model.validate(ValidationLevel::Standard)?;

// Check for specific severity levels
println!("Errors: {}", report.error_count());
println!("Warnings: {}", report.warning_count());
println!("Info: {}", report.info_count());
```

### Strict — Full Specification Compliance

**What it checks:**
- Everything from Standard level
- All attribute ranges per spec (coordinates, colors, UVs)
- Required vs. optional attribute presence
- Namespace declarations
- Extension version compatibility
- Metadata format compliance
- Unit validation (mm, inch, micron, etc.)

**What it skips:**
- Deep geometry checks (performance intensive)

**Use when:**
- Compliance certification
- 3MF Consortium validation
- Exporter testing
- Strict quality requirements

**Performance:** ~150ms for 100K triangles

**Example:**

```rust
let report = model.validate(ValidationLevel::Strict)?;

for issue in report.all_issues() {
    match issue.severity {
        Severity::Error => eprintln!("ERROR: {}", issue.message),
        Severity::Warning => println!("WARN: {}", issue.message),
        Severity::Info => println!("INFO: {}", issue.message),
    }
}
```

### Paranoid — Deep Geometry Analysis

**What it checks:**
- Everything from Strict level
- **Manifoldness** — Every edge has exactly 2 adjacent triangles
- **Vertex manifoldness** — No singular vertices or edge singularities
- **Self-intersection** — BVH-accelerated triangle-triangle intersection tests
- **Orientation consistency** — All face normals point consistently outward
- **Degenerate triangles** — Zero-area or duplicate-vertex triangles
- **Island detection** — Connected component analysis
- **Watertightness** — No holes or boundary edges

**Use when:**
- Pre-print validation (avoid print failures)
- Geometry repair workflows
- Quality assurance for manufacturing
- Debugging mesh issues

**Performance:** ~5 seconds for 100K triangles (depends on geometry complexity)

**Example:**

```rust
let report = model.validate(ValidationLevel::Paranoid)?;

if report.has_errors() {
    println!("Geometry issues found:");
    for error in report.errors() {
        println!("  - {}", error.message);
    }

    // Consider mesh repair
    use lib3mf_core::model::repair::MeshRepair;
    for (id, obj) in model.resources.iter_objects() {
        let repaired = obj.mesh.stitch_vertices(0.001)?;
        println!("Object {}: repaired", id.0);
    }
}
```

## Understanding Validation Reports

### Severity Levels

**Error** — File violates specification, may cause failures:
- Missing required resources
- Invalid references
- Out-of-bounds indices
- Geometry defects (at Paranoid level)

**Warning** — Unusual but technically valid:
- Unused resources (defined but not in build)
- Empty resource groups
- Non-optimal geometry

**Info** — Informational messages:
- Units used (millimeter, inch, etc.)
- Extensions detected
- Statistics (object count, triangle count)

### Error Codes

Validation errors use numeric codes for programmatic handling:

| Code Range | Category |
|------------|----------|
| 1000-1999 | XML/Structure errors |
| 2000-2999 | Resource errors |
| 3000-3999 | Build errors |
| 4000-4999 | Geometry errors |
| 5000-5999 | Extension errors |

**Example error codes:**
- `2001` — Resource ID not found
- `2042` — Triangle vertex index out of bounds
- `3010` — Build item references invalid object type
- `4015` — Non-manifold edge detected
- `4020` — Self-intersection found

### Accessing Report Data

```rust
use lib3mf_core::validation::{ValidationReport, Severity};

fn analyze_report(report: &ValidationReport) {
    // Check overall status
    if report.passed() {
        println!("Validation passed");
        return;
    }

    // Count by severity
    println!("Errors: {}", report.error_count());
    println!("Warnings: {}", report.warning_count());

    // Get specific severity issues
    for error in report.errors() {
        println!("ERROR [{}]: {}", error.code, error.message);
    }

    for warning in report.warnings() {
        println!("WARN: {}", warning.message);
    }

    // Access all issues
    for issue in report.all_issues() {
        println!("{:?}: {}", issue.severity, issue.message);
    }
}
```

## Geometry Validation Details

At the Paranoid level, lib3mf-rs performs sophisticated geometry analysis:

### Manifoldness Check

Every edge should have exactly 2 adjacent triangles (closed surface):

```rust
// Non-manifold edges are detected:
// - Boundary edges (only 1 triangle)
// - Over-connected edges (3+ triangles)
```

**Common causes:**
- Holes in geometry
- T-junctions
- Duplicate triangles

### Vertex Manifoldness

Vertices must have valid topological neighborhoods:

```rust
// Detects:
// - Singular vertices (vertex used by non-connected triangles)
// - Edge singularities (fan of triangles meeting at edge)
```

### Self-Intersection Detection

Uses BVH (Bounding Volume Hierarchy) for efficient O(n log n) intersection testing:

```rust
// Checks all triangle pairs for intersection
// Reports intersecting triangle IDs
```

**Performance:** BVH acceleration makes this practical even for large meshes.

### Orientation Consistency

All face normals should point outward consistently:

```rust
// Uses directed edge analysis
// Reports reversed triangles
```

**Fix:** Mesh repair can harmonize orientation automatically.

### Degenerate Triangle Detection

Finds zero-area or invalid triangles:

```rust
// Detects:
// - Duplicate vertices in triangle (v1 == v2)
// - Zero-area triangles (collinear vertices)
```

## Mesh Repair

When Paranoid validation finds issues, use the MeshRepair trait:

```rust
use lib3mf_core::model::repair::MeshRepair;

let mesh = obj.mesh;

// Stitch nearby vertices (merge within tolerance)
let stitched = mesh.stitch_vertices(0.001)?;

// Remove degenerate triangles
let cleaned = stitched.remove_degenerate_faces()?;

// Harmonize orientation (fix reversed normals)
let fixed = cleaned.harmonize_orientation()?;

println!("Repair stats:");
println!("  Vertices merged: {}", stitched.repair_stats.vertices_merged);
println!("  Triangles removed: {}", cleaned.repair_stats.triangles_removed);
println!("  Triangles flipped: {}", fixed.repair_stats.triangles_flipped);
```

Repair operations return new `Mesh` instances (immutable design).

## Common Validation Errors and Fixes

### Error: Object referenced in build but not defined

**Cause:** Build item references object ID that doesn't exist in resources.

**Fix:**
```rust
// Remove invalid build items
model.build.items.retain(|item| {
    model.resources.get_object(item.object_id).is_some()
});
```

### Error: Triangle vertex index out of bounds

**Cause:** Triangle references vertex index >= vertex count.

**Fix:** Mesh is corrupted, regenerate or repair geometry.

### Error: Non-manifold edge detected

**Cause:** Geometry has holes or inconsistent connectivity.

**Fix:**
```rust
let repaired = mesh.stitch_vertices(0.001)?;
```

### Warning: Object defined but never used

**Cause:** Resource exists but no build item references it.

**Fix:** Either add to build or remove resource (cosmetic issue).

## CLI Validation

Use the CLI tool for quick validation:

```bash
# Standard level (default)
lib3mf-cli validate model.3mf

# Paranoid level (deep checks)
lib3mf-cli validate model.3mf --level paranoid

# JSON output for scripting
lib3mf-cli validate model.3mf --format json > report.json
```

See the [CLI Guide](cli-guide.md) for details.

## Performance Tuning

**For large files:**
1. Start with Minimal to fail fast
2. Use Standard for most checks
3. Reserve Paranoid for final QA

**For CI/CD:**
- Minimal: <10ms (good for pre-commit hooks)
- Standard: ~100ms (good for PR checks)
- Paranoid: ~5s (good for release validation)

**Parallel validation:**

When built with the `parallel` feature, geometry checks use multi-threading automatically:

```bash
cargo build --features parallel
```

This can reduce Paranoid validation time by 2-4x on multi-core systems.

## Next Steps

- **[CLI Guide](cli-guide.md)** — Command-line validation tools
- **[Architecture](architecture.md)** — How validation system works internally
- **[API Reference](../rustdoc/lib3mf_core/validation/index.html)** — Detailed validation API docs
