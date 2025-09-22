mod exact;
mod log_linear;
mod table_based;
mod traits;
mod evaluation;

use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::evaluation::evaluate_estimate;
use crate::exact::ExactGeometricMean;
use crate::log_linear::LogLinearApproximation;
use crate::table_based::TableBasedApproximation;

fn main() {
    println!("Pen and Paper Geometric Mean Comparison");
    println!("======================================");

    let mut rng = StdRng::seed_from_u64(42);
    let num_tests = 10000;
    let min_value = 1.0;
    let max_value = 100000.0;

    println!("Testing {} random cases with values from {} to {}", num_tests, min_value, max_value);
    println!();

    // Exact method (baseline)
    let exact_results = evaluate_estimate::<_, ExactGeometricMean>(&mut rng, min_value, max_value, num_tests);
    println!("Exact Method:");
    println!("  Mean Absolute Relative Error: {:.6e}", exact_results.mean_absolute_relative_error);
    println!("  Valid Tests: {}", exact_results.total_tests);
    println!();

    // Log-linear approximation
    let mut rng = StdRng::seed_from_u64(42); // Reset with same seed for fair comparison
    let log_linear_results = evaluate_estimate::<_, LogLinearApproximation>(&mut rng, min_value, max_value, num_tests);
    println!("Log-Linear Interpolation:");
    println!("  Mean Absolute Relative Error: {:.6e}", log_linear_results.mean_absolute_relative_error);
    println!("  Valid Tests: {}", log_linear_results.total_tests);
    println!();

    // Table-based approximation
    let mut rng = StdRng::seed_from_u64(42); // Reset with same seed for fair comparison
    let table_results = evaluate_estimate::<_, TableBasedApproximation>(&mut rng, min_value, max_value, num_tests);
    println!("Table-Based Approximation:");
    println!("  Mean Absolute Relative Error: {:.6e}", table_results.mean_absolute_relative_error);
    println!("  Valid Tests: {}", table_results.total_tests);
    println!();

    println!("Comparison Summary:");
    println!("  Log-Linear vs Exact: {:.2}x worse", log_linear_results.mean_absolute_relative_error / exact_results.mean_absolute_relative_error);
    println!("  Table-Based vs Exact: {:.2}x worse", table_results.mean_absolute_relative_error / exact_results.mean_absolute_relative_error);
    println!("  Table-Based vs Log-Linear: {:.2}x", table_results.mean_absolute_relative_error / log_linear_results.mean_absolute_relative_error);
}
