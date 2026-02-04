# Extensions

The 3MF specification is modular, with a core specification and optional extensions that add specialized capabilities. lib3mf-rs implements all 9 official extensions with 100% feature coverage.

## What are 3MF Extensions?

Extensions are additions to the core 3MF specification that enable advanced manufacturing scenarios. They use XML namespaces to add new elements and attributes without breaking compatibility.

A 3MF file can use multiple extensions simultaneously. For example, a file might combine Materials (colors/textures), Production (part tracking), and Slice (pre-sliced layers) extensions.

## Supported Extensions

| Extension | Version | Feature Coverage | Required lib3mf Feature |
|-----------|---------|------------------|------------------------|
| Core Specification | v1.4.0 | 98/100 (98%) | None (always available) |
| Materials and Properties | v1.2.1 | 38/38 (100%) | None |
| Production | v1.1.2 | 20/20 (100%) | None |
| Beam Lattice | v1.2.0 | 29/29 (100%) | None |
| Slice | v1.0.2 | 35/35 (100%) | None |
| Volumetric | v0.8.0 | 20/20 (100%) | None |
| Secure Content | v1.0.2 | 49/50 (98%) | `crypto` |
| Boolean Operations | v1.1.1 | 16/16 (100%) | None |
| Displacement | v1.0.0 | 31/31 (100%) | None |

**Total:** 336/345 features (97.4%)

## Extension Details

### Materials and Properties Extension (v1.2.1)

**Purpose:** Add color, texture, and advanced material properties to geometry.

**Features:**
- **Base Materials** — Solid colors (RGB/RGBA) with display names
- **Color Groups** — Per-vertex color gradients via color property groups
- **Texture 2D** — UV-mapped images with tile styles and filtering
- **Composite Materials** — Blend multiple materials with mixing ratios
- **Multi-Properties** — Combine multiple property types on single geometry

**Code example:**

```rust
use lib3mf_core::Model;

let model = Model::from_file("textured.3mf")?;

// Access textures
for (id, texture) in model.resources.iter_textures() {
    println!("Texture {}: {} ({}x{})",
        id.0,
        texture.path,
        texture.width,
        texture.height
    );
}

// Access base materials
for (id, material_group) in model.resources.iter_base_materials() {
    for (idx, material) in material_group.materials.iter().enumerate() {
        println!("Material {}/{}: {} - #{:06X}",
            id.0,
            idx,
            material.name,
            material.color & 0xFFFFFF  // RGB only
        );
    }
}
```

**Use cases:**
- Full-color 3D printing
- Texture mapping from CAD
- Multi-material printing
- Visual fidelity in manufacturing

**Specification:** [Materials Extension v1.2.1](https://3mf.io/specification/)

### Production Extension (v1.1.2)

**Purpose:** Track manufacturing metadata like UUIDs, part numbers, and production paths.

**Features:**
- **UUIDs** — Unique identifiers for each build item
- **Part Numbers** — Manufacturing part numbers
- **Production Path** — Hierarchical organization of parts

**Code example:**

```rust
for item in &model.build.items {
    if let Some(uuid) = &item.uuid {
        println!("Build item {} has UUID: {}", item.object_id.0, uuid);
    }
    if let Some(part_number) = &item.part_number {
        println!("  Part number: {}", part_number);
    }
}
```

**Use cases:**
- Manufacturing tracking
- Inventory management
- Production workflow integration
- Quality assurance

**Specification:** [Production Extension v1.1.2](https://3mf.io/specification/)

### Beam Lattice Extension (v1.2.0)

**Purpose:** Define structural lattice geometries as cylindrical beams connecting vertices.

**Features:**
- **Beam Sets** — Collections of beams with shared radius
- **Cap Modes** — Sphere, hemisphere, or butt caps at beam endpoints
- **Clipping Modes** — How beams interact at intersections
- **Precision Radii** — Specify exact beam thickness

**Code example:**

```rust
use lib3mf_core::model::Geometry;

for (id, obj) in model.resources.iter_objects() {
    if let Geometry::BeamLattice(lattice) = &obj.geometry {
        println!("Object {} is a beam lattice:", id.0);
        println!("  Min length: {}", lattice.min_length);
        println!("  Beam sets: {}", lattice.beam_sets.len());
        for beam_set in &lattice.beam_sets {
            println!("    Beams: {}, Radius: {}",
                beam_set.beams.len(),
                beam_set.radius
            );
        }
    }
}
```

**Use cases:**
- Structural optimization
- Lightweight mechanical parts
- Heat exchangers
- Medical implants (porous structures)

**Specification:** [Beam Lattice Extension v1.2.0](https://3mf.io/specification/)

### Slice Extension (v1.0.2)

**Purpose:** Pre-sliced 2D layer data for DLP/SLA resin printers.

**Features:**
- **Slice Stacks** — Ordered layers with Z-heights
- **2D Polygons** — Per-layer geometry as closed polygons
- **Multi-Material Slices** — Different materials per layer
- **External Slice Refs** — Reference external slice files

**Code example:**

```rust
use lib3mf_core::model::Geometry;

for (id, obj) in model.resources.iter_objects() {
    if let Some(slice_stack_id) = obj.slice_stack_id {
        println!("Object {} references slice stack {}", id.0, slice_stack_id.0);

        if let Some(stack) = model.resources.get_slice_stack(slice_stack_id) {
            println!("  Layers: {}", stack.slices.len());
        }
    }
}
```

**Use cases:**
- SLA/DLP resin printing
- Pre-processed slicing workflows
- Layer-based manufacturing
- Optimized print files

**Specification:** [Slice Extension v1.0.2](https://3mf.io/specification/)

### Volumetric Extension (v0.8.0)

**Purpose:** Voxel-based geometry representation for volume data.

**Features:**
- **Volumetric Stacks** — 3D voxel grids
- **Field-Based Volumes** — Continuous field representations
- **Image Stacks** — Layer-by-layer image data

**Code example:**

```rust
for (id, stack) in model.resources.iter_volumetric_stacks() {
    println!("Volumetric stack {}: {} layers", id.0, stack.layers.len());
}
```

**Use cases:**
- Medical imaging (CT/MRI to 3D print)
- Scientific visualization
- Voxel-based manufacturing
- Material density variation

**Specification:** [Volumetric Extension v0.8.0](https://3mf.io/specification/)

### Secure Content Extension (v1.0.2)

**Purpose:** Digital signatures and content encryption for secure manufacturing.

**Features:**
- **XML-DSIG** — Digital signatures for authenticity verification
- **XML-ENC** — AES-GCM content encryption
- **X.509 Certificates** — Certificate chain validation
- **RSA Public Key** — Asymmetric cryptography

**Requires:** `crypto` feature flag

**Code example:**

```rust
#[cfg(feature = "crypto")]
use lib3mf_core::crypto::verify_signature;

#[cfg(feature = "crypto")]
fn check_signature(model: &Model) -> Result<bool> {
    if let Some(signature) = &model.signature {
        verify_signature(signature)
    } else {
        Ok(false)  // No signature present
    }
}
```

**Use cases:**
- Intellectual property protection
- Manufacturing authenticity
- Regulatory compliance
- Trusted supply chains

**Specification:** [Secure Content Extension v1.0.2](https://3mf.io/specification/)

### Boolean Operations Extension (v1.1.1)

**Purpose:** Constructive solid geometry operations (union, difference, intersection).

**Features:**
- **Union** — Combine meshes
- **Difference** — Subtract mesh from another
- **Intersection** — Keep only overlapping volume
- **Transform Matrices** — Position operands
- **Nested Operations** — Boolean trees

**Code example:**

```rust
use lib3mf_core::model::Geometry;

for (id, obj) in model.resources.iter_objects() {
    if let Geometry::BooleanShape(boolean) = &obj.geometry {
        println!("Boolean operation: {:?}", boolean.operation);
        println!("  Operands: {}", boolean.operands.len());
    }
}
```

**Use cases:**
- CAD modeling workflows
- Complex geometry construction
- Parametric design
- Procedural generation

**Note:** lib3mf-rs parses and writes boolean operations but does not evaluate them (geometry processing is left to slicers/CAD software).

**Specification:** [Boolean Operations Extension v1.1.1](https://3mf.io/specification/)

### Displacement Extension (v1.0.0)

**Purpose:** Texture-driven surface modification for detailed surface features.

**Features:**
- **Displacement Maps** — Grayscale textures define surface offset
- **Normal Maps** — RGB textures for surface normal perturbation
- **Texture Coordinates** — UV mapping per vertex
- **Scale Factors** — Control displacement intensity

**Code example:**

```rust
use lib3mf_core::model::Geometry;

for (id, obj) in model.resources.iter_objects() {
    if let Geometry::DisplacementMesh(displacement) = &obj.geometry {
        println!("Displacement mesh {}:", id.0);
        println!("  Texture: {}", displacement.texture_id.0);
        println!("  Scale: {}", displacement.scale);
    }
}
```

**Use cases:**
- High-detail surface textures
- Realistic skin/terrain
- Efficient detail representation
- Artistic surface effects

**Note:** lib3mf-rs parses displacement data but does not evaluate/tessellate it (surface subdivision is left to renderers).

**Specification:** [Displacement Extension v1.0.0](https://3mf.io/specification/)

## Detecting Extensions in Files

You can check which extensions a file uses:

```rust
use lib3mf_core::Model;

let model = Model::from_file("model.3mf")?;
let stats = model.compute_stats()?;

println!("Extensions used:");
for extension in &stats.extensions {
    println!("  - {}", extension);
}
```

Extensions are detected automatically during parsing based on XML namespaces.

## Vendor Extensions

Beyond official 3MF Consortium extensions, lib3mf-rs supports some vendor-specific extensions:

**Bambu Studio Project Files:**
- Multi-plate layouts
- Filament metadata
- Print time estimates
- Machine settings

These are parsed but not officially documented by the 3MF Consortium.

## Next Steps

- **[Architecture](architecture.md)** — How extensions are integrated into lib3mf-rs
- **[API Reference](../rustdoc/lib3mf_core/model/index.html)** — Detailed extension data structures
- **[Examples](https://github.com/sscargal/lib3mf-rs/tree/main/crates/lib3mf-core/examples)** — Code examples for each extension
