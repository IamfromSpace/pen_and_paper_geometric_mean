# Extended Summary Statistics Implementation Plan

## Overview

Add worst case error, worst case overestimate, and overall bias statistics to the existing evaluation system.
This is a surgical change that extends the current `Results` struct and evaluation function with minimal modifications.

## Motivation

The current evaluation only tracks mean absolute relative error, but for the trivia game use case we need:
- **Worst case error**: Track the maximum absolute relative error to understand failure modes
- **Worst case overestimate**: Track the maximum overestimate specifically (since overestimating is worse than underestimating in the trivia context)
- **Overall bias**: Track whether the method systematically over/under-estimates

## Architecture Changes

### Extended Results Structure
```rust
pub struct Results {
    pub mean_absolute_relative_error: f64,
    pub worst_case_error: f64,
    pub worst_case_overestimate: f64,
    pub overall_bias: f64,
    pub total_tests: usize,
}
```

**Design Rationale**: Extend existing struct rather than create new one to maintain API compatibility.

### Metric Definitions
- **worst_case_error**: `max(|estimate - exact| / exact)` across all tests
- **worst_case_overestimate**: `max((estimate - exact) / exact)` where `estimate > exact`, or 0.0 if no overestimates occur
- **overall_bias**: `mean((estimate - exact) / exact)` - positive means systematic overestimation, negative means underestimation

### Implementation Strategy

#### Single-Pass Computation
Extend the existing evaluation loop to track additional statistics without changing the O(1) memory constraint:

```rust
let mut max_error = 0.0;
let mut max_overestimate = 0.0;
let mut total_signed_error = 0.0; // for bias calculation
```

#### Error Handling
- **worst_case_overestimate**: If no overestimates occur, return 0.0
- **All metrics**: Return NaN if no valid tests (consistent with existing behavior)

## Testing Strategy

### Property Tests
Add QuickCheck properties to verify mathematical relationships:

1. **Error bounds**: `worst_case_error >= mean_absolute_relative_error`
2. **Overestimate bounds**: `worst_case_overestimate >= 0.0` and `worst_case_overestimate <= worst_case_error` (when overestimates exist)
3. **Bias bounds**: `-worst_case_error <= overall_bias <= worst_case_error`
4. **Exact method properties**: When evaluating exact method, `worst_case_error ~= 0`, `worst_case_overestimate ~= 0`, `overall_bias ~= 0`

### Unit Tests
- Test edge cases like all overestimates, all underestimates, mixed scenarios
- Verify calculations with known inputs and expected outputs
- Test behavior with zero valid tests

## Implementation Approach

### Phase 1: Extend Results and Core Logic
- Add new fields to `Results` struct
- Extend evaluation loop to track additional metrics
- Update existing tests to handle new fields

### Phase 2: Add Comprehensive Testing
- Add property tests for mathematical relationships
- Add unit tests for edge cases
- Verify all approximation methods work with new metrics

### Phase 3: Update Display (Future)
- Main method will eventually need updates to display new metrics
- Keep this separate from core statistical computation

## Design Constraints

### Minimal Changes Only
- **NO** changes to trait definitions or method signatures
- **NO** changes to existing test behavior expectations
- **NO** changes to memory usage patterns (maintain O(1))
- **Preserve** all existing functionality exactly

### Backward Compatibility
- Existing code using `Results` must continue to compile
- All existing tests must continue to pass
- Public API remains unchanged except for additional fields

## Error Handling Edge Cases

### No Valid Tests
All metrics return `f64::NAN` (consistent with existing behavior)

### No Overestimates
`worst_case_overestimate` returns `0.0`

### Floating Point Precision
Use same precision handling as existing mean calculation

## Future Extensions

This change sets the stage for:
- Confidence intervals and percentile tracking
- Method-specific error analysis
- More sophisticated bias analysis
- Performance regression detection

## Rejected Alternatives

### Separate Statistics Struct
Would require API changes and complicate usage

### Multiple Evaluation Passes
Would violate O(1) memory constraint and reduce performance

### Optional Statistics
Adds complexity without clear benefit - the new metrics are lightweight