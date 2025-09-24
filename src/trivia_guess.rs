use rand::distributions::Distribution;
use rand::Rng;
use std::error::Error;
use std::fmt;

/// Errors that can occur when constructing a TriviaGuessDistribution
#[derive(Debug, PartialEq)]
pub enum TriviaGuessDistributionError {
    InvalidCorrectAnswer,
    InvalidLogStdDev,
    LogStdDevTooLarge,
}

impl fmt::Display for TriviaGuessDistributionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TriviaGuessDistributionError::InvalidCorrectAnswer => {
                write!(f, "correct_answer must be greater than 0")
            }
            TriviaGuessDistributionError::InvalidLogStdDev => {
                write!(f, "log_std_dev must be finite and non-negative")
            }
            TriviaGuessDistributionError::LogStdDevTooLarge => {
                write!(f, "log_std_dev must be <= 50.0 to prevent floating point overflow")
            }
        }
    }
}

impl Error for TriviaGuessDistributionError {}

/// A distribution that generates realistic trivia-style number guesses using a log-normal
/// distribution around the correct answer, with trivia-appropriate rounding rules.
///
/// This distribution models how humans actually guess in trivia scenarios - clustering around
/// the correct answer with log-normal uncertainty and using round numbers with different
/// precision rules based on magnitude.
#[derive(Debug, Clone, PartialEq)]
pub struct TriviaGuessDistribution {
    /// The true answer that guesses should cluster around
    correct_answer: u64,
    /// Natural logarithm of the correct answer (cached for performance)
    ln_correct_answer: f64,
    /// Standard deviation in the natural logarithmic domain
    log_std_dev: f64,
}

impl TriviaGuessDistribution {
    /// Creates a new trivia guess distribution.
    ///
    /// # Parameters
    ///
    /// * `correct_answer`: The true answer that human guesses should cluster around
    /// * `log_std_dev`: Standard deviation in the natural logarithmic domain (ln),
    ///   representing uncertainty in orders of magnitude
    ///
    /// # Uncertainty Factor Interpretation
    ///
    /// * `log_std_dev = 0.0`: Perfect certainty - always returns the correct answer rounded to valid trivia format
    /// * `log_std_dev = 0.5`: Guesses span roughly ±1.6× the correct answer
    /// * `log_std_dev = 1.0`: Guesses span roughly ±2.7× the correct answer
    /// * `log_std_dev = 1.5`: Guesses span roughly ±4.5× the correct answer
    ///
    /// # Errors
    ///
    /// Returns `InvalidCorrectAnswer` if `correct_answer` is 0.
    /// Returns `InvalidLogStdDev` if `log_std_dev` is negative, NaN, or infinite.
    /// Returns `LogStdDevTooLarge` if `log_std_dev` > 50.0.
    pub fn new(correct_answer: u64, log_std_dev: f64) -> Result<Self, TriviaGuessDistributionError> {
        if correct_answer == 0 {
            return Err(TriviaGuessDistributionError::InvalidCorrectAnswer);
        }

        if !log_std_dev.is_finite() || log_std_dev < 0.0 {
            return Err(TriviaGuessDistributionError::InvalidLogStdDev);
        }

        if log_std_dev > 50.0 {
            return Err(TriviaGuessDistributionError::LogStdDevTooLarge);
        }

        let ln_correct_answer = (correct_answer as f64).ln();

        Ok(TriviaGuessDistribution {
            correct_answer,
            ln_correct_answer,
            log_std_dev,
        })
    }

    /// Round a raw floating-point value to a trivia-realistic integer using logarithmic domain rounding.
    ///
    /// This implements the O(1) bracketing algorithm described in the plan:
    /// 1. Determine the rounding rule based on the first digit
    /// 2. Use linear bracketing to find the two nearest valid candidates
    /// 3. Choose the candidate with smaller logarithmic distance
    fn round_to_trivia_value(&self, raw_value: f64) -> u64 {
        if raw_value <= 1.0 {
            return 1;
        }

        // Determine magnitude and first digit
        let log10_value = raw_value.log10();
        let magnitude = log10_value.floor() as i32;

        // Handle edge cases for very large or very small values
        if magnitude < 0 {
            return 1;
        }
        if magnitude > 18 {  // 10^18 is close to u64 max
            return u64::MAX;
        }

        let magnitude_power = 10_u64.pow(magnitude as u32);

        // Get the first digit by normalizing to [1, 10) range
        let normalized = raw_value / (magnitude_power as f64);
        let first_digit = normalized.floor() as u8;

        // Apply appropriate rounding rule based on first digit
        let (candidate_low, candidate_high) = match first_digit {
            1 => {
                // Rule: Steps of 0.05 in the leading digit position
                // Valid values: 100, 105, 110, 115, 120, 125, 130...
                let step_size = magnitude_power / 20; // 0.05 of magnitude
                let base = magnitude_power; // Start at 1 * 10^magnitude

                Self::find_bracketing_candidates(raw_value, base, step_size)
            }
            2..=4 => {
                // Rule: Two significant digits allowed
                // Valid values: 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30...
                let step_size = magnitude_power / 10; // 0.1 of magnitude
                let base = first_digit as u64 * magnitude_power;

                Self::find_bracketing_candidates(raw_value, base, step_size)
            }
            5..=9 => {
                // Rule: Half-steps in the leading digit position
                // Valid values: 500, 550, 600, 650, 700, 750, 800, 850, 900, 950...
                let step_size = magnitude_power / 2; // 0.5 of magnitude
                let base = first_digit as u64 * magnitude_power;


                Self::find_bracketing_candidates(raw_value, base, step_size)
            }
            _ => unreachable!("first_digit must be 1-9")
        };

        // Choose candidate with smaller logarithmic distance
        Self::choose_closest_in_log_space(raw_value, candidate_low, candidate_high)
    }

    /// Find the two bracketing candidates using linear arithmetic (O(1) operation).
    ///
    /// Given a target value and a step pattern, find the two consecutive valid values
    /// that bracket the target in linear space. Due to monotonicity of ln(), these
    /// will also bracket the target in logarithmic space.
    fn find_bracketing_candidates(target: f64, base: u64, step_size: u64) -> (u64, u64) {
        if step_size == 0 {
            return (base, base);
        }

        // Find which interval [k×step, (k+1)×step] contains the target
        let offset = target - (base as f64);
        let k = if offset >= 0.0 {
            (offset / (step_size as f64)).floor() as u64
        } else {
            0 // Handle edge case where target < base
        };

        // Use saturating arithmetic to prevent overflow
        let candidate_low = base.saturating_add(k.saturating_mul(step_size));
        let candidate_high = base.saturating_add((k.saturating_add(1)).saturating_mul(step_size));

        (candidate_low, candidate_high)
    }

    /// Choose the candidate with smaller logarithmic distance to the target.
    fn choose_closest_in_log_space(target: f64, candidate_low: u64, candidate_high: u64) -> u64 {
        if candidate_low == 0 || candidate_high == 0 {
            return if candidate_low > 0 { candidate_low } else { candidate_high };
        }

        let ln_target = target.ln();
        let log_distance_low = (ln_target - (candidate_low as f64).ln()).abs();
        let log_distance_high = (ln_target - (candidate_high as f64).ln()).abs();

        if log_distance_low <= log_distance_high {
            candidate_low
        } else {
            candidate_high
        }
    }
}

impl Distribution<u64> for TriviaGuessDistribution {
    /// Sample a trivia-realistic guess from the distribution.
    ///
    /// This method:
    /// 1. Generates a log-normal sample around the correct answer
    /// 2. Applies trivia-realistic rounding in the logarithmic domain
    /// 3. Returns the result as a u64
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u64 {
        if self.log_std_dev == 0.0 {
            // Perfect certainty case - return the correct answer rounded to trivia format
            return self.round_to_trivia_value(self.correct_answer as f64);
        }

        // Generate standard normal random variable using Box-Muller transform
        let normal_sample: f64 = {
            let u1: f64 = rng.gen_range(0.0..1.0);
            let u2: f64 = rng.gen_range(0.0..1.0);
            (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        };

        // Convert to log-normal distribution around correct answer
        let ln_sample = self.ln_correct_answer + self.log_std_dev * normal_sample;
        let raw_value = ln_sample.exp();

        // Round to trivia-realistic value
        self.round_to_trivia_value(raw_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use quickcheck_macros::quickcheck;

    #[test]
    fn test_constructor_valid_inputs() {
        let dist = TriviaGuessDistribution::new(100, 1.0).unwrap();
        assert_eq!(dist.correct_answer, 100);
        assert_eq!(dist.log_std_dev, 1.0);
        assert!((dist.ln_correct_answer - (100.0_f64).ln()).abs() < 1e-10);
    }

    #[test]
    fn test_constructor_zero_correct_answer() {
        let result = TriviaGuessDistribution::new(0, 1.0);
        assert_eq!(result, Err(TriviaGuessDistributionError::InvalidCorrectAnswer));
    }

    #[test]
    fn test_constructor_negative_log_std_dev() {
        let result = TriviaGuessDistribution::new(100, -1.0);
        assert_eq!(result, Err(TriviaGuessDistributionError::InvalidLogStdDev));
    }

    #[test]
    fn test_constructor_nan_log_std_dev() {
        let result = TriviaGuessDistribution::new(100, f64::NAN);
        assert_eq!(result, Err(TriviaGuessDistributionError::InvalidLogStdDev));
    }

    #[test]
    fn test_constructor_infinite_log_std_dev() {
        let result = TriviaGuessDistribution::new(100, f64::INFINITY);
        assert_eq!(result, Err(TriviaGuessDistributionError::InvalidLogStdDev));
    }

    #[test]
    fn test_constructor_too_large_log_std_dev() {
        let result = TriviaGuessDistribution::new(100, 51.0);
        assert_eq!(result, Err(TriviaGuessDistributionError::LogStdDevTooLarge));
    }

    #[test]
    fn test_constructor_boundary_log_std_dev() {
        // Should accept exactly 50.0
        let result = TriviaGuessDistribution::new(100, 50.0);
        assert!(result.is_ok());

        // Should accept 0.0 (perfect certainty)
        let result = TriviaGuessDistribution::new(100, 0.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_basic_sampling() {
        let mut rng = StdRng::seed_from_u64(42);
        let dist = TriviaGuessDistribution::new(1000, 0.5).unwrap();

        // Should be able to sample without panicking
        let sample = dist.sample(&mut rng);
        assert!(sample > 0);
    }

    #[test]
    fn test_perfect_certainty_deterministic() {
        let mut rng = StdRng::seed_from_u64(42);
        let dist = TriviaGuessDistribution::new(1000, 0.0).unwrap();

        // With perfect certainty, should always return the same value
        let sample1 = dist.sample(&mut rng);
        let sample2 = dist.sample(&mut rng);
        let sample3 = dist.sample(&mut rng);

        assert_eq!(sample1, sample2);
        assert_eq!(sample2, sample3);
    }

    #[quickcheck]
    fn prop_all_samples_positive(correct_answer: u64, log_std_dev_scaled: u8, seed: u64) -> bool {
        let correct_answer = correct_answer.max(1); // Ensure valid input
        let log_std_dev = (log_std_dev_scaled as f64) / 10.0; // Scale to [0, 25.5]

        if let Ok(dist) = TriviaGuessDistribution::new(correct_answer, log_std_dev) {
            let mut rng = StdRng::seed_from_u64(seed);
            let sample = dist.sample(&mut rng);
            sample > 0
        } else {
            true // Skip invalid constructor parameters
        }
    }

    #[quickcheck]
    fn prop_sampling_never_panics(correct_answer: u64, log_std_dev_scaled: u8, seed: u64) -> bool {
        let correct_answer = correct_answer.max(1); // Ensure valid input
        let log_std_dev = (log_std_dev_scaled as f64) / 10.0; // Scale to [0, 25.5]

        if let Ok(dist) = TriviaGuessDistribution::new(correct_answer, log_std_dev) {
            let mut rng = StdRng::seed_from_u64(seed);
            let _sample = dist.sample(&mut rng);
            true
        } else {
            true // Skip invalid constructor parameters
        }
    }

    // Unit tests for rounding rules

    #[test]
    fn test_rounding_first_digit_1_basic() {
        let dist = TriviaGuessDistribution::new(100, 0.0).unwrap();

        // Test values starting with 1 - should use steps of 0.05 * magnitude
        assert_eq!(dist.round_to_trivia_value(100.0), 100); // Exact match
        assert_eq!(dist.round_to_trivia_value(105.0), 105); // Valid step
        assert_eq!(dist.round_to_trivia_value(110.0), 110); // Valid step
        assert_eq!(dist.round_to_trivia_value(115.0), 115); // Valid step
        assert_eq!(dist.round_to_trivia_value(120.0), 120); // Valid step
        assert_eq!(dist.round_to_trivia_value(125.0), 125); // Valid step

        // Test rounding between valid values (logarithmic midpoints)
        // Log midpoint between 100 and 105 is sqrt(100*105) ≈ 102.47
        assert_eq!(dist.round_to_trivia_value(102.0), 100); // Below log midpoint -> round to 100
        assert_eq!(dist.round_to_trivia_value(103.0), 105); // Above log midpoint -> round to 105

        // Log midpoint between 105 and 110 is sqrt(105*110) ≈ 107.42
        assert_eq!(dist.round_to_trivia_value(107.0), 105); // Below log midpoint -> round to 105
        assert_eq!(dist.round_to_trivia_value(108.0), 110); // Above log midpoint -> round to 110
    }

    #[test]
    fn test_rounding_first_digit_1_different_magnitudes() {
        let dist = TriviaGuessDistribution::new(1000, 0.0).unwrap();

        // Test with thousands (magnitude 3)
        assert_eq!(dist.round_to_trivia_value(1000.0), 1000);
        assert_eq!(dist.round_to_trivia_value(1050.0), 1050);
        assert_eq!(dist.round_to_trivia_value(1100.0), 1100);

        // Test logarithmic midpoints at thousands scale
        // Log midpoint between 1000 and 1050 is sqrt(1000*1050) ≈ 1024.69
        assert_eq!(dist.round_to_trivia_value(1024.0), 1000); // Below log midpoint -> round to 1000
        assert_eq!(dist.round_to_trivia_value(1026.0), 1050); // Above log midpoint -> round to 1050
    }

    #[test]
    fn test_rounding_first_digits_2_to_4() {
        let dist = TriviaGuessDistribution::new(250, 0.0).unwrap();

        // Test values starting with 2-4 - should use two significant digits
        assert_eq!(dist.round_to_trivia_value(200.0), 200); // Exact match
        assert_eq!(dist.round_to_trivia_value(210.0), 210); // Valid two-digit
        assert_eq!(dist.round_to_trivia_value(220.0), 220); // Valid two-digit
        assert_eq!(dist.round_to_trivia_value(250.0), 250); // Valid two-digit
        assert_eq!(dist.round_to_trivia_value(290.0), 290); // Valid two-digit
        assert_eq!(dist.round_to_trivia_value(300.0), 300); // Valid two-digit
        assert_eq!(dist.round_to_trivia_value(350.0), 350); // Valid two-digit
        assert_eq!(dist.round_to_trivia_value(400.0), 400); // Valid two-digit
        assert_eq!(dist.round_to_trivia_value(450.0), 450); // Valid two-digit

        // Test rounding between valid values (logarithmic midpoints)
        // Log midpoint between 200 and 210 is sqrt(200*210) ≈ 204.94
        assert_eq!(dist.round_to_trivia_value(204.0), 200); // Below log midpoint -> round to 200
        assert_eq!(dist.round_to_trivia_value(206.0), 210); // Above log midpoint -> round to 210

        // Log midpoint between 210 and 220 is sqrt(210*220) ≈ 214.94
        assert_eq!(dist.round_to_trivia_value(214.0), 210); // Below log midpoint -> round to 210
        assert_eq!(dist.round_to_trivia_value(216.0), 220); // Above log midpoint -> round to 220
    }

    #[test]
    fn test_rounding_first_digits_5_plus() {
        let dist = TriviaGuessDistribution::new(750, 0.0).unwrap();

        // Test values starting with 5+ - should use half-steps
        assert_eq!(dist.round_to_trivia_value(500.0), 500); // Exact match
        assert_eq!(dist.round_to_trivia_value(550.0), 550); // Valid half-step
        assert_eq!(dist.round_to_trivia_value(600.0), 600); // Valid half-step
        assert_eq!(dist.round_to_trivia_value(650.0), 650); // Valid half-step
        assert_eq!(dist.round_to_trivia_value(700.0), 700); // Valid half-step
        assert_eq!(dist.round_to_trivia_value(750.0), 750); // Valid half-step
        assert_eq!(dist.round_to_trivia_value(800.0), 800); // Valid half-step
        assert_eq!(dist.round_to_trivia_value(850.0), 850); // Valid half-step
        assert_eq!(dist.round_to_trivia_value(900.0), 900); // Valid half-step
        assert_eq!(dist.round_to_trivia_value(950.0), 950); // Valid half-step

        // Test rounding between valid values (logarithmic midpoints)
        // Log midpoint between 500 and 550 is sqrt(500*550) ≈ 524.40
        assert_eq!(dist.round_to_trivia_value(524.0), 500); // Below log midpoint -> round to 500
        assert_eq!(dist.round_to_trivia_value(525.0), 550); // Above log midpoint -> round to 550

        // Log midpoint between 550 and 600 is sqrt(550*600) ≈ 574.46
        assert_eq!(dist.round_to_trivia_value(574.0), 550); // Below log midpoint -> round to 550
        assert_eq!(dist.round_to_trivia_value(575.0), 600); // Above log midpoint -> round to 600

        // Log midpoint between 600 and 650 is sqrt(600*650) ≈ 624.50
        assert_eq!(dist.round_to_trivia_value(624.0), 600); // Below log midpoint -> round to 600
        assert_eq!(dist.round_to_trivia_value(625.0), 650); // Above log midpoint -> round to 650
    }

    #[test]
    fn test_rounding_edge_cases() {
        let dist = TriviaGuessDistribution::new(100, 0.0).unwrap();

        // Test edge cases
        assert_eq!(dist.round_to_trivia_value(0.5), 1); // Below 1 should return 1
        assert_eq!(dist.round_to_trivia_value(1.0), 1); // Exactly 1 should return 1
        assert_eq!(dist.round_to_trivia_value(1.5), 1); // Just above 1 should still return 1 (rounds to magnitude 0)

        // Test very large numbers
        assert_eq!(dist.round_to_trivia_value(1_000_000.0), 1_000_000);
        assert_eq!(dist.round_to_trivia_value(5_500_000.0), 5_500_000);
    }

    #[test]
    fn test_logarithmic_midpoint_rounding() {
        let dist = TriviaGuessDistribution::new(100000, 0.0).unwrap();

        // Test case from plan: between 100,000 and 105,000, log midpoint is ~102,469.5
        // 102,469 should round to 100,000, 102,470 should round to 105,000
        assert_eq!(dist.round_to_trivia_value(102469.0), 100000);
        assert_eq!(dist.round_to_trivia_value(102470.0), 105000);
    }

    #[test]
    fn test_cross_magnitude_rounding() {
        let dist = TriviaGuessDistribution::new(197500, 0.0).unwrap();

        // Test case from plan: between 195,000 and 200,000, log midpoint is ~197,484.2
        // This tests rounding across different rule sets (1xx,xxx vs 2xx,xxx)
        assert_eq!(dist.round_to_trivia_value(197484.0), 195000); // Stays in "first digit 1" rule
        assert_eq!(dist.round_to_trivia_value(197485.0), 200000); // Jumps to "first digit 2" rule
    }

    #[test]
    fn test_rule_transitions_at_boundaries() {
        let dist = TriviaGuessDistribution::new(975000, 0.0).unwrap();

        // Test between 950,000 and 1,000,000 (both use different rules but different magnitudes)
        let test_val = (950000.0 * 1000000.0_f64).sqrt(); // Geometric mean
        let result = dist.round_to_trivia_value(test_val);
        // Should round to one of the two values
        assert!(result == 950000 || result == 1000000);
    }

    #[test]
    fn test_rule_transition_2_4_to_5_plus() {
        let dist = TriviaGuessDistribution::new(450000, 0.0).unwrap();

        // Test transition between 450,000 (2-4 rule) and 500,000 (5+ rule)
        // Note: values between these will follow the rule based on their own first digit
        let test_val_4x = 475000.0; // First digit 4, should use 2-4 rule -> rounds to 470000 or 480000
        let result_4x = dist.round_to_trivia_value(test_val_4x);
        assert!(result_4x == 470000 || result_4x == 480000);

        let test_val_5x = 500000.0; // First digit 5, should use 5+ rule -> rounds to 500000
        let result_5x = dist.round_to_trivia_value(test_val_5x);
        assert_eq!(result_5x, 500000);
    }

    // Critical validation tests from the plan

    #[test]
    fn test_three_digit_sample_validation() {
        // Create distribution with correct_answer=316, log_std_dev=1.151
        let dist = TriviaGuessDistribution::new(316, 1.151).unwrap();
        let mut rng = StdRng::seed_from_u64(42);

        // Sample many values, filter to three-digit results (100-999)
        let mut three_digit_samples = Vec::new();
        for _ in 0..1000 {
            let sample = dist.sample(&mut rng);
            if sample >= 100 && sample <= 999 {
                three_digit_samples.push(sample);
            }
        }

        // Verify all three-digit samples are valid trivia numbers
        let valid_trivia_numbers = generate_valid_trivia_numbers_in_range(100, 999);
        for sample in three_digit_samples {
            assert!(valid_trivia_numbers.contains(&sample),
                    "Sample {} is not a valid trivia number", sample);
        }
    }

    #[test]
    fn test_deterministic_perfect_certainty() {
        let test_cases = vec![
            100, 105, 110, 115, 120, 125, // First digit 1 cases
            200, 210, 220, 230, 240,     // First digits 2-4 cases
            500, 550, 600, 650, 700,     // First digits 5+ cases
            1000, 1050, 1100, 1150,      // Different magnitude cases
        ];

        for correct_answer in test_cases {
            let dist = TriviaGuessDistribution::new(correct_answer, 0.0).unwrap();
            let mut rng = StdRng::seed_from_u64(42);

            // With perfect certainty (log_std_dev=0.0), sampling should always return
            // the correct answer (deterministic rounding)
            for _ in 0..10 {
                let sample = dist.sample(&mut rng);
                assert_eq!(sample, correct_answer,
                          "Perfect certainty failed for correct_answer={}", correct_answer);
            }
        }
    }

    #[test]
    fn test_boundary_rounding_geometric_midpoints() {
        // Test boundary rounding between adjacent valid values at their geometric middle
        let test_pairs = vec![
            (100, 105), (105, 110), (110, 115),  // First digit 1
            (200, 210), (210, 220), (220, 230),  // First digits 2-4
            (500, 550), (550, 600), (600, 650),  // First digits 5+
        ];

        for (low, high) in test_pairs {
            let dist = TriviaGuessDistribution::new(low, 0.0).unwrap();

            // Find geometric middle point and scale it up for testing
            let geometric_middle = ((low as f64) * (high as f64)).sqrt();
            let scale_factor = 10000.0;
            let scaled_middle = geometric_middle * scale_factor;

            // Create test points slightly below and above the scaled geometric middle
            let test_below = scaled_middle - 1.0;
            let test_above = scaled_middle + 1.0;

            // Both should round to one of the two adjacent scaled valid values
            let result_below = dist.round_to_trivia_value(test_below);
            let result_above = dist.round_to_trivia_value(test_above);

            let scaled_low = (low as f64 * scale_factor) as u64;
            let scaled_high = (high as f64 * scale_factor) as u64;

            // Results should be one of the two candidates
            assert!(result_below == scaled_low || result_below == scaled_high,
                    "test_below={} should round to {} or {}, got {}",
                    test_below, scaled_low, scaled_high, result_below);

            assert!(result_above == scaled_low || result_above == scaled_high,
                    "test_above={} should round to {} or {}, got {}",
                    test_above, scaled_low, scaled_high, result_above);

            // They should round to different values (one below, one above the midpoint)
            assert_ne!(result_below, result_above,
                      "Values {} and {} should round to different candidates, both got {}",
                      test_below, test_above, result_below);
        }
    }

    /// Generate all valid trivia numbers in a given range for validation testing
    fn generate_valid_trivia_numbers_in_range(min: u64, max: u64) -> std::collections::HashSet<u64> {
        let mut valid_numbers = std::collections::HashSet::new();

        for magnitude in 0..=18 {
            let magnitude_power = 10_u64.pow(magnitude);
            if magnitude_power > max {
                break;
            }

            // First digit 1: steps of 0.05 in leading digit position
            if magnitude_power >= min {
                for k in 0..20 { // 0.05 * 20 = 1.0, so covers 1.xx range
                    let value = magnitude_power + (magnitude_power / 20) * k;
                    if value >= min && value <= max {
                        valid_numbers.insert(value);
                    }
                    if value > max {
                        break;
                    }
                }
            }

            // First digits 2-4: two significant digits
            for first_digit in 2..=4 {
                let base = first_digit * magnitude_power;
                if base > max {
                    break;
                }
                for k in 0..10 { // 0.1 * 10 = 1.0, covers the digit range
                    let value = base + (magnitude_power / 10) * k;
                    if value >= min && value <= max {
                        valid_numbers.insert(value);
                    }
                    if value > max {
                        break;
                    }
                }
            }

            // First digits 5-9: half-steps in leading digit position
            for first_digit in 5..=9 {
                let base = first_digit * magnitude_power;
                if base > max {
                    break;
                }
                for k in 0..2 { // 0.5 * 2 = 1.0, covers the digit range
                    let value = base + (magnitude_power / 2) * k;
                    if value >= min && value <= max {
                        valid_numbers.insert(value);
                    }
                    if value > max {
                        break;
                    }
                }
            }
        }

        valid_numbers
    }
}