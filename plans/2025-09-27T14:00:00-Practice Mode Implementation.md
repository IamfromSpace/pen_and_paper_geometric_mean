# Practice Mode Implementation

## Overview

Practice mode provides an interactive CLI tool for users to practice estimating geometric means using the table-based pen-and-paper method. Users are presented with realistic trivia-style guess sets, time their responses, and receive feedback on accuracy.

## Requirements

### Core Functionality
- Generate realistic practice problems using existing `TriviaGuessDistribution`
- Time user responses from problem presentation to answer submission
- Evaluate answers against exact geometric mean with defined tolerance
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
- **Answer range**: 10 to 100,000,000,000 (covers typical trivia magnitudes)
- **Method**: Table-based only (log-linear reserved for future iteration)

### Problem Generation
- Correct answers uniformly distributed in log space within range
- Problems generated using existing `TriviaGuessDistribution` with fixed parameters
- Each problem guaranteed to be solvable by table method

### Success Criteria
- **Correct**: User answer matches the estimation method result
- **Excellent**: User answer closer to exact geometric mean than estimation method result
- Both conditions checked and reported independently
- If the estimation and exact calculations are identical, report **Correct** instead of **Excellent**
- Incorrect calculations end the practice problem and present the correct estimation result

### Timing
- Timer starts immediately when problem is displayed
- Timer stops when valid answer submitted
- Display precision to 0.1 seconds

## Architecture

### Core Logic Separation
Practice mode splits into two layers to enable testing and future UI flexibility:

- **`src/practice_mode.rs`**: Pure state machine with no I/O dependencies
- **`src/cli/practice_mode.rs`**: CLI-specific rendering and input handling

### Type-Safe State Pattern
Core logic uses type states to enforce correct method call ordering:

- **Type states**: `Ready` and `Active` prevent wrong method sequences
- **Generic over estimation method**: Struct-level generic enables different methods while maintaining type safety
- **Method flow**: `new()` → `start(config)` → `submit_answer()`
- **Pure dependencies**: RNG and Timer injected at creation
- **Dynamic config**: Problem parameters provided at start

### Testable Time Dependencies
Abstract timing through trait to enable deterministic testing:

- **Timer trait**: Monotonic time measurement with associated Instant type
- **SystemTimer**: Production implementation using `std::time::Instant`
- **MockTimer**: Test implementation with predictable, incrementing durations
- **Validation**: Tests can verify correct instant usage and timing calculations

### CLI Layer Responsibilities
- Create practice session with pure dependencies (RNG, SystemTimer, TableBasedApproximation)
- Call `start()` with dynamic configuration for each problem
- Display problem guesses from active problem
- Prompt for user input and parse integers
- Display results returned from `submit_answer()`
- Formatting concerns, like rounding
- Handle continue/exit logic and session recreation
- Manage I/O operations (stdin/stdout) - timing handled by core logic via Timer trait

### Testing Benefits
- **Pure functions**: Test complete practice flows without any I/O mocking
- **Type safety**: Wrong method call orders prevented at compile time
- **Isolated logic**: Core calculations completely separate from I/O
- **UI independence**: Core logic works with any interface (CLI, GUI, web, etc.)

### Error Handling
- Invalid input: CLI layer validates and reprompts before calling core methods
- Calculation failures: Returned as error results from core methods
- Usage errors: Impossible due to type safety and consuming ownership

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
Minimal testing since core logic is separate:

- **Input validation**: Valid integers (1, 42, 1000000) parse correctly; invalid inputs ("abc", "-5", "1.5", "") rejected with clear error messages
  - **Property Test**: All integers converted to strings are parsed without error.

## Implementation Notes

### Existing Code Reuse
- `TriviaGuessDistribution` for realistic guess generation in `start()` method
- `TableBasedApproximation` trait for method comparison in `submit_answer()`
- `geometric_mean()` function for exact calculations in evaluation logic
- Pure dependencies (RNG) injected at creation, dynamic config at `start()`
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
