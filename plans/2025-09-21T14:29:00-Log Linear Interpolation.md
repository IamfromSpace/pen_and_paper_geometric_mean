# Second Implementation Plan - Log + Linear Interpolation Approximation

**Date**: 2025-09-21T14:29
**Goal**: Implement the first pen-and-paper approximation method for geometric mean estimation

## Current State
- Core geometric mean function implemented in `src/main.rs:18-32`
- Comprehensive test suite with 10 test cases covering edge cases
- Existing error handling with `GeometricMeanError` enum
- Foundation ready for approximation method comparison

## Proposed Change: New Module for Log + Linear Interpolation Method

### Module Structure
Create new module `src/log_linear.rs` to house this approximation method:
- Keeps `main.rs` from becoming too large
- Allows for clean separation of approximation methods
- `GeometricMeanError` will be duplicated (acceptable cost for modularity)
- Future approximation methods can follow same modular pattern
- **IMPORTANT**: Leave the existing `geometric_mean` implementation in `main.rs` completely untouched - we can extract it to its own module in a future change

### Algorithm Overview
This method approximates geometric mean without requiring memorization or complex calculations:

1. **Convert each guess to `[digit_count].[full_fractional]` format**:
   - Count non-decimal digits (digit_count)
   - Take all remaining digits as fractional portion
   - Examples: 300 → 3.3, 10000 → 5.1, 900 → 3.9, 70 → 2.7, 2847 → 4.2847

2. **Average the converted values arithmetically**
   - Example: (3.3 + 5.1 + 3.9 + 2.7) / 4 = 3.75

3. **Convert back to final estimate**:
   - Whole part gives digit count
   - Decimal part gives the significant digits, followed by zeros to reach digit count
   - **Edge case handling**: If decimal part is 0 or rounds to 0, use 0.1 instead
   - Examples: 3.75 → 750, 4.025 → 1000 (treated as 4.1), 4.0 → 1000 (treated as 4.1)

### Implementation Requirements

#### Module Setup
In `src/main.rs`, add: `mod log_linear;`

#### Core Function in `src/log_linear.rs`
```rust
#[derive(Debug, PartialEq)]
pub enum GeometricMeanError {
    EmptyInput,
    NonPositiveValue,
    ValueTooSmall,
}

pub fn log_linear_approximation(values: &[f64]) -> Result<f64, GeometricMeanError>
```

#### Helper Functions Needed
1. `convert_to_log_linear(value: f64) -> f64`
   - Input: positive number (e.g., 2847.0)
   - Output: digit_count.full_fractional format (e.g., 4.2847)

2. `convert_from_log_linear(log_value: f64) -> f64`
   - Input: digit_count.fractional format (e.g., 3.75)
   - Output: reconstructed number (e.g., 750.0)
   - Must handle edge case: fractional ≤ 0.1 becomes 0.1

#### Error Handling
- Duplicate `GeometricMeanError` enum in this module
- Same validation as `geometric_mean`: no empty input, no non-positive values
- Add validation for values < 1.0 → `GeometricMeanError::ValueTooSmall`
- Values less than 1 are out of scope for this pen-and-paper method due to complexity

### Test Cases to Implement

#### Basic Functionality Tests
1. **README example**: [300, 10000, 900, 70] → should approximate 750
2. **Same digit count**: [100, 200, 300] → should equal arithmetic mean (200)
3. **Single value**: [500] → should return 500
4. **Two values**: [100, 1000] → should approximate √(100000) ≈ 316
5. **Edge case testing**: Test the 0.1 minimum rule with inputs that result in averages like 4.025 or 4.0 → both should become 1000
6. **Concrete edge case**: [80, 80, 80, 800] → converts to [2.8, 2.8, 2.8, 3.8] → average 3.05 → treated as 3.1 → result 100

#### Algorithm Correctness Tests
1. **Hand-calculated verification** - test cases where the expected result can be manually verified
2. **Step-by-step validation** - test the conversion functions independently

#### Edge Cases
1. **Empty input** → `GeometricMeanError::EmptyInput`
2. **Zero/negative values** → `GeometricMeanError::NonPositiveValue`
3. **Very large numbers** (test precision limits)
4. **Values < 1.0** → `GeometricMeanError::ValueTooSmall`

### Implementation Details

#### Digit Counting Strategy
- Use logarithm base 10 to count digits: `(value.log10().floor() as i32) + 1`
- All input values are >= 1.0, so no special handling needed for small values

#### Fractional Part Extraction
- Use numerical operations only (no string manipulation for performance)
- After determining digit count, extract all remaining digits as fractional part
- Example: 2847 → 4 digits, remaining digits 2847, so becomes `4.2847`
- Algorithm: divide by appropriate power of 10 to normalize the fractional part

#### Reverse Conversion Logic
- Use numerical operations only (no string manipulation for performance)
- Split log_value into whole and fractional parts using floor() and modulo
- Whole part = digit count
- **Edge case first**: If fractional part < 0.1, set fractional part = 0.1
- Fractional part represents the significant digits directly
- Construct result by multiplying fractional part by appropriate power of 10
- Example: 3.75 → 3 digits, 0.75 * 1000 = 750; 4.1 → 4 digits, 0.1 * 10000 = 1000

### Success Criteria
- All tests pass including README examples
- `cargo test` runs clean
- Algorithm produces correct results for hand-verifiable test cases
- Code follows same patterns as existing `geometric_mean` function
- Clear documentation of algorithm steps in code comments

### Future Integration Notes
This implementation provides the foundation for future work:
1. Accuracy comparison with true geometric mean (separate future task)
2. Comparison with the 10^(1/10) table method (next planned approximation method)
3. Monte Carlo simulation of approximation errors
4. Formal error bound analysis mentioned in README

The function signature matches the existing `geometric_mean` pattern to enable easy future integration and comparison.
