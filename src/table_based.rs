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

const TABLE_ENTRIES: [(f64, f64); 10] = [
    (0.0, 1.0),
    (0.1, 1.25),
    (0.2, 1.6),
    (0.3, 2.0),
    (0.4, 2.5),
    (0.5, 3.0),
    (0.6, 4.0),
    (0.7, 5.0),
    (0.8, 6.0),
    (0.9, 8.0),
];

fn find_forward_table_entry(leading_digits: f64) -> f64 {
    for i in (0..TABLE_ENTRIES.len()).rev() {
        if leading_digits >= TABLE_ENTRIES[i].1 {
            return TABLE_ENTRIES[i].0;
        }
    }
    TABLE_ENTRIES[0].0
}

fn number_to_log_representation(value: f64) -> f64 {
    let zeros = value.log10().floor() as i32;
    let leading_digits = value / 10.0_f64.powi(zeros);
    let decimal_part = find_forward_table_entry(leading_digits);
    zeros as f64 + decimal_part
}

fn find_reverse_table_entry(fractional_part: f64) -> f64 {
    // Multiply by 10 to convert 0.1 increments to integers
    let scaled = fractional_part * 10.0;

    // Choose rounding strategy: exact match vs round up
    let rounded = scaled.round();
    let raw_index = if (scaled - rounded).abs() < 1e-10 {
        rounded
    } else {
        scaled.ceil()
    };

    let index = (raw_index as usize).min(9);
    TABLE_ENTRIES[index].1
}

fn log_representation_to_number(log_value: f64) -> f64 {
    let zeros = log_value.floor() as i32;
    let fractional_part = log_value - zeros as f64;
    let multiplier = find_reverse_table_entry(fractional_part);
    multiplier * 10.0_f64.powi(zeros)
}

pub fn table_based_approximation(values: &[f64]) -> Result<f64, GeometricMeanError> {
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

    let sum: f64 = values.iter()
        .map(|&v| number_to_log_representation(v))
        .sum();
    let average = sum / values.len() as f64;

    Ok(log_representation_to_number(average))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_conversion_readme_examples() {
        let result = number_to_log_representation(2000.0);
        assert!((result - 3.3).abs() < 1e-10);

        let result = number_to_log_representation(50.0);
        assert!((result - 1.7).abs() < 1e-10);

        let result = number_to_log_representation(1250000.0);
        assert!((result - 6.1).abs() < 1e-10);

        let result = number_to_log_representation(350.0);
        assert!((result - 2.5).abs() < 1e-10);

        let result = number_to_log_representation(1400.0);
        assert!((result - 3.1).abs() < 1e-10);

        let result = number_to_log_representation(11.0);
        assert!((result - 1.0).abs() < 1e-10);

        let result = number_to_log_representation(9001.0);
        assert!((result - 3.9).abs() < 1e-10);
    }

    #[test]
    fn test_reverse_conversion_readme_examples() {
        let result = log_representation_to_number(3.6);
        assert!((result - 4000.0).abs() < 1e-6);

        let result = log_representation_to_number(2.8);
        assert!((result - 600.0).abs() < 1e-6);

        let result = log_representation_to_number(7.2);
        assert!((result - 16000000.0).abs() < 1e-6);

        let result = log_representation_to_number(4.4);
        assert!((result - 25000.0).abs() < 1e-6);

        let result = log_representation_to_number(2.333);
        assert!((result - 250.0).abs() < 1e-6);

        let result = log_representation_to_number(7.75);
        assert!((result - 60000000.0).abs() < 1e-6);

        let result = log_representation_to_number(4.167);
        assert!((result - 16000.0).abs() < 1e-6);
    }

    #[test]
    fn test_table_based_approximation_single_value() {
        let result = table_based_approximation(&[500.0]).unwrap();
        assert!((result - 500.0).abs() < 1e-6);
    }

    #[test]
    fn test_table_based_approximation_error_cases() {
        assert_eq!(table_based_approximation(&[]), Err(GeometricMeanError::EmptyInput));
        assert_eq!(table_based_approximation(&[1.0, 0.0, 4.0]), Err(GeometricMeanError::NonPositiveValue));
        assert_eq!(table_based_approximation(&[1.0, -2.0, 4.0]), Err(GeometricMeanError::NonPositiveValue));
        assert_eq!(table_based_approximation(&[0.5, 2.0, 4.0]), Err(GeometricMeanError::ValueTooSmall));
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

    mod property_tests {
        use super::*;
        use crate::geometric_mean;
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
            let result = table_based_approximation(&[x.0]).unwrap();
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

            let original_result = table_based_approximation(&original).unwrap();
            let reversed_result = table_based_approximation(&reversed).unwrap();

            let tolerance = (original_result * 1e-6).max(1e-8);
            TestResult::from_bool((original_result - reversed_result).abs() < tolerance)
        }

        #[quickcheck]
        fn prop_order_of_magnitude_correctness(values: Vec<GeOneF64>) -> TestResult {
            if values.is_empty() {
                return TestResult::discard();
            }

            let nums: Vec<f64> = values.iter().map(|x| x.0).collect();
            let approximation = table_based_approximation(&nums).unwrap();
            let exact = geometric_mean(&nums).unwrap();

            TestResult::from_bool(approximation >= exact / 10.0 && approximation <= exact * 10.0)
        }

        #[quickcheck]
        fn prop_minimum_result_bounds(values: Vec<GeOneF64>) -> TestResult {
            if values.is_empty() {
                return TestResult::discard();
            }

            let nums: Vec<f64> = values.iter().map(|x| x.0).collect();
            let result = table_based_approximation(&nums).unwrap();
            let min_val = nums.iter().cloned().fold(f64::INFINITY, f64::min);

            TestResult::from_bool(result >= min_val / 10.0)
        }

        #[quickcheck]
        fn prop_maximum_result_bounds(values: Vec<GeOneF64>) -> TestResult {
            if values.is_empty() {
                return TestResult::discard();
            }

            let nums: Vec<f64> = values.iter().map(|x| x.0).collect();
            let result = table_based_approximation(&nums).unwrap();
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

            let a_result = table_based_approximation(&a_nums).unwrap();
            let b_result = table_based_approximation(&b_nums).unwrap();

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
    }
}
