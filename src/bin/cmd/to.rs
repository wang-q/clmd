use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;

use crate::cmd::utils;
use clmd::context::{IoContext, PureContext};
use clmd::io::writer::WriterRegistry;
use clmd::options::WriterOptions;

/// Make the `to` subcommand.
pub fn make_subcommand() -> Command {
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
  clmd to html input.md -o output.html
  clmd to latex input.md -o output.tex
  clmd to pdf input.md
  cat input.md | clmd to html
"###,
        )
}

/// Execute the `to` subcommand.
pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap();
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());

    // Read input
    let input = utils::read_input(input_path)?;

    // Parse the document
    let (arena, root) = clmd::parse_document(&input, options);

    // Create writer options
    let writer_options = WriterOptions::default();

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

    // Check if this is a binary format that needs special handling
    let is_binary_format = matches!(format, "docx" | "epub");

    if is_binary_format {
        // For binary formats, use IoContext and write_to_file directly
        let ctx = IoContext::new();
        let registry = WriterRegistry::new();
        if let Some(writer) = registry.get_by_name(format) {
            let path = output_path.ok_or_else(|| {
                anyhow::anyhow!(
                    "Binary format '{}' requires an output file path. Use -o option.",
                    format
                )
            })?;
            writer
                .write_to_file(&arena, root, Path::new(&path), &ctx, &writer_options)
                .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", format, e))?;
            println!("Successfully converted to {}", path);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Unsupported output format: {}", format))
        }
    } else {
        // Handle text formats using PureContext
        let ctx = PureContext::new();
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
                    return Err(anyhow::anyhow!(
                        "Unsupported output format: {}",
                        format
                    ));
                }
            }
        };

        utils::write_output(output_path.as_deref(), &output)
    }
}
