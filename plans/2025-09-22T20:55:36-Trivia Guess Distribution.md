# Trivia Guess Number Distribution Implementation Plan

## Overview

Implement a `trivia_guess` module that generates realistic trivia-style number guesses using a log-normal distribution around the correct answer.
This is foundational infrastructure for Practice Mode, enabling simulation of human guessing patterns for testing and training purposes.

Unlike naive uniform distributions, real trivia guesses cluster around the correct answer with log-normal distribution, where the variance reflects the collective knowledge and uncertainty of the guessing group.

## Motivation

Practice Mode will allow users to time themselves using pen-and-paper geometric mean methods and check accuracy against realistic guess patterns.
To make this effective, we need to generate numbers that match how people actually guess in trivia scenarios - using round numbers with different precision rules based on magnitude.

The trivia_guess generator enables:
- **Realistic practice scenarios**: Users can practice with guess patterns that match real trivia situations
- **Method evaluation**: Test geometric mean algorithms against human-like input distributions
- **Training data generation**: Create large datasets of realistic guess combinations for analysis
- **Consistency validation**: Ensure practice mode scenarios are reproducible and fair

## Architecture Design

### Core Interface

The module should provide a `TriviaGuessDistribution` struct that implements the `Distribution<u64>` trait from the rand crate.
This follows idiomatic Rust patterns and integrates seamlessly with the existing randomness ecosystem.

**Interface Signature**:
```rust
TriviaGuessDistribution::new(correct_answer: u64, log_std_dev: f64) -> Result<Self, TriviaGuessDistributionError>
```

**Parameter Definition**:
- `correct_answer`: The true answer that human guesses should cluster around
- `log_std_dev`: Standard deviation in the natural logarithmic domain (ln), representing uncertainty in orders of magnitude

**Uncertainty Factor Interpretation**:
- `log_std_dev = 0.0`: Perfect certainty - always returns the correct answer rounded to valid trivia format
- `log_std_dev = 0.5`: Guesses span roughly ±1.6× the correct answer (68% within ~0.6× to 1.6× correct answer)
- `log_std_dev = 1.0`: Guesses span roughly ±2.7× the correct answer (68% within ~0.37× to 2.7× correct answer)
- `log_std_dev = 1.5`: Guesses span roughly ±4.5× the correct answer (68% within ~0.22× to 4.5× correct answer)

This provides intuitive control over group knowledge: smaller values represent more informed guessing groups, larger values represent groups with greater uncertainty about the correct order of magnitude.

**Design Requirements**:
- Implement `Distribution<u64>` trait for integration with rand ecosystem
- Stateless sampling for thread safety and composability
- Correct answer and logarithmic standard deviation stored in distribution struct, validated at construction time
- Single responsibility: define how to sample trivia-realistic u64 values around a known correct answer
- Composable with other distributions and rand functionality
- Generate plain u64 values directly (no wrapper types)
- **No public accessor methods**: Do not expose internal fields or state via public methods (YAGNI principle)

### Rounding Rules by Magnitude

The generator implements trivia-realistic rounding based on the first digit.

**Leading digit position terminology**: For a number like 1250, the "leading digit position" refers to the magnitude of the first digit (1000 in this case). A step of 0.05 in the leading digit position means 0.05 × 1000 = 50, so valid values would be 1000, 1050, 1100, 1150, etc.

#### Numbers Starting with 1
- **Rule**: Steps of 0.05 in the leading digit position
- **Examples**: 100, 105, 110, 115, 120, 125, 130...
- **Examples**: 1000, 1050, 1100, 1150, 1200, 1250, 1300...
- **Counter-examples**: NOT 101, 102, 103, 104, 106, 107, 108, 109, 111, 112...

#### Numbers Starting with 2-4
- **Rule**: Two significant digits allowed
- **Examples**: 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30...
- **Examples**: 200, 210, 220, 230, 240, 250, 260, 270, 280, 290, 300...
- **Counter-examples**: NOT 205, 215, 225, 235, 245, 255, 265, 275, 285, 295...

#### Numbers Starting with 5+
- **Rule**: Half-steps in the leading digit position
- **Examples**: 500, 550, 600, 650, 700, 750, 800, 850, 900, 950...
- **Examples**: 5000, 5500, 6000, 6500, 7000, 7500, 8000, 8500, 9000, 9500...
- **Counter-examples**: NOT 510, 520, 530, 540, 560, 570, 580, 590, 610, 620...

### Generation Algorithm

#### Phase 1: Log-Normal Generation Around Correct Answer
Generate a value from a log-normal distribution with:
- **Mean**: `ln(correct_answer)` (distribution centered on the correct answer in log space)
- **Standard Deviation**: `log_std_dev` (explicit standard deviation in natural log domain)

This models realistic human guessing behavior where people cluster around the true value with log-normal uncertainty. The logarithmic standard deviation directly controls the spread in orders of magnitude, providing precise mathematical control over group knowledge levels.

#### Phase 2: Round to Trivia-Realistic Value in Logarithmic Domain (O(1) Algorithm Required)
Apply rounding rules in the logarithmic domain to preserve mathematical relationships, then convert back to linear space.

**Required Behavior**:
1. Convert raw log-normal sample to logarithmic representation
2. **Efficiently determine** appropriate rounding rule in log space based on magnitude and (hypothetical linear space) first digit
3. **Directly compute** the correct rounded value using mathematical formulas, not search
4. Convert rounded log value back to integer
5. Handle edge cases (values near 1, extreme outliers)

**Critical Performance Constraint**: This phase must be implemented as a direct mathematical computation, NOT as a search through pre-generated candidates. The algorithm must determine the correct trivia-rounded value in constant time regardless of magnitude.

## Rounding Behavior Requirements

### Mathematical Correctness Through Logarithmic Domain Rounding

**Critical Insight**: All rounding must occur in the logarithmic domain to preserve mathematical relationships. Linear-domain rounding destroys the geometric relationships that make trivia-realistic rounding meaningful.

This approach ensures that the spacing between valid guesses reflects proportional rather than absolute differences, which matches how humans conceptualize magnitude differences in trivia scenarios.

#### Critical Mathematical Requirements

**Logarithmic vs Linear Rounding Distinction:**

The implementation MUST use logarithmic distance for rounding decisions, not linear distance.

**Key Test Cases to Validate Logarithmic Rounding:**

1. **Basic logarithmic midpoint**:
   - Between 100,000 and 105,000, the log midpoint is ~102,469.5
   - Requirement: 102,469 → 100,000, 102,470 → 105,000

2. **Cross-magnitude rounding** (CRITICAL):
   - Between 195,000 and 200,000, the log midpoint is ~197,484.2
   - Requirement: 197,484 → 195,000 (stays in "first digit 1" rule)
   - Requirement: 197,485 → 200,000 (jumps to "first digit 2" rule)
   - This tests that the algorithm correctly handles rounding across different rule sets

3. **Rule transitions at scale boundaries**:
   - Between 950,000 and 1,000,000 (both use "5+" rule but different magnitudes)
   - Between 450,000 and 500,000 (transitions from "2-4" rule to "5+" rule)

**Requirements Summary:**
- Rounding distance MUST be measured in logarithmic space: |ln(input) - ln(candidate)|
- The implementation MUST handle cases where rounding crosses between different first-digit rule sets
- Magnitude transitions (e.g., 999,000 area) must be handled correctly
- Perfect certainty (log_std_dev = 0.0) must produce deterministic results based on logarithmic proximity

### Implementation Complexity

Rounding in logarithmic space to irregular intervals presents significant implementation challenges:
- Converting trivia rounding rules (designed for linear representation) into equivalent logarithmic operations
- Handling boundary conditions where log-space rounding crosses magnitude boundaries
- Ensuring precision and avoiding floating-point errors in the conversion chain

### Rounding Rule Application in Log Space

The implementation must apply different rounding rules based on the first digit, but perform all actual rounding operations in logarithmic domain:

- **First digit 1**: Round to steps of 0.05 in the leading digit position (converted to equivalent log-space intervals)
- **First digits 2-4**: Round to exactly two significant digits (converted to equivalent log-space intervals)
- **First digits 5+**: Round to half-steps in the leading digit position (converted to equivalent log-space intervals)

The rounding must preserve the appropriate precision level for each case while maintaining mathematical consistency throughout the logarithmic domain operations.

## Property-Based Testing Strategy

### Core Mathematical Properties

#### Generation Properties
1. **Range Bounds**: All generated values should be positive integers
2. **Infallible Sampling**: Once constructed, `sample()` method never fails
3. **Distribution Shape**: Generated samples should follow log-normal distribution around correct answer

#### Rounding Rule Properties
1. **Rounding Consistency**: Rounding the same raw value should always produce the same result
2. **Trivia-Realistic Output**: All rounded values should conform to the trivia rounding rules
3. **Log-Domain Preservation**: Rounding should preserve proportional relationships when viewed logarithmically

#### Statistical Validation with Fixed Seeds

**Critical Requirement**: Use non-cherry-picked fixed seeds for statistical validation to ensure tests are reproducible but not biased toward favorable outcomes.

**Statistical Properties to Validate**:
1. **Central Tendency**: Geometric mean of large sample should approximate correct answer
2. **Distribution Shape**: Sample distribution should exhibit log-normal characteristics
3. **Standard Deviation**: Measured standard deviation in log domain should approximate `log_std_dev` parameter
4. **Uncertainty Scaling**: Higher `log_std_dev` values should produce proportionally wider distributions in log space
5. **Boundary Behavior**: Distribution should handle extreme correct answers gracefully

#### Boundary Condition Properties
1. **Edge Case: correct_answer = 1**: Distribution should construct successfully and produce realistic small-number guesses
2. **Edge Case: very large correct answers**: Should handle values near u64::MAX without overflow
3. **Edge Case: log_std_dev extremes**: Should handle both very certain (small log_std_dev) and very uncertain (large log_std_dev) scenarios

### Test Organization

Tests should live in the same file as the code they test.

## Implementation Phases

### Phase 1: Core Structure and Basic Generation
- Create `src/trivia_guess.rs` module
- Implement `TriviaGuessDistribution` struct with `Distribution<u64>` trait
- Add exponential scale generation (Phase 1 of algorithm)
- Add basic rounding implementation
- Add unit tests for rounding rules with known inputs
- Ensure compilation and basic functionality

### Phase 2: Comprehensive Rounding Rules
- Implement detailed rounding logic for each first-digit case
- Handle edge cases (values near 1, values near max_value)
- Add floating point precision safeguards
- Extend unit tests to cover all rounding scenarios
- Add error handling for invalid inputs

### Phase 3: Property-Based Testing
- Add QuickCheck dependency (if not already present)
- Implement custom generators for test scenarios
- Add comprehensive property tests for mathematical properties
- Add boundary condition tests
- Ensure all properties pass consistently

#### Key Validation Tests
The following three specific tests are critical for validating the implementation:

1. **Three-Digit Sample Validation Test**:
   - Create distribution with `correct_answer=316, log_std_dev=1.151`
   - Sample many values, filter to three-digit results (100-999)
   - Verify all three-digit samples are members of the complete valid trivia number set
   - This validates end-to-end correctness of the log-normal generation + rounding pipeline

2. **Deterministic Perfect Certainty Test**:
   - For each valid trivia number at various scales, create distribution with `log_std_dev=0.0`
   - Verify that sampling always returns the correct answer (deterministic rounding)
   - This validates that zero uncertainty produces perfectly predictable results

3. **Boundary Rounding Between Adjacent Valid Values**:
   - For adjacent pairs of valid trivia numbers, find their geometric middle point
   - Scale the test values (e.g., `middle_value × 10000` to test 5-6 digit numbers like 324,650 and 324,680)
   - Create test points slightly below and above the scaled geometric middle with `log_std_dev=0.0`
   - Verify both test points round to one of the two adjacent scaled valid values and to different values
   - This validates that rounding boundaries work correctly at the logarithmic middle point between valid values

### Phase 4: Documentation
- Add documentation for public methods
- Add module-level documentation describing the purpose and usage

## Design Constraints

### Compatibility Requirements
- **No changes to existing interfaces**: Module should be additive only
- **Consistent with existing patterns**: Follow same error handling and testing patterns as other modules
- **QuickCheck integration**: Use same property testing approach as existing modules

### Performance Requirements
- **O(1) generation time**: Each guess should be generated in constant time
- **Memory efficiency**: Generator should have minimal memory footprint
- **Deterministic**: Same seed should produce identical sequences
- **Numeric operations only**: No string manipulation or other inefficient operations
- **Engineering excellence**: Well-designed algorithms appropriate for the mathematical operations

#### Critical Algorithmic Efficiency Requirements

**PROHIBITED APPROACHES** (these violate O(1) requirement):
- **No candidate generation**: Do not generate lists/arrays of possible valid trivia numbers and search through them
- **No brute force search**: Do not iterate through potential values to find the closest match
- **No linear scanning**: Do not examine multiple candidates per sample

**REQUIRED APPROACH** (achieves true O(1) performance):
- **Direct mathematical computation**: Given a log-domain value, directly compute which "interval" or "bucket" it falls into based on trivia rounding rules
- **At most 2 candidates**: Compute exactly the 2 nearest valid trivia values (above and below) for the determined interval
- **Direct selection**: Choose between these 2 values based on logarithmic distance without iteration
- **Algorithmic rounding**: Use mathematical formulas to determine rounding boundaries, not enumeration

**Implementation Strategy**:
1. **Interval Detection**: From log-domain input, algorithmically determine magnitude and first-digit rule type in O(1)
2. **Boundary Computation**: Mathematically compute the boundaries of the rounding interval for that rule type in O(1)
3. **Direct Rounding**: Apply logarithmic rounding within that specific interval in O(1)

This ensures that each sample requires only a constant number of arithmetic operations regardless of the magnitude or complexity of trivia rounding rules.

## Error Handling Strategy

### Input Validation
The module should define appropriate error types for constructor validation.

### Error Cases
1. **correct_answer = 0**: Constructor returns `InvalidCorrectAnswer` error
2. **log_std_dev < 0 or NaN**: Constructor returns `InvalidLogStdDev` error (note: `log_std_dev = 0.0` is explicitly allowed for deterministic rounding)
3. **log_std_dev > 50.0**: Constructor returns `LogStdDevTooLarge` error (prevents floating point overflow/underflow: extreme tail events at 6σ could push exp() beyond f64 range, causing mathematical instability)

### Recovery Strategy
- Validation at construction time prevents all runtime errors
- Once constructed, distribution is guaranteed to work correctly
- Error messages should be descriptive for debugging
- No runtime failures: sampling operations are infallible after successful construction

## Success Criteria

1. **Functional Correctness**: All rounding rules produce expected outputs for known inputs
2. **Mathematical Rigor**: Log-domain rounding preserves geometric relationships correctly
3. **Statistical Validity**: Distribution exhibits proper log-normal characteristics around correct answer
4. **Property Validation**: All mathematical properties hold under QuickCheck testing with fixed seeds
5. **O(1) Performance Validated**: Each sample generation must complete in constant time with no candidate iteration or brute force search - performance must be independent of magnitude and number of possible trivia values
6. **Comprehensive Testing**: Edge cases and error conditions are properly handled
7. **Clear Documentation**: Module is well-documented with usage examples
8. **Ergonomic API**: Constructor handles validation, `sample()` method is simple and infallible
9. **Implementation Elegance**: Code must be elegant and self-describing to optimize for review

### Implementation Excellence Requirement

**Critical**: Given the mathematical complexity of logarithmic domain rounding to irregular intervals, the implementation must prioritize elegance and self-description. A correct but opaque implementation is insufficient. We must optimize for review _because_ the problem is difficult.

**Comments are not a shortcut**: While comments should absolutely be used to highlight surprising, unexpected, or subtle details, the primary goal is to write code that doesn't need extensive commenting in the first place. Comments should explain _why_ something is done when it's not obvious from the _what_.

This trivia_guess module will serve as essential infrastructure for Practice Mode while following the project's principles of strong testing, clear abstraction, and evolutionary design.
