# Async Loading Example

This example demonstrates how to use the `lib3mf-async` crate to load a 3MF file asynchronously using `tokio`.

## Running the Example

Run the example from the project root:

```bash
cargo run -p lib3mf-async --example async_load
```

## Description

The example uses `load_model_async` to:
1. Open the `.3mf` file asynchronously using `tokio::fs::File`.
2. Parse the ZIP archive structure.
3. Locate and parse the main model XML part.
4. Report basic model metadata.
