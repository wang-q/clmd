//! Flamegraph benchmark
//!
//! Run this with: cargo flamegraph --example flamegraph_bench
//!
//! This runs many iterations to get good profiling data for flamegraph generation.

use clmd::markdown_to_html;
use clmd::parser::options::Options;

fn main() {
    // Use lorem1_full_document as it's the most comprehensive test
    let input = include_str!("../../benches/samples/lorem1.md");

    let options = Options::default();

    // Run many iterations to get good profiling data
    for _ in 0..10000 {
        let _ = markdown_to_html(input, &options);
    }
}
