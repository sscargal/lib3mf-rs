# lib3mf-core

The core library for `lib3mf-rs`. This crate implements the logic for:

*   **Model Representation**: In-memory structs for `Model`, `Mesh`, `Components`, etc.
*   **Parsing**: XML parsing via `quick-xml` and ZIP handling.
*   **Validation**: 3MF specification compliance checks.
*   **Writing**: Serialization to valid 3MF archives.
*   **Extensions**: Support for production, beam lattice, slice, and secure content extensions.

## Usage

```rust
use lib3mf_core::model::Model;
use std::fs::File;

let file = File::open("model.3mf").unwrap();
// Use the appropriate parser (see `parser` module)
```
