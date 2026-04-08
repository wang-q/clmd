//! Formatter spec tests
//!
//! Comprehensive tests for the CommonMark formatter output.

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
use test_utils::spec_parser::parse_formatter_spec_file;

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
        "references-keep-last" => {}

        // Link options
        "image-links-at-start" => options.keep_image_links_at_start = true,
        "explicit-links-at-start" => options.keep_explicit_links_at_start = true,

        // Blank line options
        "max-blank-lines-1" => options.max_blank_lines = 1,
        "max-blank-lines-2" => options.max_blank_lines = 2,
        "max-blank-lines-3" => options.max_blank_lines = 3,
        "no-tailing-blanks" => options.max_trailing_blank_lines = 0,

        // Format control
        "formatter-tags-enabled" => options.formatter_tags_enabled = true,

        // Parse options (not formatter options, but we track them)
        "parse-github" => {}
        "parse-fixed-indent" => {}
        "format-github" => {}
        "format-fixed-indent" => {}

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
        "link-address-pattern" => {}

        // List mismatch options
        "list-no-delimiter-mismatch-to-new-list" => {}
        "list-no-item-mismatch-to-new-list" => {}

        _ => {}
    }
}

/// Format markdown input using the given options
fn format_markdown(input: &str, options: &FormatOptions) -> String {
    let parse_options = ParseOptions::default();
    let (arena, root) = parse_document(input, &parse_options);

    let mut formatter = Formatter::with_options(options.clone());
    formatter.add_node_formatter(Box::new(CommonMarkNodeFormatter::with_options(
        options.clone(),
    )));

    formatter.render(&arena, root)
}

/// Run a single formatter spec example
fn run_formatter_example(example: &test_utils::spec_parser::FormatterSpecExample) {
    let mut options = FormatOptions::default();

    for opt in &example.options {
        apply_spec_options(&mut options, opt);
    }

    let output = format_markdown(&example.input, &options);

    let expected = example.expected_output.replace("\r\n", "\n");
    let actual = output.replace("\r\n", "\n");

    // Normalize both strings: trim trailing whitespace from each line and remove trailing newlines
    let normalize = |s: &str| -> String {
        s.lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_string()
    };

    let expected_normalized = normalize(&expected);
    let actual_normalized = normalize(&actual);

    assert_eq!(
        actual_normalized,
        expected_normalized,
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

    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<(String, usize, String, String, String)> = Vec::new();

    for example in &examples {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_formatter_example(example);
        })) {
            Ok(_) => passed += 1,
            Err(_) => {
                failed += 1;
                if failures.len() < 10 {
                    let mut options = FormatOptions::default();
                    for opt in &example.options {
                        apply_spec_options(&mut options, opt);
                    }
                    let output = format_markdown(&example.input, &options);
                    failures.push((
                        example.section.clone(),
                        example.number,
                        example.input.clone(),
                        example.expected_output.clone(),
                        output,
                    ));
                }
            }
        }
    }

    println!("\n=== Formatter Spec Test Results ===");
    println!(
        "Passed: {}/{} ({:.1}%)",
        passed,
        examples.len(),
        (passed as f64 / examples.len() as f64) * 100.0
    );
    println!(
        "Failed: {}/{} ({:.1}%)",
        failed,
        examples.len(),
        (failed as f64 / examples.len() as f64) * 100.0
    );

    if !failures.is_empty() {
        println!("\n=== Failed Tests ===");
        for (section, number, input, expected, actual) in &failures {
            println!("\n{}:{}", section, number);
            println!("Input:\n{}", input);
            println!("Expected:\n{}", expected);
            println!("Actual:\n{}", actual);
        }
    }

    assert!(
        failed == 0,
        "{} tests failed out of {}",
        failed,
        examples.len()
    );
}

#[test]
fn test_formatter_spec() {
    run_formatter_spec_file("tests/fixtures/formatter_spec.md");
}
