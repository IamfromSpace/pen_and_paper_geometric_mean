# Step-by-Step Calculation Display for Error Recovery

## Overview

Implement step-by-step calculation display for practice mode when users make errors. This enhances the learning experience by showing the complete calculation process for the table-based approximation method, helping users understand where they went wrong and learn the correct approach.

## Requirements

### Core Functionality
- Display step-by-step calculations when user answers are incorrect
- Always calculate using step-by-step approach, current functions become wrappers
- Support table-based approximation method (log-linear method reserved for future implementation)
- Display calculations in human-readable decimal format while maintaining internal integer precision
- Maintain exact string matching for testing (no embedded formatting in data structures)
- Preserve existing API compatibility for current users

### User Interface Enhancement
```
You have calculated the estimation method incorrectly.

Step-by-step calculation:
========================
Input values: [25, 400, 1,200, 8,000]

1. Convert each value to log representation:
   25 → 1.6
   400 → 2.6
   1,200 → 3.1
   8,000 → 3.9

2. Calculate average of log representations:
   (1.6 + 2.6 + 3.1 + 3.9) ÷ 4 = 11.2 ÷ 4 = 2.8

3. Convert back to final estimate:
   2.8 → 600

Final estimation: 600
```

### Calculation Accuracy Requirements
- Step-by-step calculation must always produce identical results to current methods
- Intermediate steps shown must reflect actual calculation path taken
- No rounding differences between step-by-step and direct calculation modes

## Design Decisions

### Architecture Pattern: Step-by-Step as Primary
All estimation methods calculate step-by-step as the authoritative implementation, with simple results derived as needed.

### New Trait Design
```rust
pub trait FinalAnswer {
    fn final_answer(&self) -> f64;
}

pub trait EstimateGeometricMeanStepByStep {
    type StepByStep;
    type Error: std::error::Error;

    fn estimate_geometric_mean_steps(values: &[f64]) -> Result<Self::StepByStep, Self::Error>;
}
```

### Backward Compatibility Strategy
Existing `EstimateGeometricMean` trait preserved through explicit implementation choice:

```rust
// TableBasedApproximation implements both traits explicitly
impl EstimateGeometricMean for TableBasedApproximation {
    type Error = GeometricMeanError;

    fn estimate_geometric_mean(values: &[f64]) -> Result<f64, Self::Error> {
        // For table-based method, prioritize correctness: delegate to step-by-step
        let steps = Self::estimate_geometric_mean_steps(values)?;
        Ok(steps.final_answer())
    }
}

impl EstimateGeometricMeanStepByStep for TableBasedApproximation {
    type StepByStep = TableBasedSteps;
    type Error = GeometricMeanError;

    fn estimate_geometric_mean_steps(values: &[f64]) -> Result<Self::StepByStep, Self::Error> {
        table_based_approximation_steps(values)
    }
}

// Step-by-step struct implements supporting traits
impl FinalAnswer for TableBasedSteps {
    fn final_answer(&self) -> f64 {
        self.final_result
    }
}

impl std::fmt::Display for TableBasedSteps {
    // Human-readable step-by-step formatting implementation
}
```

Table-based method prioritizes correctness by implementing `EstimateGeometricMean` via step-by-step delegation. Future methods can choose independent implementations if other priorities (performance, memory) are more important.

### Step-by-Step Data Structures

#### Table-Based Approximation Steps
```rust
pub struct TableBasedSteps {
    input_values: Vec<f64>,
    log_conversions: Vec<i32>,  // Internal integer representation (e.g., 36)
    sum: i32,
    average: i32,
    final_result: f64,
}
```

**Display Format Note**: The `log_conversions` integer values (e.g., 36) are displayed as human-readable decimal format (e.g., 3.6) by dividing by 10. This preserves the exact integer arithmetic internally while showing the conceptual decimal representation that humans work with on pen and paper.

### Display Implementation Requirements
- **Consistent formatting**: All step-by-step displays follow same structure (input, conversions, averaging, result)
- **Human-readable format**: Internal integer arithmetic (e.g., 36) displayed as decimal format (e.g., 3.6) for conceptual clarity
- **Readable precision**: Show appropriate decimal places for human comprehension (avoid floating-point artifacts)
- **Clear progression**: Each step builds logically on the previous step
- **Table-based specifics**: Show table lookups and decimal log representations that humans would use

## Architecture

### Core Calculation Refactoring
Transform existing calculation functions to always produce step-by-step data:

#### Current Function Transformation
```rust
// Before (current)
fn table_based_approximation(values: &[f64]) -> Result<f64, GeometricMeanError>

// After (step-by-step becomes primary)
fn table_based_approximation_steps(values: &[f64]) -> Result<TableBasedSteps, GeometricMeanError>

// Original function can be removed - EstimateGeometricMean provided via other traits
// Users call either:
// - TableBasedApproximation::estimate_geometric_mean(values) // via other traits
// - TableBasedApproximation::estimate_geometric_mean_steps(values) // direct step-by-step
```

#### Step-by-Step Data Capture
Each intermediate calculation stores its input, process, and output for complete reconstruction of the calculation path.

### Practice Mode Integration
**ActiveSession** stores step-by-step calculation, rather that just the answer.

### CLI Display Logic
CLI layer determines when to show step-by-step based on evaluation result:
- **Correct/Excellent answers**: Show standard success messages only, extracting the final answer from the step-by-step type
- **Incorrect answers**: Output the full step by step calculation, via Display trait
- **Error handling**: Gracefully handle any step-by-step calculation errors

### Generic Trait Constraints
Different components apply constraints only where needed:

```rust
// Core practice session - minimal constraints
impl<T: Timer, E> ActiveSession<T, E>
where
    E: EstimateGeometricMeanStepByStep,
    E::StepByStep: FinalAnswer,  // Needed for blanket EstimateGeometricMean
{
    // EstimateGeometricMean automatically available via blanket implementation
    // EstimateGeometricMeanStepByStep for step-by-step when needed
}

// CLI formatting - adds Display constraint where needed
pub fn format_results_display<E>(result: &PracticeResult<E>) -> String
where
    E: EstimateGeometricMeanStepByStep,
    E::StepByStep: std::fmt::Display,  // CLI-specific requirement
{
    // Can format step-by-step data for display
}
```

## Testing Philosophy

Testing is integrated throughout implementation rather than as a separate phase. Key areas:

#### Display Format Testing
Test exact string output for deterministic formatting verification:

```rust
#[test]
fn test_table_based_steps_display_format() {
    let steps = TableBasedApproximation::estimate_geometric_mean_steps(&[25.0, 400.0]).unwrap();
    let output = format!("{}", steps);

    assert_eq!(output, "Input values: [25, 400]\n\n1. Convert each value to log representation:\n   25 → 1.6\n   400 → 2.6\n\n2. Calculate average of log representations:\n   (1.6 + 2.6) ÷ 2 = 4.2 ÷ 2 = 2.1\n\n3. Convert back to final estimate:\n   2.1 → 100\n\nFinal estimation: 100");
}
```

**Key Testing Principle**: Use `assert_eq!` with complete expected strings rather than `contains()` checks. This ensures the formatting is exactly right and catches any unintended changes to the step-by-step display format.

**Use Mocks**: By using mock step-by-step method, we can perfectly predict the string output
**Property-based testing**: Step-by-step equivalence, display determinism, trait implementation consistency
**Practice mode integration**: Conditional display only for incorrect answers

## Implementation Phases

### Phase 1: Core Step-by-Step Infrastructure
1. Define `FinalAnswer` and `EstimateGeometricMeanStepByStep` traits
   - **Test**: Trait compilation and basic usage patterns
   - **Test**: EstimateGeometricMeanStepByStep works without bounds on StepByStep type
2. Create step-by-step data structures for table-based method
   - **Test**: Data structure creation and field access
3. Implement `FinalAnswer` for `TableBasedSteps`
   - **Test**: `final_answer()` returns correct value from step-by-step data
   - **Test**: FinalAnswer trait integration works as expected
4. Refactor table-based calculation to always produce step-by-step data
   - **Test**: Step-by-step calculation produces identical results to current direct methods
   - **Test**: Intermediate validation - each step produces expected intermediate values
   - **Test**: Round-trip testing - values → steps → final result matches direct calculation
5. Implement Display for TableBasedSteps with formatted output
   - **Test**: Exact string output for deterministic formatting verification
   - **Test**: Human-readable decimal format display (internal 36 shows as 3.6)
6. Add `EstimateGeometricMeanStepByStep` implementation and explicit `EstimateGeometricMean` delegation
   - **Test**: All existing tests pass without modification (backward compatibility preserved)
   - **Test**: EstimateGeometricMean delegates to step-by-step calculation for correctness
   - **Test**: Both trait methods return identical results for same input

### Phase 2: Practice Mode Integration
1. Update ActiveSession and PracticeResult to store input data for on-demand calculation
   - **Test**: Session creation and data structure updates work correctly
   - **Test**: Input values preserved correctly for later step-by-step calculation
2. Add get_step_by_step() method to PracticeResult for on-demand calculation
   - **Test**: Step-by-step calculation produces identical results to stored estimation_result
   - **Test**: Method works correctly with minimal trait bounds
3. Update CLI formatting to use on-demand step-by-step display
   - **Test**: Correct answers show no step-by-step information
   - **Test**: Excellent answers show no step-by-step information
   - **Test**: Incorrect answers show complete step-by-step calculations in human-readable format
4. Add appropriate trait constraints where needed (both traits for sessions, Display for CLI)
   - **Test**: Core practice session works with dual trait bounds (EstimateGeometricMean + EstimateGeometricMeanStepByStep)
   - **Test**: CLI formatting works with Display bounds applied only where needed
   - **Test**: Both trait methods can be called independently as needed

## Success Criteria

Implementation complete when:
1. All existing tests pass without modification (backward compatibility preserved)
2. Incorrect answers in practice mode show complete step-by-step calculations in human-readable decimal format
3. Correct and excellent answers show no step-by-step information
4. Step-by-step calculations produce identical results to current direct methods
5. Display output format is clean, readable, and shows decimal representations (e.g., 3.6) while maintaining internal integer precision
6. Testing demonstrates exact string matching for step-by-step output
7. Table-based method supports step-by-step display (log-linear method reserved for future implementation)

## Implementation Notes

### Error Handling Consistency
Step-by-step methods return same error types as current methods:
- Input validation occurs before step-by-step calculation begins
- Error conditions (empty input, non-positive values, etc.) handled identically
- No new error types introduced for step-by-step functionality

### Performance
Slight overhead from storing intermediate steps, but minimal impact given human interaction timing in practice mode.

## Implementation Notes

### Human-Readable Display
Table-based method uses integer arithmetic internally (scaled_log values like 36) but displays decimal representations (like 3.6) that match pen-and-paper calculations.

### Future Extensibility
Architecture supports log-linear and other estimation methods by implementing the two traits and choosing appropriate delegation strategy based on method priorities.

The design prioritizes clear learning outcomes while maintaining code quality and testing rigor suitable for the project's educational and research goals.
