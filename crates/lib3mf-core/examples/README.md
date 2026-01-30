# Examples

This directory contains code examples demonstrating how to use `lib3mf-rs`.

## Running Examples

You can run an example using `cargo run --example`:

```bash
# From the project root
cargo run -p lib3mf-core --example simple_read
```

## Available Examples

### Basic Usage
*   **`simple_read`**: Opens a 3MF file (defaults to `models/Benchy.3mf`), parses it, and prints basic statistics.
*   **`create_cube`**: Demonstrates building a 3D model, mesh, and build item from scratch and writing it to a 3MF file.
*   **`metadata_properties`**: Explains how to add standard (Dublin Core) and custom namespaced metadata to a model.

### Advanced Geometry & Extensions
*   **`components_transform`**: Shows how to use components and `glam::Mat4` transforms for efficient object instancing.
*   **`beam_lattice_ext`**: Demonstrates the Beam Lattice extension for creating lightweight structural designs.
*   **`slice_data`**: Shows how to define geometry using 2D slice stacks (DLP/SLA resin printing).

### Materials & Quality
*   **`advanced_materials`**: Comprehensive demonstration of Texture2DGroup, CompositeMaterials (mixing), and MultiProperties (layering).
*   **`geometry_repair`**: Shows how to use the repair engine for vertex stitching and removing degenerate geometry.
*   **`geometry_validation`**: Demonstrates "Paranoid" validation checks for manifoldness and face orientation.

### Specialized Tools
*   **`streaming_stats`**: Demonstrates the `ModelVisitor` and streaming parser for processing massive files with constant memory usage.
*   **`model_diff`**: Uses the utility API to perform a structural comparison between two 3MF models.
*   **`secure_content`**: Demonstrates signing a 3MF model and verifying existing digital signatures.
