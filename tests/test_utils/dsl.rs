//! Test DSL for clmd - inspired by pandoc's test framework
//!
//! This module provides a domain-specific language for writing Markdown parser tests.
//! It is inspired by pandoc's Haskell test DSL using `(=:)` and `(=?>)` operators.
//!
//! # Example
//!
//! ```ignore
//! use crate::test_utils::dsl::*;
//! use clmd::Options;
//!
//! // Define a test case
//! test_case!("ATX heading", "# Hello", "<h1>Hello</h1>");
//!
//! // Or using the macro directly
//! test! {
//!     name: "emphasis",
//!     input: "*hello*",
//!     expected: "<p><em>hello</em></p>"
//! }
//! ```

use clmd::markdown_to_html;
use clmd::options::Options;
use std::fmt;

/// A test case result
#[derive(Debug)]
pub struct TestResult {
    /// Name of the test case
    pub name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Input markdown text
    pub input: String,
    /// Expected output
    pub expected: String,
    /// Actual output
    pub actual: String,
    /// Diff between expected and actual (if different)
    pub diff: Option<String>,
}

impl TestResult {
    /// Create a new test result
    pub fn new(name: &str, input: &str, expected: &str, actual: &str) -> Self {
        let diff = if expected != actual {
            Some(generate_diff(expected, actual))
        } else {
            None
        };

        Self {
            name: name.to_string(),
            passed: expected == actual,
            input: input.to_string(),
            expected: expected.to_string(),
            actual: actual.to_string(),
            diff,
        }
    }
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.passed {
            writeln!(f, "✓ {}", self.name)
        } else {
            writeln!(f, "✗ {}", self.name)?;
            writeln!(f, "  Input:    {:?}", self.input)?;
            writeln!(f, "  Expected: {:?}", self.expected)?;
            writeln!(f, "  Actual:   {:?}", self.actual)?;
            if let Some(ref diff) = self.diff {
                writeln!(f, "  Diff:\n{}", diff)?;
            }
            Ok(())
        }
    }
}

/// Generate a unified diff between expected and actual output
fn generate_diff(expected: &str, actual: &str) -> String {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    let mut diff = String::new();
    let mut line_num = 1;

    // Simple line-by-line diff
    let max_lines = expected_lines.len().max(actual_lines.len());

    for i in 0..max_lines {
        let exp = expected_lines.get(i).unwrap_or(&"");
        let act = actual_lines.get(i).unwrap_or(&"");

        if exp != act {
            if !exp.is_empty() {
                diff.push_str(&format!("- {:4} {:?}\n", line_num, exp));
            }
            if !act.is_empty() {
                diff.push_str(&format!("+ {:4} {:?}\n", line_num, act));
            }
        }
        line_num += 1;
    }

    diff
}

/// Run a single test case and return the result
pub fn run_test(
    name: &str,
    input: &str,
    expected: &str,
    options: &Options,
) -> TestResult {
    let actual = markdown_to_html(input, options);
    // Normalize trailing newlines for comparison
    let actual = actual.trim_end_matches('\n').to_string();
    let expected = expected.trim_end_matches('\n').to_string();

    TestResult::new(name, input, &expected, &actual)
}

/// Run a test case with custom options
#[allow(dead_code)]
pub fn run_test_with_options<F>(
    name: &str,
    input: &str,
    expected: &str,
    options_fn: F,
) -> TestResult
where
    F: FnOnce(&mut Options),
{
    let mut options = Options::default();
    options_fn(&mut options);
    run_test(name, input, expected, &options)
}

/// A group of related test cases
#[derive(Debug)]
#[allow(dead_code)]
pub struct TestGroup {
    /// Name of the test group
    pub name: String,
    /// Test results in this group
    pub results: Vec<TestResult>,
}

impl TestGroup {
    /// Create a new test group
    #[allow(dead_code)]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            results: Vec::new(),
        }
    }

    /// Add a test result to the group
    #[allow(dead_code)]
    pub fn add(&mut self, result: TestResult) {
        self.results.push(result);
    }

    /// Check if all tests in the group passed
    #[allow(dead_code)]
    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|r| r.passed)
    }

    /// Get the number of passed tests
    #[allow(dead_code)]
    pub fn passed_count(&self) -> usize {
        self.results.iter().filter(|r| r.passed).count()
    }

    /// Get the number of failed tests
    #[allow(dead_code)]
    pub fn failed_count(&self) -> usize {
        self.results.iter().filter(|r| !r.passed).count()
    }
}

impl fmt::Display for TestGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n=== {} ===", self.name)?;
        for result in &self.results {
            write!(f, "{}", result)?;
        }
        writeln!(
            f,
            "Results: {}/{} passed",
            self.passed_count(),
            self.results.len()
        )
    }
}

/// Macro for defining a simple test case
///
/// # Example
///
/// ```ignore
/// test_case!("ATX heading", "# Hello", "<h1>Hello</h1>");
/// ```
#[macro_export]
macro_rules! test_case {
    ($name:expr, $input:expr, $expected:expr) => {
        crate::test_utils::dsl::run_test(
            $name,
            $input,
            $expected,
            &clmd::Options::default(),
        )
    };
    ($name:expr, $input:expr, $expected:expr, $options:expr) => {
        crate::test_utils::dsl::run_test($name, $input, $expected, $options)
    };
}

/// Macro for defining a test with a block
///
/// # Example
///
/// ```ignore
/// test! {
///     name: "emphasis",
///     input: "*hello*",
///     expected: "<p><em>hello</em></p>"
/// }
/// ```
#[macro_export]
macro_rules! test {
    (
        name: $name:expr,
        input: $input:expr,
        expected: $expected:expr
    ) => {
        crate::test_utils::dsl::run_test(
            $name,
            $input,
            $expected,
            &clmd::Options::default(),
        )
    };
    (
        name: $name:expr,
        input: $input:expr,
        expected: $expected:expr,
        options: $options:expr
    ) => {
        crate::test_utils::dsl::run_test($name, $input, $expected, $options)
    };
}

/// Macro for defining a test group
///
/// # Example
///
/// ```ignore
/// test_group! {
///     name: "headers",
///     tests: [
///         ("ATX", "# Hello", "<h1>Hello</h1>"),
///         ("Setext", "Hello\n===", "<h1>Hello</h1>"),
///     ]
/// }
/// ```
#[macro_export]
macro_rules! test_group {
    (
        name: $name:expr,
        tests: [$(($test_name:expr, $input:expr, $expected:expr)),*$(,)?]
    ) => {{
        let mut group = crate::test_utils::dsl::TestGroup::new($name);
        $(
            let result = crate::test_utils::dsl::run_test(
                $test_name,
                $input,
                $expected,
                &clmd::Options::default()
            );
            group.add(result);
        )*
        group
    }};
}

/// Helper function to normalize HTML for comparison
pub fn normalize_html(html: &str) -> String {
    // Normalize whitespace
    let mut result = String::new();
    let mut prev_was_space = true;

    for c in html.chars() {
        if c.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(c);
            prev_was_space = false;
        }
    }

    result.trim().to_string()
}

/// Compare HTML outputs with normalization
#[allow(dead_code)]
pub fn html_equals(expected: &str, actual: &str) -> bool {
    normalize_html(expected) == normalize_html(actual)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_generation() {
        let expected = "line1\nline2\nline3";
        let actual = "line1\nmodified\nline3";
        let diff = generate_diff(expected, actual);
        assert!(diff.contains("-"));
        assert!(diff.contains("+"));
    }

    #[test]
    fn test_html_normalization() {
        let html1 = "<p>  hello   world  </p>";
        let html2 = "<p> hello world </p>";
        assert_eq!(normalize_html(html1), normalize_html(html2));
    }

    #[test]
    fn test_run_test_pass() {
        let result =
            run_test("simple", "# Hello", "<h1>Hello</h1>", &Options::default());
        assert!(result.passed);
    }

    #[test]
    fn test_run_test_fail() {
        let result = run_test("wrong", "# Hello", "<h2>Hello</h2>", &Options::default());
        assert!(!result.passed);
        assert!(result.diff.is_some());
    }
}
