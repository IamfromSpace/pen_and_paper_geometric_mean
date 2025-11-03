#[derive(Debug, PartialEq)]
pub enum GeometricMeanError {
    EmptyInput,
    NonPositiveValue,
    ValueTooSmall,
}

impl std::fmt::Display for GeometricMeanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeometricMeanError::EmptyInput => write!(f, "Cannot calculate geometric mean of empty input"),
            GeometricMeanError::NonPositiveValue => write!(f, "Geometric mean requires all positive values"),
            GeometricMeanError::ValueTooSmall => write!(f, "Values must be >= 1.0 for this pen-and-paper method"),
        }
    }
}

impl std::error::Error for GeometricMeanError {}

pub struct TableBasedApproximation;

impl crate::traits::EstimateGeometricMean for TableBasedApproximation {
    type Error = GeometricMeanError;

    fn estimate_geometric_mean(values: &[f64]) -> Result<f64, Self::Error> {
        table_based_approximation(values)
    }
}

const MULTIPLIERS: [f64; 10] = [
    1.0, 1.25, 1.6, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0
];

fn find_forward_table_entry(leading_digits: f64) -> usize {
    for i in (0..MULTIPLIERS.len()).rev() {
        if leading_digits >= MULTIPLIERS[i] {
            return i;
        }
    }
    0
}

fn number_to_log_representation(value: f64) -> i32 {
    let zeros = value.log10().floor() as i32;
    let leading_digits = value / 10.0_f64.powi(zeros);
    let table_index = find_forward_table_entry(leading_digits);
    zeros * 10 + table_index as i32
}

fn log_representation_to_number(scaled_log: i32) -> f64 {
    let zeros = scaled_log / 10;
    let fractional_index = scaled_log % 10;
    let multiplier = MULTIPLIERS[fractional_index as usize];
    multiplier * 10.0_f64.powi(zeros)
}

fn table_based_approximation(values: &[f64]) -> Result<f64, GeometricMeanError> {
    if values.is_empty() {
        return Err(GeometricMeanError::EmptyInput);
    }

    for &value in values {
        if value <= 0.0 {
            return Err(GeometricMeanError::NonPositiveValue);
        }
        if value < 1.0 {
            return Err(GeometricMeanError::ValueTooSmall);
        }
    }

    let sum: i32 = values.iter()
        .map(|&v| number_to_log_representation(v))
        .sum();
    let average = (sum + values.len() as i32 - 1) / values.len() as i32;

    Ok(log_representation_to_number(average))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_conversion_readme_examples() {
        let result = number_to_log_representation(2000.0);
        assert_eq!(result, 33);

        let result = number_to_log_representation(50.0);
        assert_eq!(result, 17);

        let result = number_to_log_representation(1250000.0);
        assert_eq!(result, 61);

        let result = number_to_log_representation(350.0);
        assert_eq!(result, 25);

        let result = number_to_log_representation(1400.0);
        assert_eq!(result, 31);

        let result = number_to_log_representation(11.0);
        assert_eq!(result, 10);

        let result = number_to_log_representation(9001.0);
        assert_eq!(result, 39);
    }

    #[test]
    fn test_reverse_conversion_readme_examples() {
        let result = log_representation_to_number(36);
        assert!((result - 4000.0).abs() < 1e-6);

        let result = log_representation_to_number(28);
        assert!((result - 600.0).abs() < 1e-6);

        let result = log_representation_to_number(72);
        assert!((result - 16000000.0).abs() < 1e-6);

        let result = log_representation_to_number(44);
        assert!((result - 25000.0).abs() < 1e-6);

        let result = log_representation_to_number(24);
        assert!((result - 250.0).abs() < 1e-6);

        let result = log_representation_to_number(78);
        assert!((result - 60000000.0).abs() < 1e-6);

        let result = log_representation_to_number(42);
        assert!((result - 16000.0).abs() < 1e-6);
    }

    #[test]
    fn test_table_based_approximation_single_value() {
        use crate::traits::EstimateGeometricMean;
        let result = TableBasedApproximation::estimate_geometric_mean(&[500.0]).unwrap();
        assert!((result - 500.0).abs() < 1e-6);
    }

    #[test]
    fn test_table_based_approximation_error_cases() {
        use crate::traits::EstimateGeometricMean;
        assert_eq!(TableBasedApproximation::estimate_geometric_mean(&[]), Err(GeometricMeanError::EmptyInput));
        assert_eq!(TableBasedApproximation::estimate_geometric_mean(&[1.0, 0.0, 4.0]), Err(GeometricMeanError::NonPositiveValue));
        assert_eq!(TableBasedApproximation::estimate_geometric_mean(&[1.0, -2.0, 4.0]), Err(GeometricMeanError::NonPositiveValue));
        assert_eq!(TableBasedApproximation::estimate_geometric_mean(&[0.5, 2.0, 4.0]), Err(GeometricMeanError::ValueTooSmall));
    }

    #[test]
    fn test_round_trip_conversion() {
        let test_values = vec![100.0, 1000.0, 2500.0, 9999.0];
        for value in test_values {
            let log_repr = number_to_log_representation(value);
            let converted_back = log_representation_to_number(log_repr);
            let relative_error = (converted_back - value).abs() / value;
            assert!(relative_error < 0.5, "Round trip failed for {}: {} -> {} -> {}", value, value, log_repr, converted_back);
        }
    }

    // Concrete rounding boundary tests from the Mathematical Property-Based Boundary Testing Plan

    #[test]
    fn test_readme_table_method_case() {
        use crate::traits::EstimateGeometricMean;
        // Example 1: README Table Method Case - tests complete pipeline with realistic trivia-like values
        let result = TableBasedApproximation::estimate_geometric_mean(&[3600.0, 920.0, 740.0]).unwrap();
        assert!((result - 1250.0).abs() < 50.0, "Expected ~1250, got {}", result);
    }

    #[test]
    fn test_exact_table_boundary() {
        use crate::traits::EstimateGeometricMean;
        // Example 2: Exact Table Boundary - tests forward conversion floor rounding at exact table entry boundary
        // 1251 has leading digit nearly 1.25, should map to table index 0 (multiplier 1.00) due to floor rounding
        let result = TableBasedApproximation::estimate_geometric_mean(&[1251.0]).unwrap();
        assert!((result - 1250.0).abs() < 50.0, "Expected ~1250, got {}", result);
    }

    #[test]
    fn test_fractional_average_forcing_ceiling() {
        use crate::traits::EstimateGeometricMean;
        // Example 3: Fractional Average Forcing Ceiling - forces reverse conversion ceiling decision
        // 9 copies of 1000 (log 3.0) + 1 copy of 8000 (log 3.9) → Average: 3.09
        // Fractional 0.09 should ceiling to 0.1, mapping to next table entry → Expected: 1250
        let input = vec![1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 8000.0];
        let result = TableBasedApproximation::estimate_geometric_mean(&input).unwrap();
        assert!((result - 1250.0).abs() < 50.0, "Expected ~1250, got {}", result);
    }

    mod property_tests {
        use super::*;
        use crate::exact::geometric_mean;
        use crate::traits::EstimateGeometricMean;
        use quickcheck::{Arbitrary, Gen, TestResult};
        use quickcheck_macros::quickcheck;

        #[derive(Clone, Debug)]
        struct GeOneF64(f64);

        impl Arbitrary for GeOneF64 {
            fn arbitrary(g: &mut Gen) -> Self {
                let value = loop {
                    let candidate = f64::arbitrary(g).abs();
                    if candidate >= 1.0 && candidate.is_finite() && candidate < 1e20 {
                        break candidate;
                    }
                };
                GeOneF64(value)
            }
        }

        #[quickcheck]
        fn prop_single_value_identity(x: GeOneF64) -> bool {
            let result = TableBasedApproximation::estimate_geometric_mean(&[x.0]).unwrap();
            let tolerance = x.0 * 0.5;
            (result - x.0).abs() < tolerance
        }

        #[quickcheck]
        fn prop_order_independence(mut values: Vec<GeOneF64>) -> TestResult {
            if values.len() < 2 {
                return TestResult::discard();
            }

            let original: Vec<f64> = values.iter().map(|x| x.0).collect();
            values.reverse();
            let reversed: Vec<f64> = values.iter().map(|x| x.0).collect();

            let original_result = TableBasedApproximation::estimate_geometric_mean(&original).unwrap();
            let reversed_result = TableBasedApproximation::estimate_geometric_mean(&reversed).unwrap();

            let tolerance = (original_result * 1e-6).max(1e-8);
            TestResult::from_bool((original_result - reversed_result).abs() < tolerance)
        }

        #[quickcheck]
        fn prop_order_of_magnitude_correctness(values: Vec<GeOneF64>) -> TestResult {
            if values.is_empty() {
                return TestResult::discard();
            }

            let nums: Vec<f64> = values.iter().map(|x| x.0).collect();
            let approximation = TableBasedApproximation::estimate_geometric_mean(&nums).unwrap();
            let exact = geometric_mean(&nums).unwrap();

            TestResult::from_bool(approximation >= exact / 10.0 && approximation <= exact * 10.0)
        }

        #[quickcheck]
        fn prop_minimum_result_bounds(values: Vec<GeOneF64>) -> TestResult {
            if values.is_empty() {
                return TestResult::discard();
            }

            let nums: Vec<f64> = values.iter().map(|x| x.0).collect();
            let result = TableBasedApproximation::estimate_geometric_mean(&nums).unwrap();
            let min_val = nums.iter().cloned().fold(f64::INFINITY, f64::min);

            TestResult::from_bool(result >= min_val / 10.0)
        }

        #[quickcheck]
        fn prop_maximum_result_bounds(values: Vec<GeOneF64>) -> TestResult {
            if values.is_empty() {
                return TestResult::discard();
            }

            let nums: Vec<f64> = values.iter().map(|x| x.0).collect();
            let result = TableBasedApproximation::estimate_geometric_mean(&nums).unwrap();
            let max_val = nums.iter().cloned().fold(0.0, f64::max);

            TestResult::from_bool(result <= max_val * 10.0)
        }

        #[quickcheck]
        fn prop_monotonicity(a_values: Vec<GeOneF64>, b_values: Vec<GeOneF64>) -> TestResult {
            if a_values.len() != b_values.len() || a_values.is_empty() {
                return TestResult::discard();
            }

            let a_nums: Vec<f64> = a_values.iter().map(|x| x.0).collect();
            let b_nums: Vec<f64> = b_values.iter().map(|x| x.0).collect();

            let all_a_le_b = a_nums.iter().zip(b_nums.iter()).all(|(a, b)| a <= b);
            if !all_a_le_b {
                return TestResult::discard();
            }

            let a_result = TableBasedApproximation::estimate_geometric_mean(&a_nums).unwrap();
            let b_result = TableBasedApproximation::estimate_geometric_mean(&b_nums).unwrap();

            let tolerance = (b_result * 0.01).max(1e-6);
            TestResult::from_bool(a_result <= b_result + tolerance)
        }

        #[quickcheck]
        fn prop_round_trip_within_tolerance(x: GeOneF64) -> bool {
            let log_repr = number_to_log_representation(x.0);
            let converted_back = log_representation_to_number(log_repr);
            let relative_error = (converted_back - x.0).abs() / x.0;
            relative_error < 1.0
        }

        // Property 1: Forward Rounding Direction Test
        // Tests that forward conversion (Number → Log) consistently rounds DOWN to table boundaries
        #[quickcheck]
        fn prop_forward_rounding_direction_primary(n: GeOneF64) -> TestResult {
            if n.0 < 8.0 {
                return TestResult::discard();
            }

            let boundary_result = TableBasedApproximation::estimate_geometric_mean(&[n.0]).unwrap();
            let above_boundary_result = TableBasedApproximation::estimate_geometric_mean(&[boundary_result + 1.0]).unwrap();

            TestResult::from_bool((boundary_result - above_boundary_result).abs() < 1e-10)
        }

        #[quickcheck]
        fn prop_forward_rounding_direction_complementary(n: GeOneF64) -> TestResult {
            if n.0 < 8.0 || n.0 > 1e15 {
                return TestResult::discard();
            }

            let boundary_result = TableBasedApproximation::estimate_geometric_mean(&[n.0]).unwrap();

            // Prevent going below minimum value 1.0
            if boundary_result <= 1.0 {
                return TestResult::discard();
            }

            // For very large numbers, subtracting 1.0 might not make a meaningful difference
            // due to floating point precision. Ensure the subtraction is meaningful.
            if (boundary_result - 1.0) == boundary_result {
                return TestResult::discard();
            }

            let below_boundary_result = TableBasedApproximation::estimate_geometric_mean(&[boundary_result - 1.0]).unwrap();

            TestResult::from_bool(boundary_result > below_boundary_result)
        }

        // Property 2: Fractional Boundary Precision Test
        // Tests the complete rounding pipeline with controlled fractional log components
        #[derive(Clone, Debug)]
        struct ValidCounts {
            n: usize,
            m: usize,
        }

        impl Arbitrary for ValidCounts {
            fn arbitrary(g: &mut Gen) -> Self {
                let n = (usize::arbitrary(g) % 10) + 1; // 1-10
                let m = (usize::arbitrary(g) % 10) + 1; // 1-10
                ValidCounts { n, m }
            }
        }

        #[quickcheck]
        fn prop_fractional_boundary_precision(x: GeOneF64, counts: ValidCounts) -> TestResult {
            if x.0 < 1.0 || x.0 >= 1e18 {
                return TestResult::discard();
            }

            let base = x.0;
            let high_value = base * 10.0;

            // Create mixed array: n copies of high_value, (m+1) copies of base
            let mut mixed_array = vec![high_value; counts.n];
            mixed_array.extend(vec![base; counts.m + 1]);

            // Create pure high value array
            let pure_high_array = vec![high_value];

            let mixed_result = TableBasedApproximation::estimate_geometric_mean(&mixed_array).unwrap();
            let pure_high_result = TableBasedApproximation::estimate_geometric_mean(&pure_high_array).unwrap();

            // The mixed array average should be less than or equal to the pure high value
            // This tests the complete rounding pipeline with random boundary conditions
            let tolerance = (pure_high_result * 0.01).max(1e-6);
            TestResult::from_bool(mixed_result <= pure_high_result + tolerance)
        }
    }
}
