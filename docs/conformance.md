# 3MF Conformance Testing

lib3mf-rs integrates the official 3MF Consortium test suite from [3MFConsortium/3mf-samples](https://github.com/3MFConsortium/3mf-samples) to validate specification compliance.

## Test Suite Overview

The official test suite is integrated as a git submodule at `tests/conformance/3mf-samples/` and includes:

- **MUSTPASS**: 13 test files containing valid 3MF files that should parse successfully
- **MUSTFAIL**: 38 test files containing invalid 3MF files that should fail or produce errors

## Results Summary

| Category | Tests | Passing | Pass Rate | Notes |
|----------|-------|---------|-----------|-------|
| MUSTPASS | 13 | 6 | 46% | 7 failing due to parser bug (see Known Gaps) |
| MUSTFAIL | 38 | 38 | 100% | All invalid files correctly detected |
| **Total** | **51** | **44** | **86%** | |

### MUSTFAIL Tests: 100% Pass Rate ✓

All 38 MUSTFAIL tests correctly detect invalid files. These tests verify:

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

### MUSTPASS Tests: 6/13 Passing

#### Passing Tests (6)
✓ Chapter 2.3a - Ignorable markup handling
✓ Chapter 3.2b - Units of measurement (millimeters)
✓ Chapter 3.2c - Multiple items with transformations
✓ Chapter 4.1 - Explicit support objects
✓ Chapter 4.2 - Component references
✓ Chapter 5.1c - sRGB and RGB color materials

#### Failing Tests (7)
The following MUSTPASS tests fail with parser errors. This represents a known bug in the material parser:

1. **Chapter2.1_PartsRelationships.3mf** - `Unexpected EOF in texture2dgroup`
2. **Chapter2.2_PartNaming.3mf** - `Unexpected EOF in texture2dgroup`
3. **Chapter3.4.1c_MustIgnoreUndefinedMetadataName.3mf** - `Unexpected EOF in texture2dgroup`
4. **Chapter3.4.2_MetaData_Resources_Build.3mf** - `Unexpected EOF in colorgroup`
5. **Chapter3.4.3a_MustNotOutputNonReferencedObjects.3mf** - `Unexpected EOF in colorgroup`
6. **Chapter5.1a_MaterialResources_CompositeAndMultiProperties.3mf** - `Unexpected EOF in colorgroup`
7. **Chapter5.1b_MaterialResources_MultiObjects_CompositeAndMultiProperties.3mf** - `Unexpected EOF in colorgroup`

## Known Gaps

### Issue: Material Parser EOF Handling

**Status**: Known bug (discovered via conformance testing)
**Severity**: High - blocks 54% of MUSTPASS tests
**Root Cause**: Material parsers (colorgroup, texture2dgroup) incorrectly handle end-of-element events

The parser expects additional content when encountering the closing tag for material resource elements, resulting in "Unexpected EOF" errors for valid files.

**Affected Components**:
- `material_parser.rs`: `parse_colorgroup()` function
- `material_parser.rs`: `parse_texture2dgroup()` function

**Expected Behavior**: Parser should accept empty or self-closing material group elements
**Actual Behavior**: Parser fails with "Unexpected EOF" error

**Impact**:
- 7/13 (54%) of official MUSTPASS tests fail
- Files with certain material configurations cannot be parsed
- Affects real-world 3MF files using colorgroup or texture2dgroup resources

**Workaround**: None - this is a parser bug that must be fixed

**Future Work**:
- Fix EOF handling in material parsers
- Add unit tests specifically for empty/minimal material groups
- Re-run conformance tests to verify fix

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
- **lib3mf-rs**: 86% official conformance (44/51 tests passing)
- **telecos/lib3mf_rust**: Not publicly documented (repo doesn't show test results)

### Key Differences

| Aspect | lib3mf-rs | telecos/lib3mf_rust |
|--------|-----------|-------------------|
| Official conformance tests | 51 tests (86% pass) | 51 tests (pass rate unknown) |
| Test transparency | Public results documented | Test results not published |
| Known gaps | Documented with root cause | Unknown |
| Test organization | Separate conformance suite | Mixed with unit tests |
| Continuous testing | CI runs on every PR | Unknown |

## Future Improvements

1. **Fix material parser EOF bug** - Priority: High
   - Resolve 7 failing MUSTPASS tests
   - Target: 100% MUSTPASS conformance

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
