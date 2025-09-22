use rand::Rng;
use crate::traits::EstimateGeometricMean;
use crate::exact::geometric_mean;

#[derive(Debug)]
pub struct Results {
    pub mean_absolute_relative_error: f64,
    pub total_tests: usize,
}

pub fn evaluate_estimate<R: Rng, T: EstimateGeometricMean>(
    rng: &mut R,
    min: f64,
    max: f64,
    num_tests: usize
) -> Results {
    let mut total_relative_error = 0.0;
    let mut valid_tests = 0;

    for _ in 0..num_tests {
        // Generate log-uniform distributed test case size
        let test_size = rng.gen_range(1..=10);

        // Generate log-uniform distributed values
        let mut test_values = Vec::with_capacity(test_size);

        for _ in 0..test_size {
            let log_min = min.ln();
            let log_max = max.ln();
            let log_value = rng.gen_range(log_min..=log_max);
            let value = log_value.exp();

            test_values.push(value);
        }

        // Calculate exact geometric mean
        let exact_result = match geometric_mean(&test_values) {
            Ok(result) => result,
            Err(_) => continue, // Skip invalid test cases
        };

        // Calculate estimate
        let estimate_result = match T::estimate_geometric_mean(&test_values) {
            Ok(result) => result,
            Err(_) => continue, // Skip test cases that the estimator can't handle
        };

        // Calculate relative error
        let relative_error = (estimate_result - exact_result).abs() / exact_result;
        total_relative_error += relative_error;
        valid_tests += 1;
    }

    let mean_absolute_relative_error = if valid_tests > 0 {
        total_relative_error / valid_tests as f64
    } else {
        f64::NAN
    };

    Results {
        mean_absolute_relative_error,
        total_tests: valid_tests,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exact::ExactGeometricMean;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_exact_method_perfect_score() {
        let mut rng = StdRng::seed_from_u64(42);
        let results = evaluate_estimate::<_, ExactGeometricMean>(&mut rng, 1.0, 1000.0, 100);

        // Exact method should have zero error (within floating point precision)
        assert!(results.mean_absolute_relative_error < 1e-14);
        assert!(results.total_tests > 0);
    }

    #[test]
    fn test_evaluation_returns_valid_results() {
        let mut rng = StdRng::seed_from_u64(123);
        let results = evaluate_estimate::<_, ExactGeometricMean>(&mut rng, 1.0, 100.0, 50);

        assert!(results.total_tests > 0);
        assert!(results.mean_absolute_relative_error.is_finite());
        assert!(results.mean_absolute_relative_error >= 0.0);
    }

    #[test]
    fn test_evaluation_handles_edge_range() {
        let mut rng = StdRng::seed_from_u64(456);
        let results = evaluate_estimate::<_, ExactGeometricMean>(&mut rng, 1.0, 2.0, 20);

        assert!(results.total_tests > 0);
        assert!(results.mean_absolute_relative_error < 1e-14);
    }
}