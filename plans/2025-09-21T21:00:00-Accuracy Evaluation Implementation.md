# Method Evaluation Implementation Plan

## Overview

Clean accuracy evaluation for pen-and-paper geometric mean approximation methods:
1. **Log-Linear Interpolation** - Uses digit count as logarithm proxy with linear interpolation
2. **Table-Based Approximation** - Uses memorized 10^(1/10) lookup table for logarithm conversion

## Architecture Principles

- **Trait-only public API**: Internal functions private, forcing consistent interface usage
- **Streaming evaluation**: Iterator-based processing, no large Vec allocations
- **Simplified metrics**: Focus on mean absolute relative error as primary comparison
- **Clean separation**: Evaluation logic separate from presentation
- **Set the Stage**: This change should lay the ground work for comparison, not boil the ocean

## Key Design Decisions

### Trait-Only Public API
```rust
// traits.rs
pub trait EstimateGeometricMean {
    type Error: std::error::Error;
    fn estimate_geometric_mean(&self, values: &[f64]) -> Result<f64, Self::Error>;
}
```
**Rationale**: Our evaluator should be able to evaluate _any_ estimation method, we'll use traits to allow this.

#### Associated Error Types
Different methods have different input constraints:
- **Exact method**: EmptyInput, NonPositiveValue
- **Approximation methods**: EmptyInput, NonPositiveValue, ValueTooSmall
**Rationale**: Avoids forcing impossible error variants on exact method.

#### Testing

All tests for estimated methods use trait interface exclusively - internal functions for approximations should be set to private.
No new tests are required in estimation modules.
The exact method should have just a handful of tests that confirms that the exported geometric_mean function and the trait implementation are identical.

**Rationale**: Trait is the public API, must be proven correct. Code not tested is not exposed, code that's exposed is tested.

### Evaluation Module

The evaluator module only has the following public interface.
This module does _absolutely no_ formatting or printing, just pure statistical analysis.

```rust
pub fn evaluate_estimate<R: Rng, T: EstimateGeometricMean>(rng: &mut R, method: T, min: f64, max: f64, num_tests: usize) -> Results
```

This method should iterate in place or steam or fold to prevent allocation of large lists of of tests inputs or outputs.
It accepts a min and a max, because our estimation methods don't support the full range of finite floats, even though the exact method does.

The primary property test here is that when evaluating the exact method _as_ the estimate, the result should be perfect.

#### Simplified Statistics
```rust
pub struct Results {
    mean_absolute_relative_error: f64,
    total_tests: usize,
}
```

More summary statistics will be added in future changes.
The goal of this change is to set the stage.

## Comparison

Direct comparison only occurs in the main method, as very simple print out.
Iteration count will be hard-coded.
In the future a more robust interface will be offered, we're just looking to set the stage.

No tests here, this is a dead simple demo.

## Testing Strategy
- **Prefer property tests** - Better bang for the buck
- **Example-based tests are still valuable** - Better for validating key special cases, and reader-optimized examples
- **Don't disguise example-based as property** - Just make those regular unit tests

## Rejected Alternatives

### Error Handling Approaches
1. **Shared error type** - Would force exact method to expose impossible ValueTooSmall variant

### Test Data Range Handling
1. **Min/max as trait methods** - Feels wrong, adds complexity to trait interface
2. **Discard inputs that return errors** - Wasteful, reduces test coverage
3. **Check errors in evaluator** - Still need to handle them somehow, doesn't solve root issue

### Randomness Approaches
1. **Fixed seed parameter** - Less flexible, global state concerns
2. **Thread-local RNG** - Global state, harder to test deterministically

### Comparison instead of Evaluation module
1. **Compare and Evaluate simultaneously** - Greater decoupling gives greater flexibility
