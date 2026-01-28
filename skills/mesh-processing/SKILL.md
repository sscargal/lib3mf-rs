---
name: mesh-processing
description: 3D geometry algorithms, manifold checks, and mesh optimization.
---

# Mesh Processing Skill

Instructions for handling 3D mesh data within lib3mf-rs.

## Core Data Structures

### Half-Edge Data Structure
- **Recommended** for complex editing/validation.
- Allows efficient traversal (vertex -> edge -> face).
- Essential for "watertight" checks.

### Simple indexed mesh
- **Recommended** for simple storage/IO (Read/Write).
- `Vec<Vertex>` + `Vec<Triangle>` (indices).

## Validation Algorithms

### Manifold Check
A mesh is manifold if:
1. Every edge is shared by exactly two triangles (closed mesh) or one triangle (boundary).
2. The set of triangles sharing a vertex forms a complete cycle (no "bowtie" vertices).

### Watertightness
- All edges must share exactly 2 faces.
- Euler characteristic check: $V - E + F = 2$ (for genus 0).

### Degenerate Triangles
- Area near zero.
- Duplicate vertices ($v1=v2$).

## Optimization

### Spatial Indexing
- Use **R-Trees** or **BVH** (Bounding Volume Hierarchies) for collision/intersection tests.
- Crate recommendation: `parry3d` or `bvh`.

### Normal Computation
- **Face Normals**: Cross product of two edges. Normalize.
- **Vertex Normals**: Weighted average of incident face normals (weighted by angle or area).

## 3MF Specifics
- **Triangle Winding Order**: Counter-clockwise (CCW) defines the "front" face (normal points out).
- **Indices**: 0-based.
- **Precision**: 3MF vertices are `f32` (typically), but specs allow high precision. Rust `f32` is usually sufficient, but `f64` might be needed for internal calculation to avoid drift.
