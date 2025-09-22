# QuickCheck Property-Based Testing Implementation Plan

**Date**: 2025-09-21T15:39:58
**Goal**: Introduce comprehensive property-based testing using QuickCheck to validate geometric mean algorithms and approximation methods

## Current State
- Exact geometric mean implementation in `src/main.rs:20-34` with comprehensive example-based tests
- Log-linear interpolation approximation method in `src/log_linear.rs:46-68` with targeted test cases
- Strong foundation of example-based testing covering edge cases and specific scenarios
- All functions return `Result<f64, GeometricMeanError>` with well-defined error conditions

## Motivation for Property-Based Testing

Property-based testing will significantly enhance our test coverage by:

1. **Discovering edge cases**: QuickCheck generates thousands of random inputs, finding cases we might not think of
2. **Validating mathematical properties**: Geometric mean has inherent mathematical properties that should hold for any valid input
3. **Approximation quality assurance**: The log-linear method should maintain predictable relationships to the true geometric mean
4. **Regression prevention**: Properties act as invariants that must hold across code changes
5. **Documentation through properties**: Properties serve as executable specifications of expected behavior

## Implementation Plan

### Phase 1: Add QuickCheck Dependency

Add QuickCheck and QuickCheck macros as development dependencies to enable property-based testing.

### Phase 2: Properties for Exact Geometric Mean (`src/main.rs`)

#### Mathematical Properties to Test

1. **Single Value Identity**: `geometric_mean([x]) = x` for any positive `x`
2. **Multiplicative Property**: `geometric_mean([a*k, b*k, c*k]) = k * geometric_mean([a, b, c])` for positive scaling factor `k`
3. **Order Independence**: `geometric_mean([a, b, c]) = geometric_mean([c, a, b])` (permutation invariance)
4. **Subset Bounds**: For sorted input `[a₁, a₂, ..., aₙ]`, result should be `a₁ ≤ geometric_mean ≤ aₙ`
5. **Arithmetic-Geometric Mean Inequality**: `geometric_mean([a, b, c]) ≤ arithmetic_mean([a, b, c])`
6. **Monotonicity**: If all elements of array `A` are `≤` corresponding elements of array `B`, then `geometric_mean(A) ≤ geometric_mean(B)`
7. **Duplicates Don't Change Result**: `geometric_mean([x]) = geometric_mean([x, x, x, x])`
8. **Two-Value Formula**: `geometric_mean([a, b]) = √(a × b)` for any positive `a, b`

#### Implementation Strategy

Create new property test module in `src/main.rs` with a custom generator for positive floating point numbers.
The generator should produce values suitable for geometric mean calculation (positive, avoiding edge cases like very small numbers that could cause precision issues).

### Phase 3: Properties for Log-Linear Approximation (`src/log_linear.rs`)

#### Approximation Quality Properties

1. **Order of Magnitude Correctness**: Result should be within one order of magnitude of true geometric mean
   - `true_result / 10.0 ≤ approximation_result ≤ true_result * 10.0`
2. **Same Digit Count Equivalence**: When all inputs have same digit count, result should equal arithmetic mean
3. **Single Value Identity**: `log_linear_approximation([x]) = x` for any `x ≥ 1`
4. **Order Independence**: Permutation invariance like the exact method
5. **Monotonicity**: If all elements of array `A` are `≤` corresponding elements of array `B`, then `log_linear_approximation(A) ≤ log_linear_approximation(B)`
6. **Scaling Behavior**: For inputs scaled by powers of 10, approximation should scale predictably
7. **Minimum Result Bounds**: Result should never be less than the minimum input value
8. **Maximum Result Bounds**: Result should never exceed the maximum input value (for reasonable inputs)

#### Value Domain Properties

1. **Input Validation**: Function should reject values `< 1.0` with `ValueTooSmall` error
2. **Edge Case Handling**: Small fractional parts should be normalized to 0.1 minimum
3. **Large Number Stability**: Should handle inputs up to `f64::MAX` without panicking

### Phase 4: Cross-Method Comparison Properties

#### Accuracy Relationship Properties

1. **Convergence for Same Digit Counts**: When all inputs have same digit count, both methods should produce identical results
2. **Order of Magnitude Agreement**: Both methods should always produce results within one order of magnitude of each other
3. **Relative Error Bounds**: For "reasonable" inputs (within 2-3 orders of magnitude of each other), approximation error should be bounded

### Phase 6: Implementation Details

#### Test Organization

Create dedicated property test modules:
- Property tests for exact geometric mean in `src/main.rs`
- Property tests for log-linear approximation in `src/log_linear.rs`
- Cross-method comparison properties can be included in the log-linear module

#### Custom Generators

1. **PositiveF64**: Generates positive floating point numbers for exact geometric mean (1e-100 to 1e100)
2. **GeOneF64**: Generates numbers ≥ 1.0 for log-linear approximation (1.0 to 1e50)
3. **SameDigitCount**: Generates arrays where all numbers have the same digit count

#### Error Handling Strategy

Properties should handle edge cases appropriately:
- Invalid inputs (negative numbers, zeros)
- Floating point edge cases (infinity, NaN)
- Values outside intended domain
- Cases where comparison doesn't make mathematical sense

#### Performance Considerations

- Use QuickCheck macros for simple properties where possible
- Use manual QuickCheck calls for complex properties requiring custom generators
- Set reasonable test count limits for CI performance
- Handle edge cases that don't contribute meaningful test coverage

### Phase 7: Integration and Documentation

#### Test Execution Integration

1. Ensure `cargo test` runs both example-based and property-based tests
2. Add documentation comments explaining what each property validates
3. Include examples in property documentation showing expected behavior

#### Continuous Integration Considerations

1. Property tests should be deterministic (use fixed seeds for CI)
2. Consider separate test commands for quick vs. thorough property testing
3. Document expected test runtime (property tests are typically slower)

#### Documentation Updates

1. Document the mathematical properties being validated in code comments
2. Explain how properties complement example-based tests in plan documentation

### Success Criteria

1. **All properties pass**: Every mathematical property holds for generated inputs
2. **Edge case discovery**: Property tests reveal at least one case not covered by example tests
3. **Performance acceptable**: Full property test suite completes in under 30 seconds
4. **Clear failure messages**: When properties fail, error messages clearly indicate what property was violated
5. **Maintainable code**: Properties are well-documented and easy to understand
6. **Regression protection**: Properties catch regressions that example tests might miss

### Future Extensions

This foundation enables future property-based testing for:

1. **10^(1/10) table method**: Once implemented, same pattern applies
2. **Monte Carlo simulation validation**: Properties can validate statistical properties of simulation results
3. **Formal verification preparation**: Properties serve as specifications for potential LEAN proofs
4. **Performance properties**: QuickCheck can also test performance characteristics (non-regression)

## Planned Commits

Based on the principle of minimal, meaningful commits that keep the repository healthy:

### Commit 1: Add Property Tests for Exact Geometric Mean
**Goal**: Add comprehensive property-based testing for the exact geometric mean implementation
- Add QuickCheck dependencies
- Add property test module to `src/main.rs`
- Implement custom generator for positive numbers
- Add all mathematical properties: single value identity, multiplicative scaling, order independence, bounds, arithmetic-geometric inequality, monotonicity, duplicates, two-value formula
- All tests should pass, demonstrating exact method is mathematically sound

**Rationale**: This is a complete, self-contained addition that validates the foundational exact method without touching approximation code.

### Commit 2: Add Property Tests for Log-Linear Approximation
**Goal**: Add comprehensive property-based testing for the approximation method
- Add property test module to `src/log_linear.rs`
- Implement custom generator `GeOneF64` for numbers ≥ 1.0 (log-linear method's domain)
- Implement custom generator `SameDigitCount` for testing same digit count behavior
- Add approximation-specific properties: order of magnitude correctness, same digit count equivalence, bounds behavior
- Add cross-method comparison properties validating relationship between exact and approximation methods
- All tests should pass, demonstrating approximation method behaves predictably

**Rationale**: This completes the property testing suite by validating the approximation method and its relationship to the exact method.

### Risk Mitigation

1. **Floating point precision**: Use appropriate tolerance values for all comparisons
2. **Input domain validation**: Clearly define and test the valid input domains
3. **Performance impact**: Monitor test suite runtime and optimize if needed
4. **False negatives**: Ensure tolerance values aren't so loose they hide real bugs
5. **Generator quality**: Validate that custom generators produce appropriate test data distributions

This comprehensive property-based testing suite will significantly increase confidence in both the exact and approximation algorithms while serving as living documentation of their mathematical properties.
