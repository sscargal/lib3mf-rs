# Contributing to Lib3mf-rs

Thank you for your interest in contributing to `lib3mf-rs`! This guide will help you get started with building, testing, and understanding the project structure.

## Project Structure

This project is a Rust workspace containing several crates:

*   **`crates/lib3mf-core`**: The core library. Contains the logic for parsing, validating, and writing 3MF files.
*   **`crates/lib3mf-cli`**: The command-line interface tool.
*   **`crates/lib3mf-io`**: Format converters (STL, OBJ).
*   **`crates/lib3mf-wasm`**: WebAssembly bindings for checking and parsing 3MF files in the browser.
*   **`crates/lib3mf-async`**: (In Progress) Async I/O support.

## Development Setup

1.  **Install Rust**: Ensure you have the latest stable Rust toolchain installed.
    ```bash
    rustup update stable
    ```

2.  **Clone the Repository**:
    ```bash
    git clone https://github.com/yourusername/lib3mf-rs.git
    cd lib3mf-rs
    ```

## Building

To build all crates in the workspace:

```bash
cargo build
```

To build a specific crate (e.g., the CLI):

```bash
cargo build -p lib3mf-cli
```

## Testing

We use a combination of standard unit tests, integration tests, and property-based tests.

### Run All Tests
```bash
cargo test
```

### Run Property-Based Tests
We use `proptest` for robustness testing in `lib3mf-core`.
```bash
cargo test -p lib3mf-core --test proptests
```

### Benchmarking
We use `criterion` for performance benchmarking.
```bash
cargo bench -p lib3mf-core
```

### Fuzzing
We use `cargo-fuzz` to test the parser against malformed inputs.
```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run the model parser fuzz target
cargo fuzz run parse_model
```

## Code Quality

Please ensure your code is formatted and passes clippy checks before submitting a PR.

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings
```

## Architecture Notes

*   **Immutable by Default**: The `Model` struct is designed to be largely immutable after parsing for thread safety, though builders are provided for construction.
*   **Zero-Copy Intent**: Where possible, we aim to avoid copying mesh data.
*   **Error Handling**: We use `thiserror` for library errors and `anyhow` for the CLI.

## License

This project is licensed under [BSD 2-Clause](LICENSE).
