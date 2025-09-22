# Table-Based Geometric Mean Approximation Implementation Plan

**Date**: 2025-09-21T20:45
**Goal**: Implement the table-based (10^(1/10)) pen-and-paper approximation method for geometric mean estimation

## Current State Analysis

### Existing Foundation
- Core exact geometric mean function implemented in `src/main.rs:20-34`
- Log-linear interpolation approximation implemented in `src/log_linear.rs`
- Comprehensive property-based test suite using QuickCheck
- Established error handling patterns with `GeometricMeanError` enum
- Project structure supports modular approximation methods

### Key Context from README
- Table method requires memorization of 10^(1/10) lookup values
- Uses "lossy table conversions" but provides more logarithmically accurate results
- Algorithm converts numbers to logarithmic representation, averages, then converts back
- Target domain: numbers ≥ 1 (pen-and-paper trivia context)

## Algorithm Overview: 10^(1/10) Table Method

### Core Lookup Table (from README:76-87)
```
| digits | starts with |
|--------|-------------|
| 0.0    | 1           |
| 0.1    | 1.25        |
| 0.2    | 1.6         |
| 0.3    | 2           |
| 0.4    | 2.5         |
| 0.5    | 3           |
| 0.6    | 4           |
| 0.7    | 5           |
| 0.8    | 6           |
| 0.9    | 8           |
```

### Algorithm Steps

#### Forward Conversion (Number → Log Representation)
1. **Count zeros**: Calculate non-decimal digits - 1 (whole part of log₁₀)
2. **Extract leading digits**: Determine what the number "starts with"
3. **Table lookup**: Find closest match in "starts with" column
4. **Construct log value**: Combine zero count + decimal from table
5. **Rounding rule**: Round down when between table entries
6. **No Feeling Brave Here**: While a human can optionally interpolate, this will be fully rules based and will not attempt this option.

#### Examples from README:
- 2,000 → 3 zeros + starts with 2 → 3.3
- 50 → 1 zero + starts with 5 → 1.7
- 1.25M → 6 zeros + starts with 1.25 → 6.1
- 350 → 2 zeros + starts with 3.5 (round down to 3) → 2.5
- 1,400 → 3 zeros + starts with 1.4 (round down to 1.25) → 3.1
- 11 → 1 zero + starts with 1.1 (round down to 1) → 1.0
- 9001 → 3 zeros + starts with 9 (round down to 8) → 3.9

#### Averaging Step
- Take arithmetic mean of all log representations
- Standard floating-point arithmetic

#### Reverse Conversion (Log Representation → Number)
1. **Extract whole part**: Number of zeros in result
2. **Extract fractional part**: Lookup in table for "starts with" value
3. **Table lookup**: Find closest match in "digits" column
4. **Construct number**: Apply "starts with" multiplier × 10^(zero count)
5. **Rounding rule**: Round up when between table entries (or interpolate)

#### Examples from README:
- 3.6 → 3 zeros + 0.6 lookup (4) → 4,000
- 2.8 → 2 zeros + 0.8 lookup (6) → 600
- 7.2 → 7 zeros + 0.2 lookup (1.6) → 16M
- 4.4 → 4 zeros + 0.4 lookup (2.5) → 25k
- 2.333 → 2 zeros + 0.333 (round up to 0.4) → 250
- 7.75 → 7 zeros + 0.75 (round up to 0.8) → 60M
- 4.167 → 4 zeros + 0.167 (round up to 0.2) → 16k

## Implementation Plan

### Module Structure
Create new module `src/table_based.rs` following established pattern:
- Maintains separation from existing exact and log-linear implementations
- Self-contained with own error types for modularity
- Follows same function signature pattern for future comparison work

### Core Data Structures

#### Lookup Table Representation
```rust
// Static lookup table for 10^(1/10) values
const TABLE_ENTRIES: [(f64, f64); 10] = [
    (0.0, 1.0),
    (0.1, 1.25),
    (0.2, 1.6),
    (0.3, 2.0),
    (0.4, 2.5),
    (0.5, 3.0),
    (0.6, 4.0),
    (0.7, 5.0),
    (0.8, 6.0),
    (0.9, 8.0),
];
```

### Required Functions

#### 1. Main Public Interface
```rust
pub fn table_based_approximation(values: &[f64]) -> Result<f64, GeometricMeanError>
```
- Same signature as existing methods for consistency
- Input validation: empty array, non-positive values, values < 1.0
- Orchestrates full algorithm: convert → average → convert back

#### 2. Forward Conversion Helper
```rust
fn number_to_log_representation(value: f64) -> f64
```
- Input: positive number ≥ 1.0 (e.g., 2847.0)
- Output: log representation (e.g., 3.45 for some table lookup)
- Algorithm:
  1. Calculate digit count using `value.log10().floor() as i32`
  2. Extract leading significant digits
  3. Find closest table match using binary search or linear scan
  4. Apply rounding down rule for between-values cases
  5. Combine digit count + table decimal value

#### 3. Reverse Conversion Helper
```rust
fn log_representation_to_number(log_value: f64) -> f64
```
- Input: log representation (e.g., 3.6)
- Output: reconstructed number (e.g., 4000.0)
- Algorithm:
  1. Extract whole part: `log_value.floor()`
  2. Extract fractional part: `log_value - log_value.floor()`
  3. Find closest table match for fractional part
  4. Apply rounding up rule for between-values cases
  5. Multiply table value by `10^(whole_part)`

#### 4. Table Lookup Utilities
```rust
fn find_forward_table_entry(leading_digits: f64) -> f64
fn find_reverse_table_entry(fractional_part: f64) -> f64
```
- Handle table lookups with appropriate rounding rules
- Forward: round down when between entries
- Reverse: round up when between entries
- Consider "brave interpolation" as future enhancement

### Error Handling Strategy

#### Error Types
```rust
#[derive(Debug, PartialEq)]
pub enum GeometricMeanError {
    EmptyInput,
    NonPositiveValue,
    ValueTooSmall,  // for values < 1.0
}
```

#### Validation Requirements
- Empty input array → `EmptyInput`
- Zero or negative values → `NonPositiveValue`
- Values < 1.0 → `ValueTooSmall` (out of scope for pen-and-paper)
- All validation before any computation begins

### Test Implementation Plan

#### Basic Functionality Tests
1. **README examples validation**:
   - 2,000 → 3.3, 50 → 1.7, etc. (forward conversion)
   - 3.6 → 4,000, 2.8 → 600, etc. (reverse conversion)

2. **Algorithm integration tests**:
   - Full round-trip conversion accuracy
   - Multiple values averaging correctly
   - Single value returns itself

3. **Edge cases from README**:
   - Numbers requiring rounding decisions
   - Very large numbers (millions)
   - Numbers close to table boundaries

#### Property-Based Testing Integration
Follow existing QuickCheck patterns but adapt for table method constraints:
1. **Round-trip property**: `table_to_log(log_to_table(x)) ≈ x` within table precision
2. **Order independence**: Result unaffected by input ordering
3. **Single value identity**: Single-element array returns original value
4. **Monotonicity**: Larger inputs produce larger geometric means
5. **Bounds checking**: Result between min and max input values
6. **Scaling property**: Geometric mean scales multiplicatively

### Success Criteria

#### Functional Requirements
- [ ] All README examples produce correct results
- [ ] Full test suite passes including property-based tests
- [ ] Error handling matches existing patterns
- [ ] Algorithm handles edge cases correctly (rounding, boundaries)

#### Code Quality Requirements
- [ ] Follows existing code style and patterns
- [ ] Clear documentation of algorithm steps
- [ ] Performance comparable to other approximation methods
- [ ] Modular design enables future comparisons

#### Mathematical Requirements
- [ ] Table lookups implement correct rounding rules
- [ ] Forward/reverse conversion maintains consistency
- [ ] Approximation quality suitable for pen-and-paper use case

### Future Integration Points

This implementation provides foundation for:
1. **Accuracy comparison studies**: Statistical analysis vs. exact geometric mean
2. **Method comparison**: Table vs. log-linear approximation trade-offs
3. **Monte Carlo simulation**: Error distribution analysis across input ranges
4. **Formal verification**: Error bounds proof using LEAN (as mentioned in README)
5. **User interface**: Command-line tool for trivia team usage

### Performance Expectations

#### Expected Complexity
- Forward conversion: O(1) table lookup + O(1) arithmetic
- Reverse conversion: O(1) table lookup + O(1) arithmetic
- Overall algorithm: O(n) where n = number of input values
- Memory usage: O(1) constant table storage

## References and Dependencies

### Existing Code Dependencies
- Error handling patterns from `src/main.rs:3-18`
- Test patterns from existing QuickCheck integration
- Module structure following `src/log_linear.rs` precedent

### Mathematical Background
- Algorithm specification from README lines 72-100
- Table values represent 10^(x/10) for x ∈ {0, 1, 2, ..., 9}
- Rounding rules optimized for pen-and-paper usage constraints

### External Dependencies
- QuickCheck for property-based testing (already integrated)
- Standard library math functions (`log10`, `floor`, `powi`)
- No additional external crates required
