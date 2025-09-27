# Practice Mode Implementation

## Overview

Practice mode provides an interactive CLI tool for users to practice estimating geometric means using the table-based pen-and-paper method. Users are presented with realistic trivia-style guess sets, time their responses, and receive feedback on accuracy.

## Requirements

### Core Functionality
- Generate realistic practice problems using pre-configured `TriviaGuessDistribution`
- Time user responses from problem presentation to answer submission
- Evaluate answers: Correct if user answer equals floor(estimation_method_result) or ceiling(estimation_method_result)
- Compare user performance to table-based approximation method
- Loop indefinitely until user chooses to exit

### User Interface
```
Practice Mode - Table-Based Geometric Mean
=========================================

Here are the team's guesses:
  1. 150
  2. 2,500
  3. 800
  4. 45

Enter your estimated geometric mean: 420

Results:
========
Your answer: 420
Exact geometric mean: 387.4
Estimation method result: 400.0
Time taken: 12.3 seconds

★ EXCELLENT! Your answer is closer to the exact value than the estimation method!

Continue with another problem? (y/n): y

Here are the team's guesses:
  1. 180
  2. 320
  3. 550
  4. 850

Enter your estimated geometric mean: 410

Results:
========
Your answer: 410
Exact geometric mean: 417.3
Estimation method result: 400.0
Time taken: 5.1 seconds

✓ CORRECT! You calculated the estimation method properly.

Continue with another problem? (y/n): y

Here are the team's guesses:
  1. 25
  2. 400
  3. 1,200
  4. 8000

Enter your estimated geometric mean: 2000

Results:
========
Your answer: 2,000
Exact geometric mean: 346.4
Estimation method result: 400.0
Time taken: 8.7 seconds

You have calculated the estimation method incorrectly.

Continue with another problem? (y/n): n

Thanks for practicing!
```

### CLI Integration
- First command-line argument "practice" enters practice mode
- No arguments runs existing comparison behavior (renamed to `compare()`)
- Simple argument parsing without external dependencies

## Design Decisions

### Fixed CLI Parameters (For Simplicity)
- **Team size**: 4 guessers (realistic trivia scenario)
- **Uncertainty**: log_std_dev = 4.0 (moderate spread matching existing tests)
- **Answer range**: 10 to 1,000,000,000 (1B max to avoid extremely large guesses)
- **Method**: Table-based only (log-linear reserved for future iteration)

### Problem Generation
- **Correct answers**: Randomly chosen in log space (uniform distribution over ln(10) to ln(1,000,000,000))
- **Distribution creation**: Each problem creates new `TriviaGuessDistribution` with randomly chosen center point
- **Return format**: `start()` method returns tuple of `(guesses: Vec<u64>, active_session)` for immediate access to problem data

### Answer Evaluation
Answer evaluation results expressed as enum with three distinct states, using the following precedence:

1. **Correct**: User answer equals floor(estimation_method_result) or ceiling(estimation_method_result)
2. **Excellent**: User answer is within the "excellent range" around the exact geometric mean
3. **Incorrect**: User answer does not meet either above criteria

#### Evaluation Logic
```
error_margin = |estimation_result - exact_geometric_mean|
excellent_range = [exact_geometric_mean - error_margin, exact_geometric_mean + error_margin]

if user_answer == floor(estimation_result) OR user_answer == ceil(estimation_result):
    return Correct
else if user_answer is within excellent_range:
    return Excellent
else:
    return Incorrect
```

#### Evaluation Examples
Test cases demonstrating the evaluation logic:

**Case 1: Estimate: 100.5, Exact: 98.5**
- Error margin = |100.5 - 98.5| = 2.0
- Excellent range = [96.5, 100.5]
- floor(100.5) = 100, ceil(100.5) = 101

| User Answer | floor/ceil Match? | In Excellent Range? | Evaluation | Reasoning |
|-------------|-------------------|-------------------|------------|-----------|
| 100 | ✓ (floor) | ✓ | **Correct** | floor/ceil takes precedence |
| 101 | ✓ (ceil) | ✗ | **Correct** | floor/ceil takes precedence |
| 102 | ✗ | ✗ | **Incorrect** | Outside both criteria |
| 99 | ✗ | ✓ | **Excellent** | Within excellent range |
| 98 | ✗ | ✓ | **Excellent** | Within excellent range |
| 97 | ✗ | ✓ | **Excellent** | Within excellent range |
| 96 | ✗ | ✗ | **Incorrect** | Below excellent range |

**Case 2: Estimate: 150.0, Exact: 50.0**
- Error margin = |150.0 - 50.0| = 100.0
- Excellent range = [-50.0, 150.0]
- floor(150.0) = 150, ceil(150.0) = 150

| User Answer | floor/ceil Match? | In Excellent Range? | Evaluation | Reasoning |
|-------------|-------------------|-------------------|------------|-----------|
| 150 | ✓ | ✓ | **Correct** | Matches floor/ceil |
| 100 | ✗ | ✓ | **Excellent** | User surpassed estimation method |
| 200 | ✗ | ✗ | **Incorrect** | Outside excellent range |

**Case 3: Precision Edge Case - Estimate: 100.000000002, Exact: 98.5**
- Error margin = |100.000000002 - 98.5| ≈ 1.500000002
- Excellent range ≈ [97.0, 100.0]
- floor(100.000000002) = 100, ceil(100.000000002) = 101

| User Answer | floor/ceil Match? | In Excellent Range? | Evaluation | Reasoning |
|-------------|-------------------|-------------------|------------|-----------|
| 100 | ✓ (floor) | ✓ | **Correct** | Matches meaningful floor |
| 101 | ✓ (ceil) | ✗ | **Correct** | Floating point precision artifact |
| 99 | ✗ | ✓ | **Excellent** | Within excellent range |

**Case 4: Estimate: 100.0, Exact: 98.0**
- Error margin = |100.0 - 98.0| = 2.0
- Excellent range = [96.0, 100.0]
- floor(100.0) = 100, ceil(100.0) = 100

| User Answer | floor/ceil Match? | In Excellent Range? | Evaluation | Reasoning |
|-------------|-------------------|-------------------|------------|-----------|
| 100 | ✓ | ✓ | **Correct** | Matches floor/ceil exactly |
| 96 | ✗ | ✗ | **Incorrect** | At boundary but doesn't match floor/ceil |

**Case 5: Estimate: 100.5, Exact: 99.5**
- Error margin = |100.5 - 99.5| = 1.0
- Excellent range = [98.5, 100.5]
- floor(100.5) = 100, ceil(100.5) = 101

| User Answer | floor/ceil Match? | In Excellent Range? | Evaluation | Reasoning |
|-------------|-------------------|-------------------|------------|-----------|
| 100 | ✓ (floor) | ✓ | **Correct** | Matches floor, estimation was reasonable |

#### Implementation Notes
- **Floating point precision**: Use appropriate epsilon for floor/ceil comparisons to avoid precision artifacts
- **Range boundaries**: Excellent range uses inclusive boundaries
- **Precedence**: floor/ceil matching always takes precedence over excellent range checking

### Timing Requirements
- **Duration type**: Use `std::time::Duration` throughout instead of raw seconds as `f64`
- **Timer precision**: Maintain nanosecond precision internally, display to 0.1 seconds for user
- **Timer lifecycle**: Start when problem displayed, stop when valid answer submitted

## Architecture

### Core Logic Separation
Practice mode splits into two layers to enable testing and future UI flexibility:

- **`src/practice_mode.rs`**: Pure state machine with no I/O dependencies
- **`src/cli/practice_mode.rs`**: CLI-specific rendering and input handling with dedicated formatting functions

### Type-Safe State Pattern
Core logic uses type states to enforce correct method call ordering:

- **Type states**: `Ready` and `Active` prevent wrong method sequences
- **Generic over estimation method**: Struct-level generic enables different methods while maintaining type safety
- **Method flow**: `new()` → `start(config)` → `submit_answer()`
- **Public API**: Only `new()`, `start()`, and `submit_answer()` methods are public
- **Pure dependencies**: RNG and Timer injected at creation
- **Random center selection**: `start()` randomly chooses correct answer in log space, then creates distribution

### Testable Time Dependencies
Abstract timing through trait to enable deterministic testing:

- **Timer trait**: Monotonic time measurement with associated Instant type, returns `Duration` for all time calculations
- **SystemTimer**: Production implementation using `std::time::Instant` and `std::time::Duration`
- **MockTimer**: Test-only implementation with predictable, incrementing durations (not public API)
- **Validation**: Tests can verify correct instant usage and timing calculations

### CLI Layer Responsibilities
- Create practice session with pure dependencies (RNG, SystemTimer, TableBasedApproximation)
- Call `start()` with configuration for each problem
- Extract and display problem guesses from returned tuple using dedicated formatting function
- Prompt for user input and parse as `u64` (no floating point parsing needed)
- Display results using dedicated formatting function with Duration formatting
- Handle continue/exit logic and session recreation
- Manage I/O operations (stdin/stdout) - timing handled by core logic via Timer trait

### CLI Formatting Functions
- **`format_problem_display(guesses: &[u64]) -> String`**: Pure function for consistent guess presentation
- **`format_results_display(user_answer: u64, exact_mean: f64, estimation_result: u64, duration: Duration, evaluation: AnswerEvaluation) -> String`**: Pure function for consistent result presentation

### Testing Benefits
- **Pure functions**: Test complete practice flows without any I/O mocking
- **Type safety**: Wrong method call orders prevented at compile time
- **Isolated logic**: Core calculations completely separate from I/O
- **UI independence**: Core logic works with any interface (CLI, GUI, web, etc.)

### Error Handling

#### Configuration Errors (Distinct enum variants)
Configuration validation should provide specific error types:
- **ZeroTeamSize**: Team size cannot be zero
- **InvalidAnswerRange**: Answer range cannot be empty (min >= max)

#### Core Logic Errors
Core logic returns simple Result types without composite RuntimeError enum:
- **start()**: Returns `Result<(Vec<u64>, ActiveSession), ConfigurationError>`
- **Estimation/Distribution failures**: Handled internally, should not occur with valid configuration
- **Type safety**: Usage errors prevented by state machine design and consuming ownership

#### Input Processing
- **CLI layer responsibility**: Validate and reprompt for user input before calling core methods
- **Core layer responsibility**: Return structured errors only for configuration validation
- **No composite errors**: Keep error handling simple - complex error composition is CLI concern

## Testing Strategy

### Core Logic Testing Requirements
State machine testing without I/O dependencies:

- **Correct answer flow**: Create session with known RNG seed, start problem, calculate estimation method result for generated guesses, submit that result, verify marked as correct
- **Excellent answer detection**: Generate problem where user answer is closer to exact geometric mean than estimation method result, verify marked as excellent
- **Timer validation**: Use MockTimer, verify submit_answer uses the specific instant returned by start call (not a different instant)
- **State transitions**: Verify calling methods in wrong order (submit before start, start twice, etc.) prevented by type system
- **Multiple sessions**: Create multiple sessions with same MockTimer, verify each gets unique timing values

### Property-Based Testing Focus
Mathematical properties that must hold regardless of implementation:

**Injection**: Use a mock EstimateGeometricMean impl (sum all results) to get simple predictable results without re-implementation of core logic, and we can test properties rather than examples.

- **Estimation method integration**:
  - The sum always returns **Correct**
  - The sum minus one always returns **Excellent**
- **State isolation**: Multiple practice sessions don't interfere with each other's state or timing

### CLI Layer Testing
Focus on pure formatting functions and input validation:

- **Formatting functions**: Test exact string output for `format_problem_display()` and `format_results_display()`
- **Input validation**: Valid integers (1, 42, 1000000) parse correctly; invalid inputs ("abc", "-5", "1.5", "") rejected with clear error messages
  - **Property Test**: All integers converted to strings are parsed without error.

## Implementation Notes

### Existing Code Reuse
- `TriviaGuessDistribution` for realistic guess generation in `start()` method (generates `u64` values directly)
- `TableBasedApproximation` trait for method comparison in `submit_answer()`
- `geometric_mean()` function for exact calculations in evaluation logic (convert `u64` guesses to `f64` only for calculation)
- Pure dependencies (RNG, Timer) injected at creation, range-based config at `start()`
- Core logic remains pure, I/O handling isolated to CLI layer

### Future Extensibility
- Type state design supports any UI framework (iced-rs, web, etc.)
- Adding log-linear method requires only new estimation method dependency at creation
- CLI layer remains concrete (TableBasedApproximation) while core supports any method
- Statistics tracking can be added to state markers without affecting CLI layer
- Step-by-step explanations can be added to result types
- Pure dependencies stay in `new()`, dynamic config in `start()` for flexibility

### Implementation Phases
1. **Core type states** (`src/practice_mode.rs`) - Type-safe method chains, fully testable
2. **CLI integration** (`src/cli/practice_mode.rs`) - Thin I/O wrapper around core methods
3. **Main function routing** - Argument parsing and dispatch

## Success Criteria

Implementation complete when:
1. `cargo run practice` enters practice mode with specified UI
2. `cargo run` (no args) runs existing comparison behavior
3. All mathematical calculations verified correct
4. Property tests validate core behaviors
5. User experience smooth for typical practice sessions

The goal is a simple, robust foundation that provides immediate value while remaining extensible for future enhancements.
