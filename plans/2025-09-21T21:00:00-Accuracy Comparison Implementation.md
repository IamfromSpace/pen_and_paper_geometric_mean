# Method Evaluation Implementation Plan

## Overview

Clean accuracy evaluation for pen-and-paper geometric mean approximation methods:
1. **Log-Linear Interpolation** - Uses digit count as logarithm proxy with linear interpolation
2. **Table-Based Approximation** - Uses memorized 10^(1/10) lookup table for logarithm conversion

Focus on pure evaluation logic with clean separation of concerns.

## Core Requirements

### Evaluation Framework
- Evaluate each method against the exact geometric mean calculation
- Calculate accuracy metrics as pure functions
- No presentation logic in evaluation code

### Test Data Generation
- **Log-uniform distribution**: Values uniformly distributed across log scale (consistent with power law assumption)
- Various input set sizes (2-10 values)
- Fixed seed for reproducible results

### Basic Accuracy Metrics
- **Relative error**: (approximation - exact) / exact
- **Mean absolute relative error**: Average of |relative errors|
- **Success rate**: Percentage within 1 order of magnitude of exact result

## Implementation Strategy

### Clean Architecture Principles
- **EstimateGeometricMean trait** - Single responsibility: estimate geometric mean
- **Exact method implements trait too** - Enables self-testing and consistency
- **Evaluator uses concrete exact method** - No injection, calls exact::geometric_mean directly
- **Pure evaluation functions** - Return data structures, no printing/formatting
- **Presentation in main.rs** - All naming, formatting, and output logic

### First Commit: Minimal Trait Foundation
Create simple trait interface:
- **EstimateGeometricMean trait** - Only `fn estimate_geometric_mean(&self, values: &[f64]) -> Result<f64, Self::Error>`
- **Associated Error type** - Each method has its own error type (exact doesn't need ValueTooSmall variant)
- **No name() method** - Caller decides naming and presentation
- **Trait implementations** - All three methods (exact, log-linear, table-based)
- **Keep existing APIs** - Don't break current function interfaces

#### Error Type Rationale
Different methods have different constraints:
- **Exact method**: Only needs EmptyInput and NonPositiveValue errors
- **Approximation methods**: Need additional ValueTooSmall error for values < 1.0
- **Associated type approach**: Avoids forcing exact method to expose impossible error variants

### Second Commit: Pure Evaluation Framework
Build evaluation system with no presentation logic:
- **Pure evaluation function** - Takes exact values, approximation values, returns statistics
- **Test data generator** - Pure function returning Vec<Vec<f64>>
- **Statistics struct** - Simple data container with no formatting methods
- **Main.rs handles everything else** - Method names, printing, formatting

### Test Data Generator
Single pure function with injectable randomness and method-specific constraints:
```rust
fn generate_test_data<R: Rng>(
    rng: &mut R,
    num_tests: usize,
    min_value: f64,
    max_value: f64
) -> Vec<Vec<f64>>
```

#### RNG Injection Rationale
Using `rand::Rng` trait provides:
- **Deterministic testing**: Pass seeded RNG like `StdRng::seed_from_u64(42)`
- **No global state**: Pure functional approach, easier to test
- **Flexibility**: Could use different RNG implementations if needed
- **Testability**: Unit tests with predictable, repeatable randomness

#### Test Range Rationale
Different methods have different valid input ranges:
- **Exact method**: All positive values (min_value ≥ ε > 0)
- **Approximation methods**: Values ≥ 1.0 (min_value ≥ 1.0)
- **Evaluator approach**: Accept min/max parameters to respect method constraints

### Evaluation Function
Single pure function:
```rust
fn evaluate_accuracy(exact_results: &[f64], approximation_results: &[f64]) -> AccuracyStats
```

Note: The evaluator internally calls `exact::geometric_mean` directly for comparison, but the exact method also implements the trait for consistency and self-testing.

### Statistics Container
Simple data struct:
```rust
struct AccuracyStats {
    mean_absolute_relative_error: f64,
    success_rate: f64,
    total_tests: usize,
}
```

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

## Testing Strategy

### Evaluator Testing Requirements
The evaluation module contains pure functions that are critical for correctness.
All functions must be thoroughly tested to ensure reliable comparison results.

### Test Data Generator Testing
- **Deterministic output**: Same seed produces identical test data across runs
- **Range validation**: Generated values respect min/max bounds
- **Set size validation**: Generated sets have correct sizes (2-10 values)
- **Boundary values**: min_value, max_value, exactly 2 and 10 element sets

### Accuracy Evaluation Testing
- **Perfect approximation**: When exact == approximation, relative error = 0
- **Known error cases**: Hand-calculated examples with expected metrics
- **Success rate boundaries**: Test 0.1x and 10x ratio thresholds precisely
- **Single test case**: Arrays with length 1
- **Identical values**: All exact and approximation values are the same
- **Division by zero protection**: Ensure exact values are never zero

### Test Implementation Approach
- **Unit tests for pure functions**: Each function tested in isolation
- **Property-based tests**: Use QuickCheck for boundary conditions and input validation

### Expected Test Coverage
- **generate_test_data()**: Range validation, determinism, set sizes, boundary values
- **evaluate_accuracy()**: Error calculations, success rate logic, known cases, boundary conditions
- **AccuracyStats**: Data structure correctness

## Expected Deliverables

### Core Implementation
- Add evaluation functionality to existing main.rs
- All presentation logic in main.rs
- Pure functions in evaluation module

### Testing Implementation
- Comprehensive unit tests for evaluation module
- Property-based tests for boundary conditions and input validation

## Success Criteria

### Functional Requirements
- Minimal trait interface focused only on estimation
- Pure evaluation functions with no side effects
- Exact method used directly without trait wrapper
- Clear separation: evaluation logic vs presentation logic
- Foundation ready for easy extension

### Quality Requirements
- No printing/formatting in evaluation module
- Consistent with existing code style and error handling
- Reproducible results with fixed random seed
- All presentation decisions made in main.rs

### Testing Requirements
- All evaluation functions have comprehensive unit tests
- Property-based tests verify mathematical correctness
- Test coverage includes edge cases and boundary conditions
- Deterministic test data generation verified
- Success rate calculation accuracy validated