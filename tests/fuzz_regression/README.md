# Fuzz Regression Tests

This directory contains crash inputs that were found by fuzzing and subsequently fixed.
Each file serves as a regression test to prevent the bug from being reintroduced.

## File Naming Convention

```
{target}_{issue-number}_{description}.bin
```

Example: `parse_model_42_oob_triangle_index.bin`

## Adding a Regression Test

1. When a fuzzing crash is fixed, copy the minimized crash input here
2. Name it according to the convention above
3. The test suite automatically picks up `.bin` files for regression testing

## Running Regression Tests

Regression tests are run as part of the normal test suite:

```bash
cargo test --test fuzz_regression
```

Or run fuzzing targets against saved crashes:

```bash
cargo +nightly fuzz run parse_model tests/fuzz_regression/parse_model_*.bin
```
