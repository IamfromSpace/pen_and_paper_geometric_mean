# Integer Arithmetic Table-Based Geometric Mean Approximation Plan

**Date**: 2025-10-14T12:00:00
**Goal**: Eliminate floating point error from table-based approximation by using scaled integer arithmetic

## Problem Analysis

### Current Floating Point Error Sources

The existing table-based implementation in `src/table_based.rs` suffers from accumulated floating point precision errors:

1. **Log calculation precision**: `value.log10().floor()` introduces floating point error
2. **Division precision**: `value / 10.0_f64.powi(zeros)` compounds error during leading digit extraction
3. **Table lookup precision**: Fractional scaling in `find_reverse_table_entry` introduces rounding error
4. **Averaging precision**: Summing and dividing floating point values accumulates error across multiple inputs

### Key Insight: Single Digit Precision Sufficiency

Since the table-based method only provides single-digit precision anyway (0.1 increments in the logarithmic scale), we can eliminate floating point error entirely by:
- Operating on scaled integers instead of floating point decimals
- Using integer arithmetic throughout the core algorithm
- Only converting back to floating point at the final step

## Integer-Based Algorithm Design

### Core Concept
Scale logarithmic representations by 10 to work with integers instead of floating point decimals:
- `3.6` becomes `36` (eliminates decimal arithmetic precision issues)
- Use simple array indexing instead of tuple lookups
- Compatible with existing rounding behavior (forward rounds down, reverse rounds up)

### Complete Algorithm with Example
**Input**: [3600, 920, 740]

1. **Forward conversion**: Number → scaled integer log
   - 3600: digit_count=3, leading=3.6, table_lookup(3.6)→index_5, result=35
   - 920: digit_count=2, leading=9.2, table_lookup(9.2)→index_9, result=29
   - 740: digit_count=2, leading=7.4, table_lookup(7.4)→index_7, result=27

2. **Integer averaging**: [35, 29, 27] → (35+29+27)/3 = 91/3 = 31 (ceiling)

3. **Reverse conversion**: 31 → digit_count=3, fractional_index=1, MULTIPLIERS[0]=1.0, result=1250.0

**Compare**: Exact geometric mean ≈ 1281.7, our result = 1250.0

### Table Representation
```rust
const MULTIPLIERS: [f64; 10] = [
    1.0, 1.25, 1.6, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0
];
```
Array index represents decimal part (scaled by 10). Forward lookup finds largest index where `MULTIPLIERS[index] <= leading_digits`. Reverse lookup is direct array access.

## Implementation Plan

### Module Structure
Modify the existing `src/table_based.rs` module to use integer arithmetic internally while maintaining the same external interface. The public API (`TableBasedApproximation` struct and `table_based_approximation` function) remains unchanged.

### Internal Changes
Replace the current floating point implementation with integer arithmetic throughout the pipeline:

1. **Replace TABLE_ENTRIES with MULTIPLIERS array**: Use direct array indexing instead of tuple lookups
   ```rust
   const MULTIPLIERS: [f64; 10] = [1.0, 1.25, 1.6, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0];
   ```

2. **Modify find_forward_table_entry**: Return table index instead of fractional value
   ```rust
   fn find_forward_table_entry(leading_digits: f64) -> usize
   ```

3. **Update number_to_log_representation**: Return scaled integer instead of f64
   ```rust
   fn number_to_log_representation(value: f64) -> i32  // was -> f64
   ```
   - Internally: `zeros * 10 + table_index` to create scaled integer
   - Example: 3600 → digit_count=3, table_index=5 → result=35

4. **Eliminate find_reverse_table_entry**: Replace with direct array access pattern
   - Use `scaled_value % 10` as index into MULTIPLIERS array
   - Use `scaled_value / 10` for digit count

5. **Update log_representation_to_number**: Accept scaled integer instead of f64
   ```rust
   fn log_representation_to_number(scaled_log: i32) -> f64  // was scaled_log: f64
   ```

6. **Modify table_based_approximation**: Use pure integer arithmetic for averaging
   - Convert inputs: `Vec<f64>` → `Vec<i32>` via `number_to_log_representation`
   - Integer averaging: `(sum + count - 1) / count` ceiling given truncation
   - Final conversion: single call to `log_representation_to_number`

### Key Implementation Notes
- **Complete integer pipeline**: No floating point arithmetic between input validation and final result
- **Scaling factor**: All logarithmic values scaled by 10 (3.6 → 36) to work with integers
- **Forward lookup**: Find largest index where `MULTIPLIERS[index] <= leading_digits`
- **Reverse lookup**: Direct array access using `scaled_value % 10` as index
- **Integer averaging to ceiling**: `(sum_of_scaled_values + count - 1) / count` to compensate for truncation
- **Single floating point conversion**: Only at the very end in `log_representation_to_number`

## Testing and Validation

### Test Updates Required
Due to signature changes in internal functions, several tests need updates:

1. **`test_forward_conversion_readme_examples`**:
   - Update assertions: `3.3` → `33`, `1.7` → `17`, etc.
   - All expected results need to be scaled by 10

2. **`test_reverse_conversion_readme_examples`**:
   - Update inputs: `3.6` → `36`, `2.8` → `28`, etc.
   - All test inputs need to be scaled by 10
   - **CRITICAL**: Must account for original ceiling rounding behavior (e.g., `2.333` → `24`, not `23`)

3. **`test_round_trip_conversion`**:
   - Update to work with new `i32` return type from `number_to_log_representation`
   - Pass scaled integer to `log_representation_to_number`

4. **Property test `prop_round_trip_within_tolerance`**:
   - Update to work with new signatures

### Core Verification
- **Public API unchanged**: `table_based_approximation` function signature remains identical
- **README examples**: Verify identical final results for documented table method examples
- **Property-based tests**: All QuickCheck tests should continue to pass with improved precision
- **Precision improvement**: Integer method should eliminate floating point rounding errors
- **Edge cases**: Table boundaries, rounding decisions, large inputs

### Success Criteria
- [ ] Updated tests pass with new internal signatures
- [ ] Public API tests continue to pass without modification
- [ ] Eliminates floating point precision errors while maintaining table accuracy
- [ ] Deterministic, reproducible results across platforms
- [ ] Performance comparable to existing implementation
- [ ] Order independence property test should now pass consistently

## Mathematical Properties

### Precision Benefits
- **Exact averaging**: Integer arithmetic eliminates floating point averaging errors
- **Deterministic table lookups**: Integer comparisons eliminate floating point comparison errors
- **Cross-platform consistency**: Same inputs always produce identical outputs
- **Bounded error**: Error limited to table quantization (same as floating point method, but more precise)

### Future Applications
- **Formal verification**: Integer arithmetic easier to prove than floating point
- **Embedded systems**: Integer-only calculation for resource-constrained environments
- **Cross-platform consistency**: Eliminate floating point behavior variations
- **Educational clarity**: Algorithm demonstration without floating point complexity