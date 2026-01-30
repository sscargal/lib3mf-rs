# lib3mf-converters

This crate provides utilities for converting between 3MF and other 3D formats like STL and OBJ.

## Features
- **STL Import/Export**: Support for binary STL files.
- **OBJ Import/Export**: Support for Wavefront OBJ files (vertices and faces).
- **Model Integration**: Direct conversion to/from the `lib3mf-core::model::Model` structure.

## Examples

### STL Conversion
```rust
use lib3mf_converters::stl::{StlImporter, StlExporter};
// See examples/stl_conversion.rs for full usage.
```

### OBJ Conversion
```rust
use lib3mf_converters::obj::{ObjImporter, ObjExporter};
// See examples/obj_conversion.rs for full usage.
```
