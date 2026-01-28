---
name: 3mf-spec-expert
description: Domain knowledge about the 3MF specification, extension details, and best practices.
---

# 3MF Specification Expert

This skill provides expert knowledge on the 3D Manufacturing Format (3MF).

## Core Concepts

### Model Structure
- Root element is `<model>` with specific namespaces.
- Uses Open Packaging Conventions (OPC) via ZIP.
- **Resources**: Contains all mesh, material, and property definitions.
- **Build**: Contains `item` elements referencing resources to be printed.

### Unit Management
- 3MF is unit-aware.
- `unit` attribute on `<model>`: micron, millimeter (default), centimeter, inch, foot, meter.
- **Rule**: Always normalize to internal units if necessary, or strictly respect the file usage.

### Coordinate System
- Right-handed coordinate system.
- Z-up.

## Extensions
When implementing extensions, always check:
1. **Namespace Declaration**: must be in `<model>`.
2. **Ignorable Attributes**: Does valid `xmlns` require `mustunderstand`?

### Common Extensions
- **Material Extension**: Multi-color, PBR, composite materials.
- **Production Extension**: Splitting large builds into multiple files/packages.
- **Beam Lattice**: Lattice structures for lightweighting.
- **Slice**: Polygon-based stack definition.

## Implementation Traps
- **XML Namespaces**: Parsers often fail if they ignore namespaces. 3MF relies heavily on them.
- **Vertex Indexing**: 3MF uses 0-based indexing for triangles referencing vertices.
- **Locality**: Resources must be defined *before* they are used in the `<build>` section (or referenced by other resources).
- **FLOAT Precision**: Standard requires ability to handle high precision.

## Validation Checklist
- [ ] Is the ZIP header valid?
- [ ] Does `[Content_Types].xml` exist and list all parts?
- [ ] Are all relationships in `_rels/.rels` standard?
- [ ] Is the model file named correctly (typically `3D/3dmodel.model` but not strictly required by OPC)?

## Best Practices
- **Streaming**: For large files, do not load the whole DOM. Use streaming parsers.
- **Validation**: Validate schema *and* semantic rules (e.g., manifoldness).
- **Error Messages**: Provide line numbers and specific element names in specific errors.
