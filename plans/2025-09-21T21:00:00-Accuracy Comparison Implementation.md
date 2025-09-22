# Method Evaluation Implementation Plan

## Overview

Simple accuracy evaluation for pen-and-paper geometric mean approximation methods:
1. **Log-Linear Interpolation** - Uses digit count as logarithm proxy with linear interpolation
2. **Table-Based Approximation** - Uses memorized 10^(1/10) lookup table for logarithm conversion

Start with evaluation foundation first, comparison can be built on top later.

## Core Requirements

### Evaluation Framework
- Evaluate each method against the exact geometric mean calculation
- Calculate basic accuracy metrics for each method independently
- Simple output showing metrics for each method

### Test Data Generation
- **Log-uniform distribution**: Values uniformly distributed across log scale (consistent with power law assumption)
- Various input set sizes (2-10 values)
- Fixed seed for reproducible results

### Basic Accuracy Metrics
- **Relative error**: (approximation - exact) / exact
- **Mean absolute relative error**: Average of |relative errors|
- **Success rate**: Percentage within 1 order of magnitude of exact result

## Implementation Strategy

### First Commit: Trait Foundation
Create the trait interface and implement for all methods:
- **EstimateGeometricMean trait** - Common interface for all methods
- **Trait implementations** - All three methods implement the trait (exact, log-linear, table-based)
- **Basic testing** - Verify trait implementations work correctly
- **Keep existing APIs** - Don't break current function interfaces

### Second Commit: Generic Evaluation Framework
Build evaluation system on top of trait foundation:
- **Generic evaluator** - Accepts any trait implementer
- **Test data generator** - Log-uniform distribution generator
- **Accuracy metrics** - Calculate error statistics against exact method
- **Print Stats from main** - Simplest possible execution and output of our evaluator

### Test Data Generator
Single generator function:
- Generate values uniformly across log scale between specified bounds
- Use fixed random seed for reproducibility
- Support different input set sizes

### Basic Output
Simple text output showing for each method:
- Mean relative error
- Success rate (within 1 order of magnitude)
- Basic summary statistics

## Expected Deliverables

### Simple Integration
- Add evaluation functionality to existing main.rs
- Reuse existing module structure
- Maintain current error handling patterns

## Success Criteria

### Functional Requirements
- Generic trait-based evaluation framework
- All three methods implement EstimateGeometricMean trait
- Exact method evaluation shows near-zero error (validation test)
- Basic accuracy metrics calculated correctly for any trait implementer
- Foundation ready for later comparison between multiple methods

### Quality Requirements
- Consistent with existing code style and error handling
- Reproducible results with fixed random seed
- Simple, readable output
