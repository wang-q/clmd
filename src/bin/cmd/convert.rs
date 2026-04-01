use clap::{Arg, ArgAction, ArgMatches, Command};
use std::io::Read;
use std::path::Path;

use crate::cmd::utils;
use clmd::context::PureContext;
use clmd::io::reader::ReaderRegistry;
use clmd::io::writer::WriterRegistry;
use clmd::options::{ReaderOptions, WriterOptions};

/// Make the convert subcommand.
pub fn make_subcommand() -> Command {
    Command::new("convert")
        .about("Convert between Markdown and other formats")
        .subcommand_required(true)
        .subcommand(make_to_subcommand())
        .subcommand(make_from_subcommand())
}

/// Execute the convert subcommand.
pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some(("to", sub_matches)) => execute_to(sub_matches, options),
        Some(("from", sub_matches)) => execute_from(sub_matches, options),
        _ => unreachable!(),
    }
}

// ============================================================================
// to subcommand - unified output format conversion
// ============================================================================

fn make_to_subcommand() -> Command {
    Command::new("to")
        .about("Convert Markdown to various output formats")
        .arg(
            Arg::new("format")
                .help("Output format (html, xml, latex, man, typst, pdf, docx, epub, rtf, commonmark)")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("input")
                .help("Input Markdown file (default: stdin)")
                .index(2),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file (default: stdout, or auto-detect from input filename)"),
        )
        .arg(
            Arg::new("full")
                .long("full")
                .action(ArgAction::SetTrue)
                .help("Generate full document with proper headers (HTML only)"),
        )
        .arg(
            Arg::new("hardbreaks")
                .long("hardbreaks")
                .action(ArgAction::SetTrue)
                .help("Convert newlines to <br> (HTML only)"),
        )
        .arg(
            Arg::new("width")
                .long("width")
                .default_value("80")
                .help("Line width for wrapping (CommonMark output)"),
        )
        .after_help(
            r###"Supported output formats:
  html        - HTML format
  xml         - CommonMark XML format
  latex       - LaTeX format
  man         - Man page format
  typst       - Typst format
  pdf         - PDF format
  docx        - Microsoft Word format
  epub        - EPUB e-book format
  rtf         - Rich Text Format
  commonmark  - CommonMark (Markdown) format

Examples:
  clmd convert to html input.md -o output.html
  clmd convert to latex input.md -o output.tex
  clmd convert to pdf input.md
  cat input.md | clmd convert to html
"###,
        )
}

fn execute_to(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap();
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());

    // Read input
    let input = utils::read_input(input_path)?;

    // Parse the document
    let (arena, root) = clmd::parse_document(&input, options);

    // Create context and writer options
    let ctx = PureContext::new();
    let writer_options = WriterOptions::default();

    // Handle format-specific options
    let output = match format {
        "html" => {
            let mut opts = options.clone();
            if matches.get_flag("hardbreaks") {
                opts.render.hardbreaks = true;
            }

            let html = clmd::render::format::html::render(&arena, root, &opts);

            if matches.get_flag("full") {
                format!(
                    r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>Markdown Document</title>
</head>
<body>
{}
</body>
</html>"#,
                    html
                )
            } else {
                html
            }
        }
        "xml" => clmd::render::renderer::render_to_xml(&arena, root, 0),
        "latex" | "tex" => clmd::render::format::latex::render(&arena, root, 0),
        "man" => clmd::render::format::man::render(&arena, root, 0),
        "typst" => clmd::markdown_to_typst(&input, options),
        "commonmark" | "markdown" | "md" => {
            let width = matches
                .get_one::<String>("width")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(80);
            clmd::render::commonmark::render(&arena, root, 0, width)
        }
        _ => {
            // Try to use the writer registry for other formats
            let registry = WriterRegistry::new();
            if let Some(writer) = registry.get_by_name(format) {
                writer.write(&arena, root, &ctx, &writer_options)?
            } else {
                return Err(anyhow::anyhow!("Unsupported output format: {}", format));
            }
        }
    };

    // Determine output path
    let output_path = if let Some(path) = matches.get_one::<String>("output") {
        Some(path.clone())
    } else if let Some(input) = input_path {
        // Auto-generate output filename based on input
        let input_path = Path::new(input);
        let stem = input_path.file_stem().unwrap_or_default();
        let ext = match format {
            "html" => "html",
            "xml" => "xml",
            "latex" | "tex" => "tex",
            "man" => "man",
            "typst" => "typ",
            "pdf" => "pdf",
            "docx" => "docx",
            "epub" => "epub",
            "rtf" => "rtf",
            "commonmark" | "markdown" | "md" => "md",
            _ => format,
        };
        let output_name = format!("{}.{}", stem.to_string_lossy(), ext);
        Some(output_name)
    } else {
        None
    };

    utils::write_output(output_path.as_deref(), &output)
}

// ============================================================================
// from subcommand - unified input format conversion
// ============================================================================

fn make_from_subcommand() -> Command {
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
  clmd convert from html input.html -o output.md
  clmd convert from latex input.tex
  cat input.html | clmd convert from html
"###,
        )
}

fn execute_from(matches: &ArgMatches, _options: &clmd::Options) -> anyhow::Result<()> {
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
                clmd::render::commonmark::render(&arena, root, 0, 80)
            } else {
                return Err(anyhow::anyhow!("Unsupported input format: {}", format));
            }
        }
    };

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)
}
