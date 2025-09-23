# Trivia Guess Number Distribution Implementation Plan

## Overview

Implement a `trivia_guess` module that generates realistic trivia-style number guesses on an exponential scale.
This is foundational infrastructure for Practice Mode, enabling simulation of human guessing patterns for testing and training purposes.

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

**Design Requirements**:
- Implement `Distribution<u64>` trait for integration with rand ecosystem
- Stateless sampling for thread safety and composability
- Maximum value stored in distribution struct, validated at construction time
- Single responsibility: define how to sample trivia-realistic u64 values
- Composable with other distributions and rand functionality
- Generate plain u64 values directly (no wrapper types)

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

#### Phase 1: Exponential Scale Selection
Generate a value uniformly on the logarithmic scale between 1 and `max_value`, then convert to integer. This ensures that values are distributed exponentially rather than linearly, matching how trivia questions often span multiple orders of magnitude.

#### Phase 2: Round to Trivia-Realistic Value
Apply rounding rules based on the first digit of the generated integer value.

**Required Behavior**:
1. Extract the first digit from the integer value
2. Apply appropriate rounding rule based on first digit
3. Ensure result doesn't exceed `max_value`
4. Handle edge cases (values of 1, values near `max_value`)

## Rounding Behavior Requirements

### First Digit Extraction
The implementation must efficiently determine the first digit of any positive integer in constant time using mathematical operations rather than iterative approaches like loops or string manipulation.

**Performance requirement**: Use logarithmic operations to calculate order of magnitude and extract the first digit mathematically, avoiding any iterative division or character-based approaches.

### Rounding Rule Application
The implementation must apply different rounding rules based on the first digit:

- **First digit 1**: Round to steps of 0.5 in the leading digit position
- **First digits 2-4**: Round to exactly two significant digits
- **First digits 5+**: Round to half-steps in the leading digit position

The rounding must preserve the appropriate precision level for each case while maintaining mathematical consistency and producing valid integers.

## Property-Based Testing Strategy

### Core Mathematical Properties

#### Generation Properties
1. **Range Bounds**: `1 <= guess <= max_value` for any successfully constructed distribution
2. **Infallible Sampling**: Once constructed, `sample()` method never fails

#### Rounding Rule Properties
1. **Rounding Consistency**: Rounding the same raw value should always produce the same result
2. **Magnitude Preservation**: Rounding should never change the order of magnitude by more than one step

#### Boundary Condition Properties
1. **Edge Case: max_value = 1**: Distribution should construct successfully and always return 1 (Example-based test)
2. **Edge Case: max_value just above power of 10**: Should handle transitions cleanly (Example-based test)
3. **Edge Case: max_value very large**: Should not overflow or panic

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

## Error Handling Strategy

### Input Validation
The module should define appropriate error types for constructor validation.

### Error Cases
1. **max_value = 0**: Constructor returns `InvalidMaxValue` error
2. **Internal overflow during construction**: Constructor returns `GenerationFailed` error (though this should be extremely rare with u64)

### Recovery Strategy
- Validation at construction time prevents all runtime errors
- Once constructed, distribution is guaranteed to work correctly
- Error messages should be descriptive for debugging

## Success Criteria

1. **Functional Correctness**: All rounding rules produce expected outputs for known inputs
2. **Property Validation**: All mathematical properties hold under QuickCheck testing
3. **Performance Acceptable**: Generation performance supports high-volume testing
4. **Integration Ready**: Module integrates cleanly with existing codebase patterns
5. **Comprehensive Testing**: Edge cases and error conditions are properly handled
6. **Clear Documentation**: Module is well-documented with usage examples
7. **Ergonomic API**: Constructor handles validation, `sample()` method is simple and infallible

This trivia_guess module will serve as essential infrastructure for Practice Mode while following the project's principles of strong testing, clear abstraction, and evolutionary design.
