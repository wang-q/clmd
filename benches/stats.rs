//! Statistics utilities for benchmarks
//!
//! Provides statistical functions for analyzing benchmark results.

/// Calculate the mean (average) of a slice of f64 values
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Calculate the median of a slice of f64 values
pub fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let len = sorted.len();
    if len % 2 == 0 {
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
    } else {
        sorted[len / 2]
    }
}

/// Calculate the population standard deviation
pub fn stdev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let mean_val = mean(values);
    let variance = values.iter().map(|&x| (x - mean_val).powi(2)).sum::<f64>()
        / values.len() as f64;

    variance.sqrt()
}

/// Calculate the sample standard deviation
pub fn sample_stdev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let mean_val = mean(values);
    let variance = values.iter().map(|&x| (x - mean_val).powi(2)).sum::<f64>()
        / (values.len() - 1) as f64;

    variance.sqrt()
}

/// Calculate min and max values
pub fn min_max(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }

    let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    (min, max)
}

/// Format a duration in nanoseconds to a human-readable string
pub fn format_duration(ns: f64) -> String {
    if ns < 1_000.0 {
        format!("{:.2} ns", ns)
    } else if ns < 1_000_000.0 {
        format!("{:.2} µs", ns / 1_000.0)
    } else if ns < 1_000_000_000.0 {
        format!("{:.2} ms", ns / 1_000_000.0)
    } else {
        format!("{:.2} s", ns / 1_000_000_000.0)
    }
}

/// Print a statistical summary of benchmark results
pub fn print_summary(name: &str, values: &[f64]) {
    if values.is_empty() {
        println!("{}: No data", name);
        return;
    }

    let mean_val = mean(values);
    let median_val = median(values);
    let stdev_val = stdev(values);
    let (min_val, max_val) = min_max(values);

    println!("\n=== {} ===", name);
    println!("  Count:    {}", values.len());
    println!("  Mean:     {}", format_duration(mean_val));
    println!("  Median:   {}", format_duration(median_val));
    println!("  StdDev:   {}", format_duration(stdev_val));
    println!("  Min:      {}", format_duration(min_val));
    println!("  Max:      {}", format_duration(max_val));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(mean(&values), 3.0);
    }

    #[test]
    fn test_median_odd() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(median(&values), 3.0);
    }

    #[test]
    fn test_median_even() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(median(&values), 2.5);
    }

    #[test]
    fn test_stdev() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let result = stdev(&values);
        // Population stdev should be approximately 2.0
        assert!((result - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_min_max() {
        let values = vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
        let (min, max) = min_max(&values);
        assert_eq!(min, 1.0);
        assert_eq!(max, 9.0);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500.0), "500.00 ns");
        assert_eq!(format_duration(5_000.0), "5.00 µs");
        assert_eq!(format_duration(5_000_000.0), "5.00 ms");
        assert_eq!(format_duration(5_000_000_000.0), "5.00 s");
    }
}
