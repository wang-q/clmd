//! Flamegraph benchmark - run this with: cargo flamegraph --example flamegraph_bench

use clmd::{markdown_to_html, options};

fn main() {
    // Use lorem1_full_document as it's the most comprehensive test
    let input = include_str!("../benches/samples/lorem1.md");

    // Run many iterations to get good profiling data
    for _ in 0..10000 {
        let _ = markdown_to_html(input, options::DEFAULT);
    }
}
