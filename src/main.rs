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
}
