//! Comrak benchmark runner
//!
//! This binary runs the same benchmark as cmark and commonmark.js
//! for fair comparison.

use comrak::{markdown_to_html, Options};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <markdown-file>", args[0]);
        std::process::exit(1);
    }

    let input = fs::read_to_string(&args[1]).expect("Failed to read file");

    // Run once (hyperfine will handle iterations)
    let _ = markdown_to_html(&input, &Options::default());
}
