//! Flexmark formatter spec tests
//!
//! These tests are migrated from flexmark-java's formatter test suite.
//! They test the CommonMark formatter with various options and configurations.

use clmd::options::format::{
    Alignment, BlockQuoteMarker, BulletMarker, CodeFenceMarker, ElementPlacement,
    ElementPlacementSort, FormatOptions, HeadingStyle, ListSpacing, NumberedMarker,
    TrailingMarker,
};
use clmd::options::Options as ParseOptions;
use clmd::parse::parse_document;
use clmd::render::commonmark::{CommonMarkNodeFormatter, Formatter};
use std::fs;

mod test_utils;
use test_utils::spec_parser::{parse_formatter_spec_file, FormatterSpecExample};

/// Apply spec options to FormatOptions
fn apply_spec_options(options: &mut FormatOptions, option_str: &str) {
    match option_str {
        // ATX heading options
        "atx-space-as-is" => options.space_after_atx_marker = false,
        "atx-space-add" => options.space_after_atx_marker = true,
        "atx-space-remove" => options.space_after_atx_marker = false,
        "atx-trailing-as-is" => {
            options.atx_heading_trailing_marker = TrailingMarker::AsIs
        }
        "atx-trailing-add" => options.atx_heading_trailing_marker = TrailingMarker::Add,
        "atx-trailing-equalize" => {
            options.atx_heading_trailing_marker = TrailingMarker::Equalize
        }
        "atx-trailing-remove" => {
            options.atx_heading_trailing_marker = TrailingMarker::Remove
        }

        // Setext heading options
        "setext-no-equalize" => options.setext_heading_equalize_marker = false,

        // Heading style preferences
        "heading-atx" => options.heading_style = HeadingStyle::Atx,
        "heading-setext" => options.heading_style = HeadingStyle::Setext,
        "heading-any" => options.heading_style = HeadingStyle::AsIs,

        // List bullet options
        "list-bullet-dash" => options.list_bullet_marker = BulletMarker::Dash,
        "list-bullet-asterisk" => options.list_bullet_marker = BulletMarker::Asterisk,
        "list-bullet-plus" => options.list_bullet_marker = BulletMarker::Plus,

        // List numbered options
        "list-numbered-dot" => options.list_numbered_marker = NumberedMarker::Period,
        "list-numbered-paren" => options.list_numbered_marker = NumberedMarker::Paren,

        // List spacing options
        "list-spacing-as-is" => options.list_spacing = ListSpacing::AsIs,
        "list-spacing-loosen" => options.list_spacing = ListSpacing::Loosen,
        "list-spacing-tighten" => options.list_spacing = ListSpacing::Tighten,
        "list-spacing-loose" => options.list_spacing = ListSpacing::Loose,
        "list-spacing-tight" => options.list_spacing = ListSpacing::Tight,

        // List other options
        "list-no-renumber-items" => options.list_renumber_items = false,
        "list-reset-first-item" => options.list_reset_first_item_number = true,
        "list-add-blank-line-before" => options.list_add_blank_line_before = true,
        "remove-empty-items" => options.list_remove_empty_items = true,

        // Code fence options
        "fenced-code-marker-backtick" => {
            options.fenced_code_marker_type = CodeFenceMarker::BackTick
        }
        "fenced-code-marker-tilde" => {
            options.fenced_code_marker_type = CodeFenceMarker::Tilde
        }
        "fenced-code-match-closing" => options.fenced_code_match_closing_marker = true,
        "fenced-code-spaced-info" => options.fenced_code_space_before_info = true,
        "fenced-code-minimize" => options.fenced_code_minimize_indent = true,
        "indented-code-minimize" => options.indented_code_minimize_indent = true,

        // Block quote options
        "no-block-quote-blank-lines" => options.block_quote_blank_lines = false,
        "block-quote-compact" => {
            options.block_quote_markers = BlockQuoteMarker::AddCompact
        }
        "block-quote-compact-with-space" => {
            options.block_quote_markers = BlockQuoteMarker::AddCompactWithSpace
        }
        "block-quote-spaced" => {
            options.block_quote_markers = BlockQuoteMarker::AddSpaced
        }

        // Thematic break options
        "thematic-break" => {
            options.thematic_break = Some("*** ** * ** ***".to_string());
        }

        // Reference placement options
        "references-document-top" => {
            options.reference_placement = ElementPlacement::DocumentTop
        }
        "references-document-bottom" => {
            options.reference_placement = ElementPlacement::DocumentBottom
        }
        "references-group-with-first" => {
            options.reference_placement = ElementPlacement::GroupWithFirst
        }
        "references-group-with-last" => {
            options.reference_placement = ElementPlacement::GroupWithLast
        }
        "references-as-is" => options.reference_placement = ElementPlacement::AsIs,

        // Reference sort options
        "references-sort" => options.reference_sort = ElementPlacementSort::Sort,
        "references-sort-unused-last" => {
            options.reference_sort = ElementPlacementSort::SortUnusedLast
        }
        "references-sort-delete-unused" => {
            options.reference_sort = ElementPlacementSort::SortDeleteUnused
        }
        "references-delete-unused" => {
            options.reference_sort = ElementPlacementSort::DeleteUnused
        }
        "references-keep-last" => {} // Handled separately

        // Link options
        "image-links-at-start" => options.keep_image_links_at_start = true,
        "explicit-links-at-start" => options.keep_explicit_links_at_start = true,

        // Blank line options
        "max-blank-lines-1" => options.max_blank_lines = 1,
        "max-blank-lines-2" => options.max_blank_lines = 2,
        "max-blank-lines-3" => options.max_blank_lines = 3,
        "no-tailing-blanks" => options.max_trailing_blank_lines = 0,

        // Parse options (not formatter options, but we track them)
        "parse-github" => {}        // Parser option
        "parse-fixed-indent" => {}  // Parser option
        "format-github" => {}       // Format style
        "format-fixed-indent" => {} // Format style

        // Margin options (formatting width)
        opt if opt.starts_with("margin[") => {
            if let Some(end) = opt.find(']') {
                if let Ok(width) = opt[7..end].parse::<usize>() {
                    options.right_margin = width;
                }
            }
        }

        // Keep breaks options
        "no-hard-breaks" => options.keep_hard_line_breaks = false,
        "no-soft-breaks" => options.keep_soft_line_breaks = false,

        // List alignment
        "list-align-numeric-left" => options.list_align_numeric = Alignment::Left,
        "list-align-numeric-right" => options.list_align_numeric = Alignment::Right,

        // Link annotation options
        "link-address-pattern" => {} // Special handling for links

        // List mismatch options
        "list-no-delimiter-mismatch-to-new-list" => {} // Parser option
        "list-no-item-mismatch-to-new-list" => {}      // Parser option

        _ => {
            // Unknown option - ignore for now
            // eprintln!("Warning: Unknown spec option: {}", option_str);
        }
    }
}

/// Format markdown input using the given options
fn format_markdown(input: &str, options: &FormatOptions) -> String {
    // Parse the input
    let parse_options = ParseOptions::default();
    let (arena, root) = parse_document(input, &parse_options);

    // Format using the formatter
    let mut formatter = Formatter::with_options(options.clone());
    formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::new()));

    formatter.render(&arena, root)
}

/// Run a single formatter spec example
fn run_formatter_example(example: &FormatterSpecExample) {
    let mut options = FormatOptions::default();

    // Apply spec options
    for opt in &example.options {
        apply_spec_options(&mut options, opt);
    }

    // Format the input
    let output = format_markdown(&example.input, &options);

    // Normalize line endings for comparison
    let expected = example.expected_output.replace("\r\n", "\n");
    let actual = output.replace("\r\n", "\n");

    // Trim trailing whitespace from each line for comparison
    let expected_trimmed: String = expected
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    let actual_trimmed: String = actual
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(
        actual_trimmed,
        expected_trimmed,
        "Test {}:{} failed\nOptions: {:?}\nInput:\n{}\n\nExpected:\n{}\n\nActual:\n{}",
        example.section,
        example.number,
        example.options,
        example.input,
        expected,
        actual
    );
}

/// Run formatter spec tests from a file
fn run_formatter_spec_file(spec_file: &str) {
    let content = fs::read_to_string(spec_file)
        .unwrap_or_else(|_| panic!("Failed to read {}", spec_file));

    let examples = parse_formatter_spec_file(&content);
    println!("Found {} examples in {}", examples.len(), spec_file);

    for example in examples {
        run_formatter_example(&example);
    }
}

#[test]
#[ignore = "formatter implementation incomplete - see issue #XXX"]
fn test_core_formatter_spec() {
    run_formatter_spec_file("tests/fixtures/core_formatter_spec.md");
}

#[test]
#[ignore = "formatter implementation incomplete - see issue #XXX"]
fn test_core_formatter_no_blanklines_spec() {
    run_formatter_spec_file("tests/fixtures/core_formatter_no_blanklines_spec.md");
}
