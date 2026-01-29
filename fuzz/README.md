# lib3mf-fuzz

Fuzzing targets for `lib3mf-rs`.

Uses `cargo-fuzz` to test the robustness of the parser against malformed or malicious inputs.

## Running

```bash
cargo fuzz run parse_model
```
