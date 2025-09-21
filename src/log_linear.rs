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

/// Converts a number to log-linear format: digit_count.remaining_digits
/// Example: 2847 -> 4.2847, 300 -> 3.3, 70 -> 2.7
fn convert_to_log_linear(value: f64) -> f64 {
    let digit_count = (value.log10().floor() as i32) + 1;
    let fractional_part = value / 10.0_f64.powi(digit_count);
    digit_count as f64 + fractional_part
}

/// Converts from log-linear format back to a number
/// Example: 3.75 -> 750, 4.1 -> 1000
/// Handles edge case: if fractional part < 0.1, treat as 0.1
fn convert_from_log_linear(log_value: f64) -> f64 {
    let digit_count = log_value.floor() as i32;
    let mut fractional_part = log_value - digit_count as f64;

    // Edge case: if fractional part is too small, use 0.1
    if fractional_part < 0.1 {
        fractional_part = 0.1;
    }

    fractional_part * 10.0_f64.powi(digit_count)
}

/// Approximates geometric mean using log-linear interpolation method
/// This pen-and-paper method converts each value to digit_count.fractional format,
/// averages them arithmetically, then converts back to get the final estimate
pub fn log_linear_approximation(values: &[f64]) -> Result<f64, GeometricMeanError> {
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

    // Calculate arithmetic mean of log-linear values
    let sum: f64 = values.iter()
        .map(|&v| convert_to_log_linear(v))
        .sum();
    let average = sum / values.len() as f64;

    // Convert back to final estimate
    Ok(convert_from_log_linear(average))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_log_linear_basic() {
        // 300 should become 3.3 (3 digits, starts with 3)
        let result = convert_to_log_linear(300.0);
        assert!((result - 3.3).abs() < 1e-10);

        // 2847 should become 4.2847
        let result = convert_to_log_linear(2847.0);
        assert!((result - 4.2847).abs() < 1e-10);

        // 70 should become 2.7
        let result = convert_to_log_linear(70.0);
        assert!((result - 2.7).abs() < 1e-10);
    }

    #[test]
    fn test_convert_from_log_linear_basic() {
        // 3.75 should become 750
        let result = convert_from_log_linear(3.75);
        assert!((result - 750.0).abs() < 1e-8);

        // 4.1 should become 1000 (4 digits starting with 1)
        let result = convert_from_log_linear(4.1);
        assert!((result - 1000.0).abs() < 1e-8);
    }

    #[test]
    fn test_convert_from_log_linear_edge_case() {
        // 4.025 should be treated as 4.1 -> 1000
        let result = convert_from_log_linear(4.025);
        assert!((result - 1000.0).abs() < 1e-8);

        // 4.0 should be treated as 4.1 -> 1000
        let result = convert_from_log_linear(4.0);
        assert!((result - 1000.0).abs() < 1e-8);
    }

    #[test]
    fn test_log_linear_approximation_readme_example() {
        // README example: [300, 10000, 900, 70] should approximate 750
        let result = log_linear_approximation(&[300.0, 10000.0, 900.0, 70.0]).unwrap();
        assert!((result - 750.0).abs() < 1e-8);
    }

    #[test]
    fn test_log_linear_approximation_edge_case_example() {
        // Edge case from README: [80, 80, 80, 800] -> [2.8, 2.8, 2.8, 3.8] -> 3.05 -> 3.1 -> 100
        let result = log_linear_approximation(&[80.0, 80.0, 80.0, 800.0]).unwrap();
        assert!((result - 100.0).abs() < 1e-8);
    }

    #[test]
    fn test_log_linear_approximation_same_digit_count() {
        // When all values have same digit count, should equal arithmetic mean
        let result = log_linear_approximation(&[100.0, 200.0, 300.0]).unwrap();
        assert!((result - 200.0).abs() < 1e-8);
    }

    #[test]
    fn test_log_linear_approximation_single_value() {
        let result = log_linear_approximation(&[500.0]).unwrap();
        assert!((result - 500.0).abs() < 1e-8);
    }

    #[test]
    fn test_log_linear_approximation_two_values() {
        // [100, 1000] should approximate sqrt(100000) ≈ 316
        let result = log_linear_approximation(&[100.0, 1000.0]).unwrap();
        let expected = (100.0_f64 * 1000.0_f64).sqrt();
        // For pen-and-paper approximation, should be within same order of magnitude
        assert!(result > expected / 10.0 && result < expected * 10.0);
    }

    #[test]
    fn test_log_linear_approximation_empty_input() {
        let result = log_linear_approximation(&[]);
        assert_eq!(result, Err(GeometricMeanError::EmptyInput));
    }

    #[test]
    fn test_log_linear_approximation_zero_value() {
        let result = log_linear_approximation(&[1.0, 0.0, 4.0]);
        assert_eq!(result, Err(GeometricMeanError::NonPositiveValue));
    }

    #[test]
    fn test_log_linear_approximation_negative_value() {
        let result = log_linear_approximation(&[1.0, -2.0, 4.0]);
        assert_eq!(result, Err(GeometricMeanError::NonPositiveValue));
    }

    #[test]
    fn test_log_linear_approximation_value_too_small() {
        let result = log_linear_approximation(&[0.5, 2.0, 4.0]);
        assert_eq!(result, Err(GeometricMeanError::ValueTooSmall));
    }

    #[test]
    fn test_log_linear_approximation_large_numbers() {
        let result = log_linear_approximation(&[1000.0, 10000.0]).unwrap();
        // This should be reasonably close to sqrt(1000 * 10000) = sqrt(10000000) ≈ 3162
        let expected = (1000.0_f64 * 10000.0_f64).sqrt();
        // For pen-and-paper approximation, should be within same order of magnitude
        assert!(result > expected / 10.0 && result < expected * 10.0);
    }
}