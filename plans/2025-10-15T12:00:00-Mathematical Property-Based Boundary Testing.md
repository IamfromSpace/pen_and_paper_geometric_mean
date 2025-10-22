# Mathematical Property-Based Boundary Testing Plan

**Date**: 2025-10-15T12:00:00
**Goal**: Enhance table-based approximation testing through systematic mathematical property testing that guarantees detection of rounding implementation errors

## Problem Analysis

### Current Testing Gaps

The existing test suite has a critical weakness: **rounding boundary errors** can pass all unit tests while breaking the overall algorithm. Current tests include:

1. **Unit tests**: Test individual conversion functions with predetermined inputs/outputs
2. **Property tests**: Use random inputs within broad tolerance ranges
3. **Integration tests**: Single-value and error-case validation

**Missing**: Systematic testing of the **exact boundaries** where different rounding strategies produce detectably different results.

## Table-Based Approximation Rounding Requirements

### Correct Rounding Directions

The table-based method requires **opposite rounding directions** for forward and reverse conversions:

**Forward Conversion (Number → Log)**: Round **DOWN**
- **Principle**: Conservative log-space estimation prevents error amplification in linear space
- **Error Analysis**: Floor rounding in log space provides better worst-case relative error when exponentiating back to linear space
- **Example**: For values near geometric mean √(10×100) ≈ 31.6
  - Floor rounding: log₁₀(31) ≈ 1.49 → 10^1.49 ≈ 31, relative error = 0/31 = 0%
  - Ceiling rounding: log₁₀(32) ≈ 1.51 → 10^1.51 ≈ 32, creates systematic overestimation
- **Implementation**: Find largest table entry ≤ leading digits

**Reverse Conversion (Log → Number)**: Round **UP**
- **Principle**: Ceiling rounding balances the conservative forward estimation across the full pipeline
- **Error Balancing**: After floor rounding in forward direction, ceiling in reverse prevents systematic underestimation
- **Example**: Log average 2.67 is between entries 2.6→2.7
  - Should round up to 2.7 to balance the conservative forward rounding
- **Implementation**: Use `ceil()` on scaled fractional part

### Mathematical Foundation

**Distribution Assumption**: Unlike linear averaging, geometric mean computation assumes log-uniform distribution of input numbers.
This means equal probability density in log space, not linear space.
It may be counter-intuitive since there are more numbers above the logarithmic midpoint than below (e.g., between 10 and 100, there are 68 integers above √(10×100) ≈ 31.6 versus 22 below), but this linear counting is irrelevant for log-distributed data where we expect equal probability mass on either side of the midpoint in log space.
Therefore, rounding decisions should optimize for log-space error characteristics rather than linear counting arguments.

**Error Amplification Analysis**: The critical insight is how errors propagate through the Number→Log→Average→Number pipeline:
- **Forward errors**: Rounding errors in log conversion become multiplicative errors when exponentiating back
- **Averaging effects**: Addition in log space equals multiplication in linear space, so systematic bias compounds geometrically
- **Worst-case bounds**: Floor rounding in log space provides superior worst-case relative error bounds in linear space

**Comprehensive Error Analysis**:
Consider the boundary between table entries with a concrete example around values near 16:

**Scenario 1 - Value at boundary (16.0)**:
- Floor strategy: 16.0 → log₂(16) = 4.0 → 2^4.0 = 16.0, relative error = 0%
- Ceiling strategy: 16.0 → log₂(16) = 4.0 → 2^4.0 = 16.0, relative error = 0%

**Scenario 2 - Value just below higher entry (15.9)**:
- Floor strategy: 15.9 → log₂(8) = 3.0 → 2^3.0 = 8.0, relative error = (8.0-15.9)/15.9 ≈ -50%
- Ceiling strategy: 15.9 → log₂(16) = 4.0 → 2^4.0 = 16.0, relative error = (16.0-15.9)/15.9 ≈ +0.6%

**Scenario 3 - Value just above lower entry (8.1)**:
- Floor strategy: 8.1 → log₂(8) = 3.0 → 2^3.0 = 8.0, relative error = (8.0-8.1)/8.1 ≈ -1.2%
- Ceiling strategy: 8.1 → log₂(16) = 4.0 → 2^4.0 = 16.0, relative error = (16.0-8.1)/8.1 ≈ +98%

**Key Insight**: Floor rounding produces bounded worst-case errors (~50% underestimation), while ceiling rounding can produce unbounded overestimation errors (approaching 100% for values just above the lower boundary). The multiplicative nature of geometric operations amplifies overestimation errors more severely than underestimation errors.

**Pipeline Error Balancing**:
- **Forward floor rounding**: Conservative estimation prevents systematic overestimation in log space
- **Reverse ceiling rounding**: Compensates for conservative forward bias, ensuring final results aren't systematically underestimated
- **Combined effect**: Creates an error-balanced pipeline that minimizes worst-case relative error

**Scaling Property**: The key insight enabling systematic boundary testing:
- `N × 10` maps to `L + 1.0` (exactly)
- `N ÷ 10` maps to `L - 1.0` (exactly)
- Linear mixing creates predictable fractional averages for testing

**Boundary Control**: This relationship provides precise control over fractional log components, enabling systematic testing of every rounding boundary.

## Mathematical Properties for Testing

### Property 1: Forward Rounding Direction Test
**Principle**: Tests that forward conversion (Number → Log) consistently rounds DOWN to table boundaries.

**Primary Test**: `estimate([N]) == estimate([estimate([N]) + 1])`
**Complementary Test**: `estimate([N]) > estimate([estimate([N]) - 1])`

**Boundary-Forcing Mechanism**: The double estimation `estimate([N])` automatically forces the value to a table boundary. Then:
- Adding 1 steps just above that boundary - should round back DOWN to the same entry
- Subtracting 1 steps just below that boundary - should round DOWN to the next lower entry

**Minimum Valid N**: Must be ≥ 8
**Catches**: Forward conversion errors (round-up vs round-down, off-by-one boundary detection)

### Property 2: Fractional Boundary Precision
**Principle**: By controlling the exact fractional component of log averages, we can test the precise boundaries where rounding decisions occur.

**Why the ×10 Scaling Technique Works**:
The power of scaling by exactly 10 is that it **automatically guarantees** predictable precision:
- Scaling by 10 shifts log by exactly 1.0 while preserving leading digit characteristics
- Example: `1234 → 12340 → 1234` (all have leading digit ~1.23, map to same table entry)
- **Guaranteed property**: base, base×10, and base÷10 naturally map to the same table entry
- This eliminates confounding quantization effects from the base values themselves

**Simple Property-Based Testing**:
Instead of complex fractional calculations, use this elegant monotonicity property:

**Core Property**: For any base `x` and counts `n`, `m`:
```
estimate([n copies of x×10, (m+1) copies of x]) ≤ estimate([x×10])
```

**Why This Works**:
- `x×10` maps to log `L+1`, `x` maps to log `L`
- Mixed array average: `L + n/(n+m+1)` where `n/(n+m+1) < 1`
- Therefore mixed array log < pure high value log
- **Key insight**: This tests the complete rounding pipeline with random boundary conditions

## Why This Approach is Superior

### Mathematical Foundation
Rather than guessing edge cases, this approach is **mathematically derived** from the algorithm's structure. Every test case has predictable behavior that can be verified.

### Complete Coverage
Systematic generation ensures testing of **all critical boundaries** rather than hoping random inputs will hit them.

### Error Detection Precision
Can detect subtle implementation errors (like half-up vs. round-down) that would be invisible to tolerance-based property tests.

### Algorithm Agnostic
This testing approach works for any log-space averaging algorithm, making it valuable beyond just the current table-based implementation.

## Essential Concrete Test Examples

While property tests provide systematic coverage, concrete examples offer immediate comprehension and would likely have caught the original rounding bug:

**Example 1: README Table Method Case**
```
Input: [3600, 920, 740]
Expected: 1250
Why Critical: Tests complete pipeline with realistic trivia-like values
Catches: End-to-end rounding errors in multi-value averaging
```

**Example 2: Exact Table Boundary**
```
Input: [1251] (leading digit nearly 1.25)
Expected: 1250 (should map to table index 0, multiplier 1.00 due to floor rounding)
Why Critical: Tests forward conversion floor rounding at exact table entry boundary
Catches: Forward rounding errors (≤ vs < comparisons)
```

**Example 3: Fractional Average Forcing Ceiling**
```
Input: [1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 8000]
Logs: [3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.9] → Average: 3.09
Expected: 1250 (fractional 0.09 should ceiling to 0.1, mapping to next table entry)
Why Critical: Forces reverse conversion ceiling decision
Catches: Reverse rounding errors (floor vs ceil vs round)
```

## Success Criteria

 - Both property tests are implemented and pass
 - All three example tests are implemented and pass
