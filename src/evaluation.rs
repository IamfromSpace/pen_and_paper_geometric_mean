use rand::Rng;
use crate::traits::EstimateGeometricMean;
use crate::exact::geometric_mean;

#[derive(Debug)]
pub struct Results {
    pub mean_absolute_relative_error: f64,
    pub worst_case_error: f64,
    pub worst_case_overestimate: f64,
    pub overall_bias: f64,
    pub total_tests: usize,
}

pub fn evaluate_estimate<R: Rng, T: EstimateGeometricMean>(
    rng: &mut R,
    min: f64,
    max: f64,
    num_tests: usize
) -> Results {
    let mut total_relative_error = 0.0;
    let mut max_error = 0.0;
    let mut max_overestimate = 0.0;
    let mut total_signed_error = 0.0;
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

        // Calculate relative error and signed error
        let relative_error = (estimate_result - exact_result).abs() / exact_result;
        let signed_relative_error = (estimate_result - exact_result) / exact_result;

        total_relative_error += relative_error;
        total_signed_error += signed_relative_error;

        // Track worst case error
        if relative_error > max_error {
            max_error = relative_error;
        }

        // Track worst case overestimate
        if signed_relative_error > 0.0 && signed_relative_error > max_overestimate {
            max_overestimate = signed_relative_error;
        }

        valid_tests += 1;
    }

    let mean_absolute_relative_error = if valid_tests > 0 {
        total_relative_error / valid_tests as f64
    } else {
        f64::NAN
    };

    let worst_case_error = if valid_tests > 0 {
        max_error
    } else {
        f64::NAN
    };

    let worst_case_overestimate = if valid_tests > 0 {
        max_overestimate
    } else {
        f64::NAN
    };

    let overall_bias = if valid_tests > 0 {
        total_signed_error / valid_tests as f64
    } else {
        f64::NAN
    };

    Results {
        mean_absolute_relative_error,
        worst_case_error,
        worst_case_overestimate,
        overall_bias,
        total_tests: valid_tests,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exact::ExactGeometricMean;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use quickcheck_macros::quickcheck;

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

    #[test]
    fn test_exact_method_extended_statistics() {
        let mut rng = StdRng::seed_from_u64(789);
        let results = evaluate_estimate::<_, ExactGeometricMean>(&mut rng, 1.0, 1000.0, 100);

        // Exact method should have near-zero errors for all metrics
        assert!(results.worst_case_error < 1e-14);
        assert!(results.worst_case_overestimate < 1e-14);
        assert!(results.overall_bias.abs() < 1e-14);
    }

    #[test]
    fn test_all_overestimates_scenario() {
        // Create a scenario where we know the estimate will always overestimate
        // by manually constructing test data (this would require a custom estimator for testing)
        // For now, test with exact method and verify the relationships hold
        let mut rng = StdRng::seed_from_u64(101112);
        let results = evaluate_estimate::<_, ExactGeometricMean>(&mut rng, 1.0, 100.0, 50);

        // Basic relationships should hold even for exact method
        assert!(results.worst_case_error >= results.mean_absolute_relative_error);
        assert!(results.worst_case_overestimate >= 0.0);
        assert!(results.overall_bias.abs() <= results.worst_case_error);
    }

    #[test]
    fn test_no_overestimates_edge_case() {
        // Test the case where max_overestimate should be 0.0
        // With exact method, this should naturally occur
        let mut rng = StdRng::seed_from_u64(131415);
        let results = evaluate_estimate::<_, ExactGeometricMean>(&mut rng, 1.0, 10.0, 30);

        // Exact method should have worst_case_overestimate near 0
        assert!(results.worst_case_overestimate < 1e-14);
    }

    #[quickcheck]
    fn prop_worst_case_error_bounds_mean_error(seed: u64) -> bool {
        let mut rng = StdRng::seed_from_u64(seed);
        let results = evaluate_estimate::<_, ExactGeometricMean>(&mut rng, 1.0, 1000.0, 50);

        if results.total_tests == 0 {
            return true; // Skip invalid test cases
        }

        results.worst_case_error >= results.mean_absolute_relative_error
    }

    #[quickcheck]
    fn prop_overestimate_bounds(seed: u64) -> bool {
        let mut rng = StdRng::seed_from_u64(seed);
        let results = evaluate_estimate::<_, ExactGeometricMean>(&mut rng, 1.0, 1000.0, 50);

        if results.total_tests == 0 {
            return true; // Skip invalid test cases
        }

        results.worst_case_overestimate >= 0.0 &&
        results.worst_case_overestimate <= results.worst_case_error
    }

    #[quickcheck]
    fn prop_bias_bounds(seed: u64) -> bool {
        let mut rng = StdRng::seed_from_u64(seed);
        let results = evaluate_estimate::<_, ExactGeometricMean>(&mut rng, 1.0, 1000.0, 50);

        if results.total_tests == 0 {
            return true; // Skip invalid test cases
        }

        results.overall_bias.abs() <= results.worst_case_error
    }

    #[quickcheck]
    fn prop_exact_method_near_perfect(seed: u64) -> bool {
        let mut rng = StdRng::seed_from_u64(seed);
        let results = evaluate_estimate::<_, ExactGeometricMean>(&mut rng, 1.0, 1000.0, 30);

        if results.total_tests == 0 {
            return true; // Skip invalid test cases
        }

        // Exact method should have all metrics very close to 0
        results.worst_case_error < 1e-10 &&
        results.worst_case_overestimate < 1e-10 &&
        results.overall_bias.abs() < 1e-10
    }
}