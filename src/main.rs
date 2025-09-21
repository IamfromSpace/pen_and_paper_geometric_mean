mod log_linear;

#[derive(Debug, PartialEq)]
pub enum GeometricMeanError {
    EmptyInput,
    NonPositiveValue,
}

impl std::fmt::Display for GeometricMeanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeometricMeanError::EmptyInput => write!(f, "Cannot calculate geometric mean of empty input"),
            GeometricMeanError::NonPositiveValue => write!(f, "Geometric mean requires all positive values"),
        }
    }
}

impl std::error::Error for GeometricMeanError {}

pub fn geometric_mean(values: &[f64]) -> Result<f64, GeometricMeanError> {
    if values.is_empty() {
        return Err(GeometricMeanError::EmptyInput);
    }

    for &value in values {
        if value <= 0.0 {
            return Err(GeometricMeanError::NonPositiveValue);
        }
    }

    let log_sum: f64 = values.iter().map(|&x| x.ln()).sum();
    let log_mean = log_sum / values.len() as f64;
    Ok(log_mean.exp())
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometric_mean_basic() {
        let result = geometric_mean(&[1.0, 4.0]).unwrap();
        assert!((result - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_geometric_mean_multiple_values() {
        let result = geometric_mean(&[2.0, 8.0]).unwrap();
        assert!((result - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_geometric_mean_three_values() {
        let result = geometric_mean(&[1.0, 2.0, 4.0]).unwrap();
        assert!((result - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_geometric_mean_single_value() {
        let result = geometric_mean(&[5.0]).unwrap();
        assert!((result - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_geometric_mean_empty_input() {
        let result = geometric_mean(&[]);
        assert_eq!(result, Err(GeometricMeanError::EmptyInput));
    }

    #[test]
    fn test_geometric_mean_zero_value() {
        let result = geometric_mean(&[1.0, 0.0, 4.0]);
        assert_eq!(result, Err(GeometricMeanError::NonPositiveValue));
    }

    #[test]
    fn test_geometric_mean_negative_value() {
        let result = geometric_mean(&[1.0, -2.0, 4.0]);
        assert_eq!(result, Err(GeometricMeanError::NonPositiveValue));
    }

    #[test]
    fn test_geometric_mean_large_numbers() {
        let result = geometric_mean(&[100.0, 10000.0]).unwrap();
        assert!((result - 1000.0).abs() < 1e-8);
    }

    #[test]
    fn test_geometric_mean_small_numbers() {
        let result = geometric_mean(&[0.1, 0.01]).unwrap();
        assert!((result - 0.031622776601683795).abs() < 1e-10);
    }

    #[test]
    fn test_geometric_mean_power_law_example() {
        let result = geometric_mean(&[10.0, 10.0, 10.0, 100000.0]).unwrap();
        assert!((result - 100.0).abs() < 1e-8);
    }

    mod property_tests {
        use super::*;
        use quickcheck::{Arbitrary, Gen, TestResult};
        use quickcheck_macros::quickcheck;

        #[derive(Clone, Debug)]
        struct PositiveF64(f64);

        impl Arbitrary for PositiveF64 {
            fn arbitrary(g: &mut Gen) -> Self {
                let value = loop {
                    let candidate = f64::arbitrary(g).abs();
                    if candidate > 1e-100 && candidate.is_finite() && candidate < 1e100 {
                        break candidate;
                    }
                };
                PositiveF64(value)
            }
        }

        #[quickcheck]
        fn prop_single_value_identity(x: PositiveF64) -> bool {
            let result = geometric_mean(&[x.0]).unwrap();
            let tolerance = (x.0 * 1e-12).max(1e-14);
            (result - x.0).abs() < tolerance
        }

        #[quickcheck]
        fn prop_multiplicative_scaling(values: Vec<PositiveF64>, scale: PositiveF64) -> TestResult {
            if values.is_empty() || scale.0 <= 0.0 {
                return TestResult::discard();
            }

            let original: Vec<f64> = values.iter().map(|x| x.0).collect();
            let scaled: Vec<f64> = original.iter().map(|&x| x * scale.0).collect();

            let original_mean = geometric_mean(&original).unwrap();
            let scaled_mean = geometric_mean(&scaled).unwrap();
            let expected = original_mean * scale.0;

            let tolerance = (expected * 1e-12).max(1e-14);
            TestResult::from_bool((scaled_mean - expected).abs() < tolerance)
        }

        #[quickcheck]
        fn prop_order_independence(mut values: Vec<PositiveF64>) -> TestResult {
            if values.len() < 2 {
                return TestResult::discard();
            }

            let original: Vec<f64> = values.iter().map(|x| x.0).collect();
            values.reverse();
            let reversed: Vec<f64> = values.iter().map(|x| x.0).collect();

            let original_mean = geometric_mean(&original).unwrap();
            let reversed_mean = geometric_mean(&reversed).unwrap();

            let tolerance = (original_mean * 1e-12).max(1e-14);
            TestResult::from_bool((original_mean - reversed_mean).abs() < tolerance)
        }

        #[quickcheck]
        fn prop_subset_bounds(values: Vec<PositiveF64>) -> TestResult {
            if values.is_empty() {
                return TestResult::discard();
            }

            let nums: Vec<f64> = values.iter().map(|x| x.0).collect();
            let result = geometric_mean(&nums).unwrap();
            let min_val = nums.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_val = nums.iter().cloned().fold(0.0, f64::max);

            let tolerance = (result * 1e-12).max(1e-14);
            TestResult::from_bool(result >= min_val - tolerance && result <= max_val + tolerance)
        }

        #[quickcheck]
        fn prop_arithmetic_geometric_inequality(values: Vec<PositiveF64>) -> TestResult {
            if values.is_empty() {
                return TestResult::discard();
            }

            let nums: Vec<f64> = values.iter().map(|x| x.0).collect();
            let geometric = geometric_mean(&nums).unwrap();
            let arithmetic = nums.iter().sum::<f64>() / nums.len() as f64;

            let tolerance = (arithmetic * 1e-12).max(1e-14);
            TestResult::from_bool(geometric <= arithmetic + tolerance)
        }

        #[quickcheck]
        fn prop_monotonicity(a_values: Vec<PositiveF64>, b_values: Vec<PositiveF64>) -> TestResult {
            if a_values.len() != b_values.len() || a_values.is_empty() {
                return TestResult::discard();
            }

            let a_nums: Vec<f64> = a_values.iter().map(|x| x.0).collect();
            let b_nums: Vec<f64> = b_values.iter().map(|x| x.0).collect();

            let all_a_le_b = a_nums.iter().zip(b_nums.iter()).all(|(a, b)| a <= b);
            if !all_a_le_b {
                return TestResult::discard();
            }

            let a_mean = geometric_mean(&a_nums).unwrap();
            let b_mean = geometric_mean(&b_nums).unwrap();

            let tolerance = (b_mean * 1e-12).max(1e-14);
            TestResult::from_bool(a_mean <= b_mean + tolerance)
        }

        #[quickcheck]
        fn prop_duplicates_same_result(x: PositiveF64) -> bool {
            let single = geometric_mean(&[x.0]).unwrap();
            let duplicated = geometric_mean(&[x.0, x.0, x.0, x.0]).unwrap();
            let tolerance = (single * 1e-12).max(1e-14);
            (single - duplicated).abs() < tolerance
        }

        #[quickcheck]
        fn prop_two_value_formula(a: PositiveF64, b: PositiveF64) -> bool {
            let result = geometric_mean(&[a.0, b.0]).unwrap();
            let expected = (a.0 * b.0).sqrt();
            let tolerance = (expected * 1e-12).max(1e-14);
            (result - expected).abs() < tolerance
        }
    }
}
