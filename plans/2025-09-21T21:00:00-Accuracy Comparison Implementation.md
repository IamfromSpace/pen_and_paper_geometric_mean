# Accuracy Comparison Implementation Plan

## Overview

Simple accuracy comparison between the two pen-and-paper geometric mean approximation methods:
1. **Log-Linear Interpolation** - Uses digit count as logarithm proxy with linear interpolation
2. **Table-Based Approximation** - Uses memorized 10^(1/10) lookup table for logarithm conversion

Start small and build up from there.

## Core Requirements

### Comparison Framework
- Compare both methods against the exact geometric mean calculation
- Calculate basic accuracy metrics
- Simple output showing which method performs better

### Test Data Generation
- **Log-uniform distribution**: Values uniformly distributed across log scale (consistent with power law assumption)
- Various input set sizes (2-10 values)
- Fixed seed for reproducible results

### Basic Accuracy Metrics
- **Relative error**: (approximation - exact) / exact
- **Mean absolute relative error**: Average of |relative errors|
- **Success rate**: Percentage within 1 order of magnitude of exact result

## Implementation Components

### Simple Comparison Engine
Create a minimal comparison tool that:
- Generates log-uniform test data
- Computes results using all three methods (exact, log-linear, table-based)
- Calculates basic error metrics
- Outputs simple summary statistics

### Test Data Generator
Single generator function:
- Generate values uniformly across log scale between specified bounds
- Use fixed random seed for reproducibility
- Support different input set sizes

### Basic Output
Simple text output showing:
- Mean relative error for each method
- Success rate (within 1 order of magnitude) for each method
- Which method wins overall

## Expected Deliverables

### Minimal CLI Tool
A simple command that:
- Runs comparison with default parameters
- Shows basic accuracy comparison between the two methods
- Clearly indicates which method performs better overall

### Simple Integration
- Add comparison functionality to existing main.rs
- Reuse existing module structure
- Maintain current error handling patterns

## Success Criteria

### Functional Requirements
- Both approximation methods compared against exact calculation
- Basic accuracy metrics calculated correctly
- Clear determination of which method performs better

### Quality Requirements
- Consistent with existing code style and error handling
- Reproducible results with fixed random seed
- Simple, readable output