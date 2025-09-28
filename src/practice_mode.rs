use rand::Rng;
use rand::distributions::Distribution;
use std::marker::PhantomData;
use std::time::Duration;

use crate::exact::geometric_mean;
use crate::traits::EstimateGeometricMean;
use crate::trivia_guess::TriviaGuessDistribution;

/// Timer trait for abstracting time measurement, enabling testable timing
pub trait Timer {
    type Instant: Clone;

    /// Get the current instant
    fn now(&self) -> Self::Instant;

    /// Calculate duration between two instants
    fn elapsed(&self, start: Self::Instant) -> Duration;
}

/// Production timer implementation using std::time
#[derive(Copy, Clone)]
pub struct SystemTimer;

impl Timer for SystemTimer {
    type Instant = std::time::Instant;

    fn now(&self) -> Self::Instant {
        std::time::Instant::now()
    }

    fn elapsed(&self, start: Self::Instant) -> Duration {
        start.elapsed()
    }
}

/// Configuration for practice mode sessions
#[derive(Debug, Clone, PartialEq)]
pub struct PracticeModeConfig {
    pub team_size: usize,
    pub log_std_dev: f64,
    pub min_answer: u64,
    pub max_answer: u64,
}

/// Errors that can occur during practice mode configuration
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigurationError {
    ZeroTeamSize,
    InvalidAnswerRange,
}

impl std::fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigurationError::ZeroTeamSize => write!(f, "Team size cannot be zero"),
            ConfigurationError::InvalidAnswerRange => write!(f, "Answer range cannot be empty (min >= max)"),
        }
    }
}

impl std::error::Error for ConfigurationError {}

impl PracticeModeConfig {
    pub fn new(team_size: usize, log_std_dev: f64, min_answer: u64, max_answer: u64) -> Result<Self, ConfigurationError> {
        if team_size == 0 {
            return Err(ConfigurationError::ZeroTeamSize);
        }

        if min_answer >= max_answer {
            return Err(ConfigurationError::InvalidAnswerRange);
        }

        Ok(PracticeModeConfig {
            team_size,
            log_std_dev,
            min_answer,
            max_answer,
        })
    }
}

/// Answer evaluation result
#[derive(Debug, Clone, PartialEq)]
pub enum AnswerEvaluation {
    /// User answer equals floor(estimation_method_result) or ceiling(estimation_method_result)
    Correct,
    /// User answer is closer to exact geometric mean than estimation method result
    Excellent,
    /// User answer does not meet either criteria above
    Incorrect,
}

/// Type states for practice mode session
pub struct Ready;

/// Core practice mode session with type-safe state pattern
pub struct PracticeSession<S, R, T, E> {
    rng: R,
    timer: T,
    estimation_method: PhantomData<E>,
    state: PhantomData<S>,
}

/// Active session containing problem data and timing information
pub struct ActiveSession<T: Timer, E> {
    exact_geometric_mean: f64,
    estimation_result: f64,
    start_instant: T::Instant,
    timer: T,
    estimation_method: PhantomData<E>,
}

impl<R: Rng, T: Timer, E: EstimateGeometricMean> PracticeSession<Ready, R, T, E> {
    /// Create a new practice session in ready state
    pub fn new(rng: R, timer: T) -> Self {
        PracticeSession {
            rng,
            timer,
            estimation_method: PhantomData,
            state: PhantomData,
        }
    }

    /// Start a new practice problem, returning guesses and active session
    pub fn start(mut self, config: PracticeModeConfig) -> Result<(Vec<u64>, ActiveSession<T, E>), ConfigurationError> {
        // Validate configuration - config was already validated during construction

        // Generate random correct answer in log space
        let ln_min = (config.min_answer as f64).ln();
        let ln_max = (config.max_answer as f64).ln();
        let ln_correct_answer = self.rng.gen_range(ln_min..ln_max);
        let correct_answer = ln_correct_answer.exp() as u64;

        // Create trivia guess distribution
        let distribution = TriviaGuessDistribution::new(correct_answer, config.log_std_dev)
            .map_err(|_| ConfigurationError::InvalidAnswerRange)?;

        // Generate team guesses
        let guesses: Vec<u64> = (0..config.team_size)
            .map(|_| distribution.sample(&mut self.rng))
            .collect();

        // Calculate exact geometric mean
        let guesses_f64: Vec<f64> = guesses.iter().map(|&x| x as f64).collect();
        let exact_geometric_mean = geometric_mean(&guesses_f64)
            .map_err(|_| ConfigurationError::InvalidAnswerRange)?;

        // Calculate estimation method result
        let estimation_result = E::estimate_geometric_mean(&guesses_f64)
            .map_err(|_| ConfigurationError::InvalidAnswerRange)?;

        // Start timing
        let start_instant = self.timer.now();

        let active_session = ActiveSession {
            exact_geometric_mean,
            estimation_result,
            start_instant,
            timer: self.timer,
            estimation_method: PhantomData,
        };

        Ok((guesses, active_session))
    }
}

impl<T: Timer, E: EstimateGeometricMean> ActiveSession<T, E> {
    /// Submit user answer and get evaluation result
    pub fn submit_answer(self, user_answer: u64) -> PracticeResult {
        let duration = self.timer.elapsed(self.start_instant);

        let evaluation = evaluate_answer(
            user_answer,
            self.exact_geometric_mean,
            self.estimation_result,
        );

        PracticeResult {
            user_answer,
            exact_geometric_mean: self.exact_geometric_mean,
            estimation_result: self.estimation_result as u64,
            duration,
            evaluation,
        }
    }
}

/// Result of a practice session submission
#[derive(Debug, Clone, PartialEq)]
pub struct PracticeResult {
    pub user_answer: u64,
    pub exact_geometric_mean: f64,
    pub estimation_result: u64,
    pub duration: Duration,
    pub evaluation: AnswerEvaluation,
}

/// Evaluate user answer according to plan specifications
fn evaluate_answer(user_answer: u64, exact_geometric_mean: f64, estimation_result: f64) -> AnswerEvaluation {
    let estimation_floor = estimation_result.floor() as u64;
    let estimation_ceil = estimation_result.ceil() as u64;

    // Check if user answer matches floor or ceiling of estimation result (highest precedence)
    if user_answer == estimation_floor || user_answer == estimation_ceil {
        return AnswerEvaluation::Correct;
    }

    // Check if user answer is within excellent range
    let error_margin = (estimation_result - exact_geometric_mean).abs();
    let excellent_range_min = exact_geometric_mean - error_margin;
    let excellent_range_max = exact_geometric_mean + error_margin;

    // Use exclusive boundaries for excellent range to avoid edge case conflicts
    let user_answer_f64 = user_answer as f64;
    if user_answer_f64 > excellent_range_min && user_answer_f64 < excellent_range_max {
        return AnswerEvaluation::Excellent;
    }

    AnswerEvaluation::Incorrect
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table_based::TableBasedApproximation;
    use rand::{SeedableRng, rngs::StdRng};

    /// Mock timer for testing with predictable, incrementing durations
    #[derive(Clone)]
    struct MockTimer {
        counter: std::cell::Cell<u64>,
    }

    impl MockTimer {
        fn new() -> Self {
            MockTimer {
                counter: std::cell::Cell::new(0),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct MockInstant(u64);

    impl Timer for MockTimer {
        type Instant = MockInstant;

        fn now(&self) -> Self::Instant {
            let current = self.counter.get();
            self.counter.set(current + 1);
            MockInstant(current)
        }

        fn elapsed(&self, start: Self::Instant) -> Duration {
            let current = self.counter.get();
            Duration::from_millis((current - start.0) * 100) // 100ms increments
        }
    }

    /// Mock estimation method that sums all values for predictable testing
    struct SumEstimation;

    impl EstimateGeometricMean for SumEstimation {
        type Error = crate::exact::GeometricMeanError;

        fn estimate_geometric_mean(values: &[f64]) -> Result<f64, Self::Error> {
            if values.is_empty() {
                return Err(crate::exact::GeometricMeanError::EmptyInput);
            }
            Ok(values.iter().sum())
        }
    }

    #[test]
    fn test_configuration_validation() {
        // Valid configuration
        let config = PracticeModeConfig::new(4, 1.0, 10, 1000).unwrap();
        assert_eq!(config.team_size, 4);
        assert_eq!(config.log_std_dev, 1.0);
        assert_eq!(config.min_answer, 10);
        assert_eq!(config.max_answer, 1000);

        // Zero team size
        let result = PracticeModeConfig::new(0, 1.0, 10, 1000);
        assert_eq!(result, Err(ConfigurationError::ZeroTeamSize));

        // Invalid answer range
        let result = PracticeModeConfig::new(4, 1.0, 1000, 10);
        assert_eq!(result, Err(ConfigurationError::InvalidAnswerRange));

        let result = PracticeModeConfig::new(4, 1.0, 100, 100);
        assert_eq!(result, Err(ConfigurationError::InvalidAnswerRange));
    }

    #[test]
    fn test_answer_evaluation_floor_ceil_precedence() {
        // Case 1: Estimate: 100.5, Exact: 98.5
        assert_eq!(evaluate_answer(100, 98.5, 100.5), AnswerEvaluation::Correct); // floor match
        assert_eq!(evaluate_answer(101, 98.5, 100.5), AnswerEvaluation::Correct); // ceil match
        assert_eq!(evaluate_answer(99, 98.5, 100.5), AnswerEvaluation::Excellent); // within excellent range
        assert_eq!(evaluate_answer(102, 98.5, 100.5), AnswerEvaluation::Incorrect); // outside both criteria

        // Case 2: Integer estimation result
        assert_eq!(evaluate_answer(150, 50.0, 150.0), AnswerEvaluation::Correct); // exact match
        assert_eq!(evaluate_answer(100, 50.0, 150.0), AnswerEvaluation::Excellent); // better than estimation
    }

    #[test]
    fn test_answer_evaluation_excellent_range_boundaries() {
        // Case with exclusive boundaries: Estimate: 100.0, Exact: 98.0
        // Error margin = 2.0, excellent range = (96.0, 100.0) exclusive
        assert_eq!(evaluate_answer(100, 98.0, 100.0), AnswerEvaluation::Correct); // floor/ceil match
        assert_eq!(evaluate_answer(96, 98.0, 100.0), AnswerEvaluation::Incorrect); // at boundary (exclusive)
        assert_eq!(evaluate_answer(97, 98.0, 100.0), AnswerEvaluation::Excellent); // within range
        assert_eq!(evaluate_answer(99, 98.0, 100.0), AnswerEvaluation::Excellent); // within range
    }

    #[test]
    fn test_practice_session_flow_with_sum_estimation() {
        let rng = StdRng::seed_from_u64(42);
        let timer = MockTimer::new();
        let config = PracticeModeConfig::new(2, 1.0, 10, 100).unwrap();

        // Create session and start problem
        let session: PracticeSession<Ready, _, _, SumEstimation> = PracticeSession::new(rng, timer);
        let (guesses, active_session) = session.start(config).unwrap();

        // Verify we got expected number of guesses
        assert_eq!(guesses.len(), 2);

        // Calculate what the sum estimation should be
        let expected_sum: u64 = guesses.iter().sum();

        // Submit the expected sum - should be Correct
        let result = active_session.submit_answer(expected_sum);
        assert_eq!(result.evaluation, AnswerEvaluation::Correct);
        assert_eq!(result.user_answer, expected_sum);
        assert_eq!(result.estimation_result, expected_sum);

        // Verify timing worked
        assert!(result.duration > Duration::from_millis(0));
    }

    #[test]
    fn test_practice_session_sum_minus_one_excellent() {
        let rng = StdRng::seed_from_u64(123);
        let timer = MockTimer::new();
        let config = PracticeModeConfig::new(3, 0.5, 50, 500).unwrap();

        let session: PracticeSession<Ready, _, _, SumEstimation> = PracticeSession::new(rng, timer);
        let (guesses, active_session) = session.start(config).unwrap();

        let sum: u64 = guesses.iter().sum();
        let sum_minus_one = sum.saturating_sub(1);

        // Submit sum minus one - should be Excellent (closer to exact geometric mean)
        let result = active_session.submit_answer(sum_minus_one);
        assert_eq!(result.evaluation, AnswerEvaluation::Excellent);
    }

    #[test]
    fn test_timer_validation_with_mock() {
        let rng = StdRng::seed_from_u64(999);
        let timer = MockTimer::new();
        let config = PracticeModeConfig::new(2, 1.0, 10, 100).unwrap();

        // Track initial timer state
        let initial_counter = timer.counter.get();

        let session: PracticeSession<Ready, _, _, SumEstimation> = PracticeSession::new(rng, timer);
        let (_guesses, active_session) = session.start(config).unwrap();

        // Timer should have been called once during start
        let mid_counter = active_session.timer.counter.get();
        assert!(mid_counter > initial_counter);

        let result = active_session.submit_answer(100);

        // Timer should have been called again during submit_answer for elapsed calculation
        let final_counter = result.duration.as_millis();
        assert!(final_counter > 0);
    }

    #[test]
    fn test_multiple_sessions_independent_timing() {
        // Each session should get its own timer to ensure independent timing
        let timer1 = MockTimer::new();
        let timer2 = MockTimer::new();

        // First session
        let rng1 = StdRng::seed_from_u64(111);
        let config = PracticeModeConfig::new(2, 1.0, 10, 100).unwrap();
        let session1: PracticeSession<Ready, _, _, SumEstimation> = PracticeSession::new(rng1, timer1);
        let (_guesses1, active1) = session1.start(config.clone()).unwrap();
        let result1 = active1.submit_answer(50);

        // Second session
        let rng2 = StdRng::seed_from_u64(222);
        let session2: PracticeSession<Ready, _, _, SumEstimation> = PracticeSession::new(rng2, timer2);
        let (_guesses2, active2) = session2.start(config).unwrap();
        let result2 = active2.submit_answer(75);

        // Both should have valid timing (each session uses separate timer instances)
        assert!(result1.duration > Duration::from_millis(0));
        assert!(result2.duration > Duration::from_millis(0));
    }

    #[test]
    fn test_real_table_based_approximation_integration() {
        let rng = StdRng::seed_from_u64(42);
        let timer = MockTimer::new();
        let config = PracticeModeConfig::new(4, 1.0, 100, 10000).unwrap();

        let session: PracticeSession<Ready, _, _, TableBasedApproximation> = PracticeSession::new(rng, timer);
        let (guesses, active_session) = session.start(config).unwrap();

        // Should generate valid guesses
        assert_eq!(guesses.len(), 4);
        for &guess in &guesses {
            assert!(guess >= 1);
        }

        // Submit the estimation method result - should be Correct
        let estimation_result = active_session.estimation_result as u64;
        let result = active_session.submit_answer(estimation_result);
        assert_eq!(result.evaluation, AnswerEvaluation::Correct);

        // Verify all result fields are populated
        assert!(result.exact_geometric_mean > 0.0);
        assert!(result.estimation_result > 0);
        assert!(result.duration > Duration::from_millis(0));
    }
}
