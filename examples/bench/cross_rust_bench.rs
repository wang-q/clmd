//! Cross-Rust benchmark runner
//!
//! This binary compares clmd with other Rust Markdown parsers:
//! - comrak
//! - pulldown-cmark
//!
//! Usage:
//!   cargo build --release --example cross_rust_bench --features "comrak pulldown-cmark"
//!   ./target/release/examples/cross_rust_bench <markdown-file>

#[cfg(feature = "comrak")]
use comrak::{markdown_to_html as comrak_to_html, Options as ComrakOptions};
#[cfg(feature = "pulldown-cmark")]
use pulldown_cmark::{html, Parser};

use clmd::{markdown_to_html as clmd_to_html, Options as ClmdOptions};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <markdown-file>", args[0]);
        std::process::exit(1);
    }

    let input = fs::read_to_string(&args[1]).expect("Failed to read file");

    // Run clmd benchmark
    let clmd_opts = ClmdOptions::default();
    let _ = clmd_to_html(&input, &clmd_opts);

    // Run comrak benchmark (if enabled)
    #[cfg(feature = "comrak")]
    {
        let _ = comrak_to_html(&input, &ComrakOptions::default());
    }

    // Run pulldown-cmark benchmark (if enabled)
    #[cfg(feature = "pulldown-cmark")]
    {
        let parser = Parser::new(&input);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
    }
}
