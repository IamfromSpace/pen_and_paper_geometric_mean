use std::io::{self, Write};
use std::time::Duration;

use crate::practice_mode::{
    AnswerEvaluation, PracticeModeConfig, PracticeSession, Ready, SystemTimer,
};
use crate::table_based::TableBasedApproximation;
use rand::{SeedableRng, rngs::StdRng};

/// Format problem display for consistent presentation
pub fn format_problem_display(guesses: &[u64]) -> String {
    let mut output = String::new();
    output.push_str("Here are the team's guesses:\n");

    for (i, guess) in guesses.iter().enumerate() {
        output.push_str(&format!("  {}. {}\n", i + 1, format_number(*guess)));
    }

    output
}

/// Format results display for consistent presentation
pub fn format_results_display(
    user_answer: u64,
    exact_mean: f64,
    estimation_result: u64,
    duration: Duration,
    evaluation: AnswerEvaluation,
) -> String {
    let mut output = String::new();

    output.push_str("Results:\n");
    output.push_str("========\n");
    output.push_str(&format!("Your answer: {}\n", format_number(user_answer)));
    output.push_str(&format!("Exact geometric mean: {:.1}\n", exact_mean));
    output.push_str(&format!("Estimation method result: {}\n", format_number(estimation_result)));
    output.push_str(&format!("Time taken: {:.1} seconds\n", duration.as_secs_f64()));
    output.push('\n');

    match evaluation {
        AnswerEvaluation::Correct => {
            output.push_str("✓ CORRECT! You calculated the estimation method properly.\n");
        }
        AnswerEvaluation::Excellent => {
            output.push_str("★ EXCELLENT! Your answer is closer to the exact value than the estimation method!\n");
        }
        AnswerEvaluation::Incorrect => {
            output.push_str("You have calculated the estimation method incorrectly.\n");
        }
    }

    output
}

/// Format numbers with thousands separators for display
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();

    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result.chars().rev().collect()
}

/// Parse user input as u64, handling validation
fn parse_user_input(input: &str) -> Result<u64, String> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err("Please enter a number".to_string());
    }

    // Remove commas for parsing
    let cleaned = trimmed.replace(',', "");

    match cleaned.parse::<u64>() {
        Ok(value) => {
            if value == 0 {
                Err("Please enter a positive number".to_string())
            } else {
                Ok(value)
            }
        }
        Err(_) => {
            if cleaned.contains('.') {
                Err("Please enter a whole number (no decimals)".to_string())
            } else if cleaned.starts_with('-') {
                Err("Please enter a positive number".to_string())
            } else {
                Err("Please enter a valid number".to_string())
            }
        }
    }
}

/// Prompt user for input with validation and retry
fn prompt_for_answer() -> u64 {
    loop {
        print!("Enter your estimated geometric mean: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input. Please try again.");
            continue;
        }

        match parse_user_input(&input) {
            Ok(value) => return value,
            Err(error) => {
                println!("Invalid input: {}. Please try again.", error);
            }
        }
    }
}

/// Prompt user for continue/exit choice
fn prompt_for_continue() -> bool {
    loop {
        print!("Continue with another problem? (y/n): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input. Please try again.");
            continue;
        }

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => {
                println!("Please enter 'y' for yes or 'n' for no.");
            }
        }
    }
}

/// Run the practice mode CLI
pub fn run_practice_mode() {
    println!("Practice Mode - Table-Based Geometric Mean");
    println!("=========================================");
    println!();

    // Fixed configuration as specified in the plan
    let config = PracticeModeConfig::new(4, 4.0, 10, 1_000_000_000).unwrap();

    // Use system-generated seed for variety
    let mut rng = StdRng::from_entropy();
    let timer = SystemTimer;

    loop {
        // Create new session for each problem
        let session: PracticeSession<Ready, _, _, TableBasedApproximation> =
            PracticeSession::new(&mut rng, timer);

        // Start problem
        let (guesses, active_session) = match session.start(config.clone()) {
            Ok(result) => result,
            Err(e) => {
                println!("Error generating problem: {}", e);
                return;
            }
        };

        // Display problem
        print!("{}", format_problem_display(&guesses));
        println!();

        // Get user answer
        let user_answer = prompt_for_answer();
        println!();

        // Submit answer and get results
        let result = active_session.submit_answer(user_answer);

        // Display results
        print!("{}", format_results_display(
            result.user_answer,
            result.exact_geometric_mean,
            result.estimation_result,
            result.duration,
            result.evaluation,
        ));
        println!();

        // Check if user wants to continue
        if !prompt_for_continue() {
            break;
        }
        println!();
    }

    println!("Thanks for practicing!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_format_problem_display() {
        let guesses = vec![150, 2500, 800, 45];
        let result = format_problem_display(&guesses);

        let expected = "Here are the team's guesses:\n  1. 150\n  2. 2,500\n  3. 800\n  4. 45\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_results_display_correct() {
        let result = format_results_display(
            420,
            387.4,
            400,
            Duration::from_millis(12300),
            AnswerEvaluation::Correct,
        );

        assert!(result.contains("Your answer: 420"));
        assert!(result.contains("Exact geometric mean: 387.4"));
        assert!(result.contains("Estimation method result: 400"));
        assert!(result.contains("Time taken: 12.3 seconds"));
        assert!(result.contains("✓ CORRECT! You calculated the estimation method properly."));
    }

    #[test]
    fn test_format_results_display_excellent() {
        let result = format_results_display(
            410,
            417.3,
            400,
            Duration::from_millis(5100),
            AnswerEvaluation::Excellent,
        );

        assert!(result.contains("Your answer: 410"));
        assert!(result.contains("Exact geometric mean: 417.3"));
        assert!(result.contains("Estimation method result: 400"));
        assert!(result.contains("Time taken: 5.1 seconds"));
        assert!(result.contains("★ EXCELLENT! Your answer is closer to the exact value than the estimation method!"));
    }

    #[test]
    fn test_format_results_display_incorrect() {
        let result = format_results_display(
            2000,
            346.4,
            400,
            Duration::from_millis(8700),
            AnswerEvaluation::Incorrect,
        );

        assert!(result.contains("Your answer: 2,000"));
        assert!(result.contains("Exact geometric mean: 346.4"));
        assert!(result.contains("Estimation method result: 400"));
        assert!(result.contains("Time taken: 8.7 seconds"));
        assert!(result.contains("You have calculated the estimation method incorrectly."));
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(123), "123");
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(12345), "12,345");
        assert_eq!(format_number(123456), "123,456");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(1000000000), "1,000,000,000");
    }

    #[test]
    fn test_parse_user_input_valid() {
        assert_eq!(parse_user_input("42"), Ok(42));
        assert_eq!(parse_user_input("  100  "), Ok(100));
        assert_eq!(parse_user_input("1,000"), Ok(1000));
        assert_eq!(parse_user_input("1,234,567"), Ok(1234567));
    }

    #[test]
    fn test_parse_user_input_invalid() {
        assert!(parse_user_input("").is_err());
        assert!(parse_user_input("   ").is_err());
        assert!(parse_user_input("abc").is_err());
        assert!(parse_user_input("-5").is_err());
        assert!(parse_user_input("1.5").is_err());
        assert!(parse_user_input("0").is_err());
    }

    #[test]
    fn test_parse_user_input_error_messages() {
        assert!(parse_user_input("").unwrap_err().contains("Please enter a number"));
        assert!(parse_user_input("abc").unwrap_err().contains("Please enter a valid number"));
        assert!(parse_user_input("-5").unwrap_err().contains("Please enter a positive number"));
        assert!(parse_user_input("1.5").unwrap_err().contains("Please enter a whole number"));
        assert!(parse_user_input("0").unwrap_err().contains("Please enter a positive number"));
    }

    // Property test: All integers converted to strings parse without error
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use quickcheck_macros::quickcheck;

        #[quickcheck]
        fn prop_all_positive_integers_parse_correctly(n: u64) -> bool {
            let n = n.max(1); // Ensure positive
            let formatted = format_number(n);
            parse_user_input(&formatted).unwrap() == n
        }
    }
}