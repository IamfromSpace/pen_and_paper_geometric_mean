# Extract Exact Geometric Mean Module

## Overview

Extract the exact geometric mean implementation from `src/main.rs` into its own dedicated module `src/exact.rs`.
This follows the established pattern of having separate modules for different geometric mean approaches (log_linear, table_based, exact).

## Current State

The exact geometric mean implementation currently lives in `src/main.rs` and includes:
- Error type: `GeometricMeanError` with variants `EmptyInput` and `NonPositiveValue`
- Main function: `geometric_mean(values: &[f64]) -> Result<f64, GeometricMeanError>`
- Comprehensive test suite with unit tests and property-based tests using QuickCheck
- Formal verification tests using Kani

## Goals

1. Create a new `src/exact.rs` module containing the exact geometric mean implementation
2. Maintain all existing functionality and test coverage
3. Ensure the module follows the project's established patterns
4. Keep `src/main.rs` clean and focused on coordination between modules

## Implementation Steps

### 1. Create the New Module File

Create `src/exact.rs` with the following structure:
- Public error type `GeometricMeanError` with Display and Error trait implementations
- Public function `geometric_mean` with the exact same signature and behavior
- All existing unit tests moved to this module
- All existing property-based tests (QuickCheck) moved to this module
- All existing formal verification tests (Kani) moved to this module

### 2. Update Module Declaration

In `src/main.rs`:
- Add `mod exact;` declaration alongside existing `mod log_linear;` and `mod table_based;`
- Import the function and error type: `use exact::{geometric_mean, GeometricMeanError};`

### 3. Clean Up Main Module

Remove from `src/main.rs`:
- The `GeometricMeanError` enum definition and implementations
- The `geometric_mean` function implementation
- All test modules (`tests`, property tests, and verification tests)

Keep in `src/main.rs`:
- Module declarations
- The `main()` function
- Any re-exports needed for public API consistency

### 4. Verify Module Integration

Ensure that:
- All tests continue to pass after the refactoring
- The module can be imported and used by other parts of the codebase
- Public API remains unchanged from external perspective
- Cargo build and test commands work without modification

## Dependencies

The new module will need:
- Standard library imports for error handling (`std::fmt`, `std::error`)
- QuickCheck dependency for property-based testing (already in project)
- Kani for formal verification (already in project)

## Testing Strategy

After extraction:
- Run full test suite to ensure no regressions
- Verify that `cargo test` passes for all test types (unit, property, verification)
- Confirm that the exact geometric mean functionality works identically to before
- Test module imports work correctly

## Module Interface

The `src/exact.rs` module should expose:
```rust
pub enum GeometricMeanError {
    EmptyInput,
    NonPositiveValue,
}

pub fn geometric_mean(values: &[f64]) -> Result<f64, GeometricMeanError>
```

This maintains the same public interface while organizing the code into a dedicated module.

## Benefits

1. **Organization**: Separates exact implementation from other approximation methods
2. **Maintainability**: Each approach has its own focused module
3. **Consistency**: Follows the established pattern of `log_linear.rs` and `table_based.rs`
4. **Clarity**: Makes the main module cleaner and easier to understand
5. **Extensibility**: Makes it easier to add new geometric mean approaches in the future

## Implementation Notes

- Preserve all existing comments and documentation
- Maintain identical function signatures and behavior
- Keep all test coverage at 100%
- Ensure error handling remains exactly the same
- Follow Rust module conventions and the project's established code style