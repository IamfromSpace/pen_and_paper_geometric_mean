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

2. **Integer averaging**: [35, 29, 27] → (35+29+27)/3 = 91/3 = 30 (truncate)

3. **Reverse conversion**: 30 → digit_count=3, fractional_index=0, MULTIPLIERS[0]=1.0, result=1000.0

**Compare**: Exact geometric mean ≈ 1281.7, our result = 1000.0

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
Replace the current floating point implementation with integer arithmetic:

1. **Replace TABLE_ENTRIES with MULTIPLIERS array**: Use direct array indexing instead of tuple lookups
2. **Modify find_forward_table_entry**: Use integer comparisons to find table index
3. **Update number_to_log_representation**: Work with scaled integers internally, return f64 for compatibility
4. **Replace find_reverse_table_entry**: Use direct array access with integer indices
5. **Update log_representation_to_number**: Accept f64 but work with scaled integers internally
6. **Modify table_based_approximation**: Use integer averaging for the core calculation

### Key Implementation Notes
- **Internal scaling**: Convert f64 log values to `i32` by multiplying by 10 (3.6 → 36)
- **Forward lookup**: Find largest index where `MULTIPLIERS[index] <= leading_digits`
- **Reverse lookup**: Direct array access using `scaled_value % 10` as index
- **Integer averaging**: Sum scaled integers, divide by count, truncate (maintains existing rounding behavior)
- **External compatibility**: All public functions still accept/return f64 values

## Testing and Validation

### Core Verification
- **Existing tests pass**: All current tests in `table_based.rs` should continue to pass
- **README examples**: Verify identical results to documented table method examples
- **Property-based tests**: Existing QuickCheck tests should maintain or improve precision
- **Precision improvement**: Integer method should eliminate floating point rounding errors
- **Edge cases**: Table boundaries, rounding decisions, large inputs

### Success Criteria
- [ ] All existing tests pass without modification
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