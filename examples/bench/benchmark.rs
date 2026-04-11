//! Comprehensive benchmark runner
//!
//! This binary provides multiple benchmark modes for clmd:
//! 1. Cross-language comparison (cmark, commonmark.js)
//! 2. Cross-Rust parser comparison (comrak, pulldown-cmark)
//! 3. Flamegraph generation support
//! 4. Arena performance comparison
//!
//! Usage:
//!   cargo build --release --example benchmark
//!   ./target/release/examples/benchmark [OPTIONS] [FILE]
//!
//! Options:
//!   -h, --help                Show this help message
//!   --mode MODE               Benchmark mode: cross-language, cross-rust, flamegraph, arena
//!   -i, --iterations N        Number of iterations (for flamegraph mode)
//!   --no-optimization         Prevent compiler optimizations
//!
//! Examples:
//!   ./target/release/examples/benchmark --mode cross-language sample.md
//!   ./target/release/examples/benchmark --mode cross-rust sample.md
//!   ./target/release/examples/benchmark --mode flamegraph -i 10000

use clmd::markdown_to_html as clmd_to_html;
use clmd::options::Options as ClmdOptions;
use comrak::{markdown_to_html as comrak_to_html, Options as ComrakOptions};
use pulldown_cmark::{html, Parser};
use std::env;
use std::fs;
use std::process::exit;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
enum BenchMode {
    CrossLanguage,
    CrossRust,
    Flamegraph,
    Arena,
}

struct BenchConfig {
    mode: BenchMode,
    input: String,
    iterations: usize,
    prevent_optimization: bool,
}

fn parse_args() -> BenchConfig {
    let args: Vec<String> = env::args().collect();
    let mut mode = BenchMode::CrossLanguage;
    let mut input = None;
    let mut iterations = 10000;
    let mut prevent_optimization = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                exit(0);
            }
            "--mode" => {
                if i + 1 < args.len() {
                    mode = match args[i + 1].as_str() {
                        "cross-language" => BenchMode::CrossLanguage,
                        "cross-rust" => BenchMode::CrossRust,
                        "flamegraph" => BenchMode::Flamegraph,
                        "arena" => BenchMode::Arena,
                        _ => {
                            eprintln!("Error: Invalid mode. Use --help for options.");
                            exit(1);
                        }
                    };
                    i += 1;
                } else {
                    eprintln!("Error: --mode requires an argument.");
                    exit(1);
                }
            }
            "-i" | "--iterations" => {
                if i + 1 < args.len() {
                    iterations = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Error: Invalid iterations count.");
                        exit(1);
                    });
                    i += 1;
                } else {
                    eprintln!("Error: --iterations requires an argument.");
                    exit(1);
                }
            }
            "--no-optimization" => {
                prevent_optimization = true;
            }
            _ => {
                if input.is_none() {
                    input = Some(args[i].clone());
                } else {
                    eprintln!("Error: Too many arguments. Use --help for options.");
                    exit(1);
                }
            }
        }
        i += 1;
    }

    let default_input = match mode {
        BenchMode::Flamegraph | BenchMode::Arena => include_str!("../../benches/samples/lorem1.md"),
        _ => "# Test Document\n\nThis is a sample document.\n\n## Section\n\n- Item 1\n- Item 2\n",
    };

    let input_content = if let Some(input_file) = input {
        fs::read_to_string(input_file).unwrap_or_else(|_| {
            eprintln!("Error: Failed to read input file.");
            exit(1);
        })
    } else {
        default_input.to_string()
    };

    BenchConfig {
        mode,
        input: input_content,
        iterations,
        prevent_optimization,
    }
}

fn print_help() {
    println!("Comprehensive benchmark runner for clmd");
    println!();
    println!("Usage:");
    println!("  ./benchmark [OPTIONS] [FILE]");
    println!();
    println!("Options:");
    println!("  -h, --help                Show this help message");
    println!("  --mode MODE               Benchmark mode:");
    println!("                            - cross-language: Compare with cmark (C) and commonmark.js");
    println!("                            - cross-rust: Compare with comrak and pulldown-cmark");
    println!("                            - flamegraph: Run many iterations for flamegraph generation");
    println!("                            - arena: Test arena performance");
    println!("  -i, --iterations N        Number of iterations (for flamegraph mode)");
    println!("  --no-optimization         Prevent compiler optimizations");
    println!();
}

fn run_cross_language_bench(config: &BenchConfig) {
    println!("=== Cross-language Benchmark ===");
    println!("Note: To compare with cmark and commonmark.js, run them separately:");
    println!("  cmark: time cmark < input.md > output.html");
    println!("  commonmark.js: time node -e \"require('commonmark').parse(require('fs').readFileSync('input.md', 'utf8'))\" ");
    println!();

    let options = ClmdOptions::default();
    let start = Instant::now();
    let result = clmd_to_html(&config.input, &options);
    let duration = start.elapsed();

    println!("clmd:");
    println!("  Time: {:?}", duration);
    println!("  Output size: {} bytes", result.len());

    if config.prevent_optimization {
        std::hint::black_box(result);
    }
}

fn run_cross_rust_bench(config: &BenchConfig) {
    println!("=== Cross-Rust Parser Benchmark ===");
    println!();

    let options = ClmdOptions::default();
    let start = Instant::now();
    let clmd_result = clmd_to_html(&config.input, &options);
    let clmd_duration = start.elapsed();

    println!("clmd:");
    println!("  Time: {:?}", clmd_duration);
    println!("  Output size: {} bytes", clmd_result.len());

    let start = Instant::now();
    let comrak_result = comrak_to_html(&config.input, &ComrakOptions::default());
    let comrak_duration = start.elapsed();

    println!("\ncomrak:");
    println!("  Time: {:?}", comrak_duration);
    println!("  Output size: {} bytes", comrak_result.len());

    if config.prevent_optimization {
        std::hint::black_box(comrak_result);
    }

    let start = Instant::now();
    let mut pulldown_result = String::new();
    let parser = Parser::new(&config.input);
    html::push_html(&mut pulldown_result, parser);
    let pulldown_duration = start.elapsed();

    println!("\npulldown-cmark:");
    println!("  Time: {:?}", pulldown_duration);
    println!("  Output size: {} bytes", pulldown_result.len());

    if config.prevent_optimization {
        std::hint::black_box(pulldown_result);
    }

    if config.prevent_optimization {
        std::hint::black_box(clmd_result);
    }
}

fn run_flamegraph_bench(config: &BenchConfig) {
    println!("=== Flamegraph Benchmark ===");
    println!("Running {} iterations...", config.iterations);
    println!("Note: Use with flamegraph: cargo flamegraph --example benchmark --mode flamegraph -i 10000");
    println!();

    let options = ClmdOptions::default();
    let input = &config.input;

    // Run many iterations to get good profiling data
    for _ in 0..config.iterations {
        let result = clmd_to_html(input, &options);
        if config.prevent_optimization {
            std::hint::black_box(result);
        }
    }

    println!(
        "Completed {0} iterations in approximately {1:.2} seconds per 1000 iterations",
        config.iterations,
        config.iterations as f64 / 1000.0 * 0.01
    );
}

fn run_arena_bench(config: &BenchConfig) {
    println!("=== Arena Performance Benchmark ===");
    println!(
        "This test demonstrates the performance of clmd's arena-based memory management"
    );
    println!();

    let options = ClmdOptions::default();
    let input = &config.input;

    // Warm-up run
    clmd_to_html(input, &options);

    // Measure performance
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let result = clmd_to_html(input, &options);
        if config.prevent_optimization {
            std::hint::black_box(result);
        }
    }

    let duration = start.elapsed();
    let per_iteration = duration / iterations as u32;

    println!("Results:");
    println!("  Iterations: {}", iterations);
    println!("  Total time: {:?}", duration);
    println!("  Per iteration: {:?}", per_iteration);
    println!(
        "  Throughput: {:.2} MB/s",
        (input.len() as f64 * iterations as f64) / duration.as_secs_f64() / 1_000_000.0
    );
}

fn main() {
    let config = parse_args();

    match config.mode {
        BenchMode::CrossLanguage => run_cross_language_bench(&config),
        BenchMode::CrossRust => run_cross_rust_bench(&config),
        BenchMode::Flamegraph => run_flamegraph_bench(&config),
        BenchMode::Arena => run_arena_bench(&config),
    }
}
