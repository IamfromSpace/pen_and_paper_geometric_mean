# Kani Integration for Safety Verification

## Objective

Integrate Kani formal verification to complement existing QuickCheck property tests by exhaustively checking for panics, arithmetic overflows, and undefined behavior in the geometric mean calculation functions.

## Verification Scope

Focus exclusively on safety properties rather than mathematical correctness:
- **Panics**: Division by zero, unwrap() failures, array bounds violations
- **Arithmetic overflows**: Integer overflow in digit counting, power operations
- **Undefined behavior**: Invalid floating-point operations (NaN, infinity propagation)

## Functions to Verify

### Primary Targets
1. `geometric_mean()` - Core exact calculation using logarithms
2. `log_linear_approximation()` - Pen-and-paper method with linear interpolation
3. `table_based_approximation()` - Pen-and-paper method using lookup tables

### Supporting Functions
4. `convert_to_log_linear()` / `convert_from_log_linear()`
5. `number_to_log_representation()` / `log_representation_to_number()`
6. Table lookup functions in table_based module

## Implementation Plan

### Phase 1: Setup and Basic Infrastructure
- Add Kani as development dependency in Cargo.toml
- Create basic proof harnesses for each function using `kani::any()`
- Establish input constraints based on function preconditions
- Verify Kani can analyze the codebase without compilation issues

### Phase 2: Core Safety Verification
- **Exact geometric mean verification**:
  - Verify no panics with positive finite inputs
  - Check logarithm domain constraints (`x > 0`)
  - Verify exponential doesn't produce invalid results
  - Test edge cases: very small values, very large values

- **Log-linear approximation verification**:
  - Verify digit counting operations don't overflow
  - Check power operations (`10.0_f64.powi()`) for valid ranges
  - Verify fractional part calculations stay in bounds
  - Test edge case handling (fractional_part < 0.1)

- **Table-based approximation verification**:
  - Verify array bounds in table lookups
  - Check integer conversions don't overflow
  - Verify rounding operations stay within table indices

### Phase 3: Edge Case Discovery
- Use Kani's exhaustive search to find input combinations that cause:
  - Integer overflow in digit counting logic
  - Floating-point edge cases (subnormal numbers, near-infinity values)
  - Boundary conditions in table lookups
  - Division operations that might approach zero

### Phase 4: Input Domain Refinement
- Document any input constraints discovered by Kani
- Add appropriate input validation or restrict function domains
- Update error handling to prevent panics where possible
- Consider whether certain edge cases should return errors vs panicking

## Proof Harness Strategy

### Input Generation Approach
```rust
#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn verify_geometric_mean_no_panic() {
        let len: usize = kani::any_where(|&x| x > 0 && x <= 10);
        let values: Vec<f64> = (0..len)
            .map(|_| kani::any_where(|&x| x > 0.0 && x.is_finite()))
            .collect();

        // Should not panic for positive finite inputs
        let _ = geometric_mean(&values);
    }
}
```

### Constraint Strategy
- Limit array sizes to reasonable bounds (≤ 100 elements) for verification tractability
- Focus on mathematically valid inputs first, then gradually expand
- Use `kani::assume()` to establish preconditions
- Test boundary conditions systematically

## Integration with Existing Tests

### Complementary Approach
- Keep existing QuickCheck tests for mathematical property verification
- Use Kani for exhaustive safety checking within bounded domains
- Kani findings should inform QuickCheck test improvements

### CI Integration
- Add Kani verification to build pipeline
- Consider separate CI stage due to potentially longer verification times
- Document verification coverage and limitations

## Expected Outcomes

### Safety Guarantees
- Proof that functions won't panic for specified input domains
- Identification of precise input constraints for safe operation
- Documentation of any unavoidable edge cases

### Code Quality Improvements
- More robust error handling
- Clearer function preconditions
- Better input validation
- Improved documentation of edge cases

## Implementation Considerations

### Performance and Scalability
- Start with small input domains and expand based on verification time
- May need to bound floating-point ranges for tractable verification
- Consider separate harnesses for different input magnitude ranges

### Floating-Point Challenges
- Kani may have limitations with transcendental functions (`ln`, `exp`)
- Focus on operations that can be fully verified (arithmetic, comparisons)
- Document any floating-point operations that can't be fully verified

### Documentation and Maintenance
- Document verification assumptions and limitations
- Maintain proof harnesses alongside function changes
- Include verification status in function documentation

## Success Criteria

1. **No Panics**: All functions proven panic-free for valid input domains
2. **Overflow Detection**: Any integer overflow conditions identified and handled
3. **Edge Case Documentation**: Complete catalog of input constraints and edge cases
4. **CI Integration**: Verification running automatically on code changes
5. **Developer Confidence**: Clear understanding of when functions are safe to call

This approach leverages Kani's strengths in safety verification while acknowledging that mathematical correctness is already well-covered by the existing QuickCheck property tests.

## Resources

### Essential Documentation
- **[Kani Getting Started Guide](https://model-checking.github.io/kani/)** - Installation, basic setup, and first steps
- **[Kani Tutorial](https://model-checking.github.io/kani/kani-tutorial.html)** - Comprehensive tutorial covering proof harnesses and verification techniques
- **[Kani GitHub Repository](https://github.com/model-checking/kani)** - Source code, examples, and issue tracking
