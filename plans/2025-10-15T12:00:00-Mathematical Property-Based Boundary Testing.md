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

**Forward Conversion (Number → Log)**: Round **UP**
- **Principle**: Most numbers between table entries are closer to the higher value
- **Example**: Between 10 and 100, logarithmic midpoint ≈ 31.6
  - Numbers 32-99 (68 values) closer to 100 → should round up to log 2.0
  - Numbers 10-31 (22 values) closer to 10 → should round up to log 1.0
- **Implementation**: Find smallest table entry ≥ leading digits

**Reverse Conversion (Log → Number)**: Round **DOWN**
- **Principle**: Conservative estimation prevents systematic overestimation
- **Example**: Log average 2.67 is between entries 2.6→2.7
  - Should round down to 2.6 to avoid inflating the result
- **Implementation**: Use `floor()` on scaled fractional part

### Mathematical Foundation

**Scaling Property**: The key insight enabling systematic boundary testing:
- `N × 10` maps to `L + 1.0` (exactly)
- `N ÷ 10` maps to `L - 1.0` (exactly)
- Linear mixing creates predictable fractional averages for testing

**Boundary Control**: This relationship provides precise control over fractional log components, enabling systematic testing of every rounding boundary.

## Mathematical Properties for Testing

### Property 1: Forward Rounding Direction Test
**Principle**: Tests that forward conversion (Number → Log) consistently rounds UP to table boundaries.

**Primary Test**: `estimate([N]) == estimate([estimate([N]) - 1])`
**Complementary Test**: `estimate([N]) < estimate([estimate([N]) + 1])`

**Boundary-Forcing Mechanism**: The double estimation `estimate([N])` automatically forces the value to a table boundary. Then:
- Subtracting 1 steps just below that boundary - should round back UP to the same entry
- Adding 1 steps just above that boundary - should round UP to the next higher entry

**Minimum Valid N**: Must be ≥ 8
**Catches**: Forward conversion errors (round-down vs round-up, off-by-one boundary detection)

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
Input: [1249] (leading digit nearly 1.25)
Expected: 1250 (should map to table index 1, multiplier 1.25)
Why Critical: Tests forward conversion at exact table entry boundary
Catches: Forward rounding errors (≥ vs > comparisons)
```

**Example 3: Fractional Average Forcing Floor**
```
Input: [1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 8000]
Logs: [3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.9] → Average: 3.09
Expected: 1000 (fractional 0.09 should floor to 0.0)
Why Critical: Forces reverse conversion floor decision
Catches: Reverse rounding errors (ceil vs floor vs round)
```

## Success Criteria

 - Both property tests are implemented and pass
 - All three example tests are implemented and pass
