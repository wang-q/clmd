use clap::{Arg, ArgMatches, Command};
use std::io::Read;
use std::path::Path;

use crate::cmd::utils;
use clmd::io::reader::ReaderRegistry;
use clmd::options::ReaderOptions;

/// Make the `from` subcommand.
pub fn make_subcommand() -> Command {
    Command::new("from")
        .about("Convert from other formats to Markdown")
        .arg(
            Arg::new("format")
                .help("Input format (html, latex, bibtex)")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("input")
                .help("Input file (default: stdin)")
                .index(2),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file (default: stdout)"),
        )
        .after_help(
            r###"Supported input formats:
  html    - HTML format
  latex   - LaTeX format
  bibtex  - BibTeX format

Examples:
  clmd from html input.html -o output.md
  clmd from latex input.tex
  cat input.html | clmd from html
"###,
        )
}

/// Execute the `from` subcommand.
pub fn execute(matches: &ArgMatches, _options: &clmd::Options) -> anyhow::Result<()> {
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap();
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());

    // For HTML, use the direct conversion function
    let output = match format {
        "html" | "htm" => {
            let input = utils::read_input(input_path)?;
            clmd::io::reader::html::html_to_markdown(&input)
        }
        _ => {
            // Try to use the reader registry
            let registry = ReaderRegistry::new();
            let reader_options = ReaderOptions::default();

            let input = if let Some(path) = input_path {
                let path = Path::new(path);
                std::fs::read_to_string(path).map_err(|e| {
                    anyhow::anyhow!("Failed to read file '{}': {}", path.display(), e)
                })?
            } else {
                let mut buffer = String::new();
                std::io::stdin()
                    .read_to_string(&mut buffer)
                    .map_err(|e| anyhow::anyhow!("Failed to read stdin: {}", e))?;
                buffer
            };

            if let Some(reader) = registry.get(format) {
                let (arena, root) = reader
                    .read(&input, &reader_options)
                    .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", format, e))?;

                // Convert to CommonMark
                clmd::render::commonmark::render(&arena, root, 80)
            } else {
                return Err(anyhow::anyhow!("Unsupported input format: {}", format));
            }
        }
    };

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)
}
