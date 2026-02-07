# 3MF Conformance Testing

lib3mf-rs integrates the official 3MF Consortium test suite from [3MFConsortium/3mf-samples](https://github.com/3MFConsortium/3mf-samples) to validate specification compliance.

## Test Suite Overview

The official test suite is integrated as a git submodule at `tests/conformance/3mf-samples/` and includes:

- **MUSTPASS**: 13 test files containing valid 3MF files that should parse successfully
- **MUSTFAIL**: 38 test files containing invalid 3MF files that should fail or produce errors

## Results Summary

| Category | Tests | Passing | Pass Rate | Notes |
|----------|-------|---------|-----------|-------|
| MUSTPASS | 13 | 13 | 100% | All valid files parse successfully ✓ |
| MUSTFAIL | 38 | 15 | 39% | Parser too lenient on some validation rules |
| **Total** | **51** | **28** | **55%** | |

### MUSTPASS Tests: 13/13 Passing ✓

All 13 MUSTPASS tests successfully parse valid 3MF files and pass validation. These include:

✓ Chapter 2.1 - OPC parts and relationships
✓ Chapter 2.2 - Part naming conventions
✓ Chapter 2.3a - Ignorable markup handling
✓ Chapter 3.2b - Units of measurement (millimeters)
✓ Chapter 3.2c - Multiple items with transformations
✓ Chapter 3.4.1c - Undefined metadata names (ignored correctly)
✓ Chapter 3.4.2 - Metadata, resources, and build structure
✓ Chapter 3.4.3a - Non-referenced objects handling
✓ Chapter 4.1 - Explicit support objects
✓ Chapter 4.2 - Component references
✓ Chapter 5.1a - Composite and multi-properties materials
✓ Chapter 5.1b - Multi-object composite and multi-properties
✓ Chapter 5.1c - sRGB and RGB color materials

### MUSTFAIL Tests: 15/38 Passing

The parser correctly detects 15 invalid files but is too lenient on 23 others. Passing validations include:

#### Materials and Properties Extension (18 tests)
- Duplicated resource IDs (color groups, textures, composite materials, multi-properties)
- Missing required attributes (color values, texture IDs, material IDs, PIDs)
- Invalid references between material types
- Invalid content types and missing paths for textures

#### Core Specification (20 tests)
- OPC relationship validation (missing parts, external links, duplicate relationships)
- XML encoding and namespace validation
- Metadata validation (missing names, duplicates)
- Model structure validation (multiple models, missing build/resources)
- Object type validation (invalid types, PID requirements)
- Build item validation (non-existent object IDs, type constraints)

#### Passing MUSTFAIL Tests (15)
✓ Duplicated resource IDs detection (6 tests)
✓ Missing required attributes (5 tests)
✓ Non-existent object references (2 tests)
✓ Invalid object types in build (1 test)
✓ Missing model structure elements (1 test)

#### Failing MUSTFAIL Tests (23)
The parser currently does NOT detect these invalid scenarios:

**OPC/Archive Issues (4 tests)**:
- External references in relationships
- Multiple 3D model parts
- Non-existent thumbnail parts
- Multiple print tickets

**XML/Metadata Issues (5 tests)**:
- Non-UTF encoding
- Invalid data type definitions
- Undefined namespaces in XSD
- Whitespace in XML elements
- Missing/duplicated metadata names

**Model Structure Issues (3 tests)**:
- Multiple model elements
- Missing build element
- Invalid object types

**Material Reference Issues (8 tests)**:
- Multiple references to same material group
- Cross-references between material types
- Invalid matid references
- References to multi-properties from other resources

**Texture Issues (2 tests)**:
- Invalid content types
- Missing path attributes

**Other (1 test)**:
- PID specified without material reference

## Known Gaps

### Validation Gaps (23 MUSTFAIL Tests)

**Status**: Parser too lenient
**Severity**: Medium - affects invalid file detection
**Root Cause**: Missing validation checks in parser and validator

The parser successfully parses 23 files that should be rejected according to the 3MF specification. These represent missing validation rules across multiple areas:

**Impact**:
- Invalid files may be accepted and processed
- Does not affect parsing of valid files
- Primarily affects strict compliance validation

**Categories of Missing Validation**:
1. OPC/Archive validation (4 tests) - relationship constraints, part existence checks
2. XML/Encoding validation (5 tests) - encoding detection, namespace validation
3. Model structure validation (3 tests) - multiple model elements, required elements
4. Material reference validation (8 tests) - cross-reference rules, duplicate references
5. Texture validation (2 tests) - content type and path validation
6. Attribute validation (1 test) - PID without material reference

**Future Work**:
- Implement stricter OPC relationship validation
- Add XML encoding and namespace checks
- Enforce material reference constraints
- Add content type validation for textures
- Improve metadata validation (name requirements, duplicates)

These gaps represent opportunities for enhancement but do not prevent parsing of valid 3MF files.

## Test Categories Explained

### MUSTPASS Tests

These tests verify that valid 3MF files parse successfully and pass validation:

- **OPC Container**: Proper relationship handling, part naming
- **XML Structure**: Ignorable namespaces, proper encoding
- **Model Structure**: Units, transformations, metadata
- **Objects**: Components, support objects, type handling
- **Materials**: Base materials, color groups, textures, composite materials, multi-properties

### MUSTFAIL Tests

These tests verify that invalid 3MF files are properly rejected:

- **Structural Errors**: Missing required elements, duplicate resources
- **Reference Errors**: Invalid object IDs, cross-resource type references
- **Material Errors**: Missing attributes, invalid material combinations
- **OPC Errors**: Invalid relationships, missing parts, external links

## Running Conformance Tests

```bash
# Run all conformance tests
cargo test --test conformance

# Run only MUSTPASS tests
cargo test --test conformance test_mustpass

# Run only MUSTFAIL tests
cargo test --test conformance test_mustfail

# Run specific test
cargo test --test conformance test_mustpass_chapter4_2_components
```

## Continuous Integration

Conformance tests run on every PR via GitHub Actions. While test failures don't currently block merges, they are tracked as issues for resolution.

## Comparison with Competitor Implementation

The competitor implementation (telecos/lib3mf_rust) claims "2,200+ test cases" which includes:

1. **Official 3MF Consortium tests** (same 51 tests as lib3mf-rs)
2. **Custom unit tests** (majority of their test count)
3. **Property-based tests** (fuzzing-style tests)

Our conformance results:
- **lib3mf-rs**: 100% MUSTPASS conformance (13/13 valid files), 55% total (28/51 tests)
- **telecos/lib3mf_rust**: Not publicly documented (repo doesn't show test results)

### Key Differences

| Aspect | lib3mf-rs | telecos/lib3mf_rust |
|--------|-----------|-------------------|
| Official conformance tests | 51 tests (100% MUSTPASS, 55% total) | 51 tests (pass rate unknown) |
| Test transparency | Public results documented | Test results not published |
| Known gaps | Documented with root cause | Unknown |
| Test organization | Separate conformance suite | Mixed with unit tests |
| Continuous testing | CI runs on every PR | Unknown |

## Future Improvements

1. **Improve MUSTFAIL detection** - Priority: Medium
   - Add stricter validation for OPC relationships
   - Implement XML encoding and namespace checks
   - Enforce material reference constraints
   - Target: >80% MUSTFAIL detection

2. **Add extension-specific conformance tests**
   - Beam Lattice extension tests
   - Slice extension tests
   - Boolean operations tests (not in official suite yet)
   - Displacement extension tests (not in official suite yet)

3. **Performance benchmarking**
   - Compare parsing speed on conformance files
   - Document memory usage profiles
   - Compare with lib3mf (C++) implementation

4. **Automated regression testing**
   - Track conformance pass rate over time
   - Alert on any regression in test results
   - Publish conformance badge on README

## Contributing

If you discover conformance issues:

1. Open an issue with the failing test name
2. Include the error message and backtrace
3. If possible, provide a minimal reproduction case
4. Reference this conformance documentation

For parser bugs discovered through conformance testing:
1. Mark issue with `conformance` and `parser` labels
2. Reference the specific MUSTPASS test that fails
3. Prioritize based on impact (how many real-world files affected)
