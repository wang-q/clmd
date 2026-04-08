//! Integration tests for line breaking functionality
//!
//! These tests verify the line breaking behavior using the public API.

use clmd::render::commonmark::line_breaking::LineBreakingContext;
use clmd::text::unicode_width;

// =============================================================================
// Paragraph Formatting Tests
// =============================================================================

#[test]
fn test_paragraph_multiple_lines() {
    let mut ctx = LineBreakingContext::new(20, 25);
    ctx.add_text("First line with some text. Second part with more text here.");

    let formatted = ctx.format();
    let lines: Vec<&str> = formatted.lines().collect();

    assert!(
        lines.len() >= 2,
        "Should have multiple lines, got: {:?}",
        lines
    );

    for line in &lines {
        let width = unicode_width::width(line) as usize;
        assert!(
            width <= 25,
            "Line exceeds max width: {} (width: {})",
            line,
            width
        );
    }
}

#[test]
fn test_paragraph_single_line() {
    let mut ctx = LineBreakingContext::new(20, 25);
    ctx.add_text("Short text");

    let formatted = ctx.format();
    let lines: Vec<&str> = formatted.lines().collect();

    assert_eq!(lines.len(), 1, "Short text should be on one line");
    assert_eq!(lines[0], "Short text");
}

#[test]
fn test_paragraph_optimal_break_points() {
    let mut ctx = LineBreakingContext::new(15, 20);
    ctx.add_text("The quick brown fox jumps over the lazy dog");

    let formatted = ctx.format();
    let lines: Vec<&str> = formatted.lines().collect();

    for line in &lines {
        let width = unicode_width::width(line) as usize;
        assert!(width <= 20, "Line exceeds max width: {}", line);
        assert!(width >= 7, "Line too short, not optimal: {}", line);
    }
}

#[test]
fn test_paragraph_with_mixed_content() {
    let mut ctx = LineBreakingContext::new(20, 25);
    ctx.add_text("A very longwordthatmightbeproblematic and then more text here");

    let formatted = ctx.format();

    assert!(!formatted.is_empty(), "Should produce some output");

    for line in formatted.lines() {
        let width = unicode_width::width(line) as usize;
        if !line.contains("longwordthatmightbeproblematic") {
            assert!(width <= 25, "Line exceeds max width: {}", line);
        }
    }
}

#[test]
fn test_paragraph_large_width() {
    let mut ctx = LineBreakingContext::new(100, 120);
    ctx.add_text("This is a paragraph that should fit on a single line because the max width is very large");

    let formatted = ctx.format();
    let lines: Vec<&str> = formatted.lines().collect();

    assert_eq!(lines.len(), 1, "Should fit on one line with large width");
}

#[test]
fn test_paragraph_small_width() {
    let mut ctx = LineBreakingContext::new(10, 12);
    ctx.add_text("This is a test");

    let formatted = ctx.format();
    let lines: Vec<&str> = formatted.lines().collect();

    assert!(
        lines.len() >= 2,
        "Should have multiple lines with small width"
    );

    for line in &lines {
        let width = unicode_width::width(line) as usize;
        assert!(width <= 12, "Line exceeds max width: {}", line);
    }
}

#[test]
fn test_paragraph_with_numbers_and_punctuation() {
    let mut ctx = LineBreakingContext::new(20, 25);
    ctx.add_text("Version 1.2.3 is released on 2024-01-15! Check it out.");

    let formatted = ctx.format();

    for line in formatted.lines() {
        let width = unicode_width::width(line) as usize;
        assert!(width <= 25, "Line exceeds max width: {}", line);
    }
}

// =============================================================================
// Block Quote Tests
// =============================================================================

#[test]
fn test_block_quote_line_breaking() {
    let mut ctx = LineBreakingContext::with_prefixes(20, 30, "", "> ");
    ctx.add_text("This is a blockquote with some text that should wrap");

    let formatted = ctx.format();
    let lines: Vec<&str> = formatted.lines().collect();

    if lines.len() > 1 {
        for line in &lines[1..] {
            assert!(
                line.starts_with("> "),
                "Block quote continuation line should start with '> ': {}",
                line
            );
        }
    }
}

#[test]
fn test_nested_block_quote_line_breaking() {
    let mut ctx = LineBreakingContext::with_prefixes(15, 20, "", "> > ");
    ctx.add_text("Nested blockquote with some text");

    let formatted = ctx.format();
    let lines: Vec<&str> = formatted.lines().collect();

    if lines.len() > 1 {
        for line in &lines[1..] {
            assert!(
                line.starts_with("> > "),
                "Nested block quote continuation line should start with '> > ': {}",
                line
            );
        }
    }
}

#[test]
fn test_block_quote_single_line() {
    let mut ctx = LineBreakingContext::with_prefixes(20, 30, "", "> ");
    ctx.add_text("Short quote");

    let formatted = ctx.format();
    let lines: Vec<&str> = formatted.lines().collect();

    assert_eq!(lines.len(), 1, "Short quote should be on one line");
    assert_eq!(lines[0], "Short quote");
}

#[test]
fn test_block_quote_width_constraint() {
    let mut ctx = LineBreakingContext::with_prefixes(15, 20, "", "> ");
    ctx.add_text("This is a longer text that should wrap properly");

    let formatted = ctx.format();

    for line in formatted.lines() {
        let width = unicode_width::width(line) as usize;
        assert!(
            width <= 20,
            "Block quote line exceeds max width: {} (width: {})",
            line,
            width
        );
    }
}

#[test]
fn test_triple_nested_block_quote() {
    let mut ctx = LineBreakingContext::with_prefixes(10, 15, "", "> > > ");
    ctx.add_text("Deeply nested quote");

    let formatted = ctx.format();
    let lines: Vec<&str> = formatted.lines().collect();

    if lines.len() > 1 {
        for line in &lines[1..] {
            assert!(
                line.starts_with("> > > "),
                "Triple nested continuation line should start with '> > > ': {}",
                line
            );
        }
    }
}

// =============================================================================
// List Prefix Tests
// =============================================================================

#[test]
fn test_line_breaking_with_prefixes() {
    let mut ctx = LineBreakingContext::with_prefixes(20, 30, "- ", "  ");
    ctx.add_text("This is a list item with some text");

    let formatted = ctx.format();
    let lines: Vec<&str> = formatted.lines().collect();

    assert!(
        lines[0].starts_with("- "),
        "First line should start with list marker"
    );

    if lines.len() > 1 {
        assert!(
            lines[1].starts_with("  "),
            "Continuation lines should be indented"
        );
    }
}

#[test]
fn test_line_breaking_with_ordered_list_prefix() {
    let mut ctx = LineBreakingContext::with_prefixes(20, 35, "1. ", "   ");
    ctx.add_text("First ordered item with some text content");

    let formatted = ctx.format();
    let lines: Vec<&str> = formatted.lines().collect();

    assert!(
        lines[0].starts_with("1. "),
        "First line should start with ordered list marker"
    );

    if lines.len() > 1 {
        assert!(
            lines[1].starts_with("   "),
            "Continuation lines should be indented to align with content"
        );
    }
}

#[test]
fn test_prefix_width_considered_in_breaks() {
    let mut ctx = LineBreakingContext::with_prefixes(20, 25, "- ", "  ");
    ctx.add_text("This is a test paragraph");

    let formatted = ctx.format();

    for line in formatted.lines() {
        let width = unicode_width::width(line) as usize;
        assert!(width <= 25, "Line with prefix exceeds max width: {}", line);
    }
}

#[test]
fn test_empty_prefixes() {
    let mut ctx = LineBreakingContext::with_prefixes(20, 25, "", "");
    ctx.add_text("Simple text without prefixes");

    let formatted = ctx.format();
    let lines: Vec<&str> = formatted.lines().collect();

    for line in lines {
        assert!(
            !line.starts_with("- ") && !line.starts_with("  "),
            "Line should not have prefix: {}",
            line
        );
    }
}

#[test]
fn test_nested_list_prefixes() {
    let mut ctx = LineBreakingContext::with_prefixes(15, 20, "    - ", "      ");
    ctx.add_text("Nested item with text");

    let formatted = ctx.format();
    let lines: Vec<&str> = formatted.lines().collect();

    assert!(lines[0].starts_with("    - "), "Should have nested marker");

    for line in formatted.lines() {
        let width = unicode_width::width(line) as usize;
        assert!(width <= 20, "Nested line exceeds max width: {}", line);
    }
}

// =============================================================================
// CJK Text Tests
// =============================================================================

#[test]
fn test_cjk_text_formatting() {
    let mut ctx = LineBreakingContext::new(80, 80);
    ctx.add_text("单词和数字123。");

    assert_eq!(
        ctx.words().len(),
        2,
        "Should split at CJK/ASCII boundary: {:?}",
        ctx.words()
    );
    assert_eq!(ctx.words()[0].text, "单词和数字");
    assert_eq!(ctx.words()[1].text, "123。");

    let formatted = ctx.format();
    assert!(
        formatted.contains("单词和数字123"),
        "CJK text should not have spaces between words: {}",
        formatted
    );
}

#[test]
fn test_cjk_text_formatting_with_spacing() {
    let mut ctx = LineBreakingContext::new(80, 80);
    ctx.add_text("单词和数字 123。");

    assert_eq!(ctx.words().len(), 2);
    assert_eq!(ctx.words()[0].text, "单词和数字");
    assert_eq!(ctx.words()[1].text, "123。");

    let formatted = ctx.format();
    assert!(
        formatted.contains("单词和数字 123"),
        "Should have space between CJK and number with CJK spacing: {}",
        formatted
    );
}

#[test]
fn test_cjk_punctuation_handling() {
    let mut ctx = LineBreakingContext::new(80, 80);
    ctx.add_markdown_marker("**");
    ctx.add_text("特性：");

    assert_eq!(ctx.words().len(), 2);
    assert_eq!(ctx.words()[0].text, "**");
    assert_eq!(ctx.words()[1].text, "特性：");
    assert!(!ctx.words()[1].needs_leading_space);
}

#[test]
fn test_cjk_text_after_inline_code() {
    let mut ctx = LineBreakingContext::new(80, 80);
    ctx.add_inline_element("`tva`");
    ctx.add_text("的开发者");

    assert_eq!(ctx.words().len(), 2);
    assert_eq!(ctx.words()[0].text, "`tva`");
    assert_eq!(ctx.words()[1].text, "的开发者");
    assert!(ctx.words()[1].needs_leading_space);
}

// =============================================================================
// Inline Code and Punctuation Tests
// =============================================================================

#[test]
fn test_ascii_punctuation_no_space_after_marker() {
    let mut ctx = LineBreakingContext::new(80, 80);

    ctx.add_markdown_marker("`");
    ctx.add_inline_element("replace_na");
    ctx.add_markdown_marker("`");
    ctx.add_text(": 将显式");

    let formatted = ctx.format();
    assert!(
        formatted.contains("`replace_na`:"),
        "Colon should not have leading space after inline code: {}",
        formatted
    );
}

#[test]
fn test_colon_after_inline_code_with_cjk() {
    let mut ctx = LineBreakingContext::new(80, 80);

    ctx.add_markdown_marker("`");
    ctx.add_inline_element("longer");
    ctx.add_markdown_marker("`");
    ctx.add_text(": 支持在 `--names-to` 中使用");

    let formatted = ctx.format();

    assert!(
        formatted.contains("`longer`: 支持"),
        "Colon should not have leading space but should have trailing space: {}",
        formatted
    );
}

#[test]
fn test_ascii_punctuation_various() {
    let test_cases = vec![
        (":", "colon"),
        (",", "comma"),
        (".", "period"),
        (";", "semicolon"),
        ("!", "exclamation"),
        ("?", "question"),
    ];

    for (punct, name) in test_cases {
        let mut ctx = LineBreakingContext::new(80, 80);
        ctx.add_markdown_marker("`");
        ctx.add_inline_element("code");
        ctx.add_markdown_marker("`");
        ctx.add_text(&format!("{} text", punct));

        let formatted = ctx.format();
        let expected = format!("`code`{} text", punct);
        assert!(
            formatted.contains(&expected),
            "{} should not have leading space after inline code: got '{}'",
            name,
            formatted
        );
    }
}

#[test]
fn test_left_paren_has_space_after_inline_code() {
    let mut ctx = LineBreakingContext::new(80, 80);

    ctx.add_inline_element("`strbin`");
    ctx.add_text("(字符串哈希分箱)");

    let formatted = ctx.format();

    assert!(
        formatted.contains("`strbin` (字符串哈希分箱)"),
        "Left parenthesis should have leading space after inline code: got {}",
        formatted
    );
}

#[test]
fn test_brackets_have_space_after_inline_code() {
    let test_cases = vec![
        ("(", ")", "parentheses"),
        ("[", "]", "brackets"),
        ("{", "}", "braces"),
    ];

    for (open, close, name) in test_cases {
        let mut ctx = LineBreakingContext::new(80, 80);
        ctx.add_inline_element("`code`");
        ctx.add_text(&format!("{}text{}", open, close));

        let formatted = ctx.format();
        let expected = format!("`code` {}text{}", open, close);
        assert!(
            formatted.contains(&expected),
            "{} should have leading space after inline code: got '{}'",
            name,
            formatted
        );
    }
}

// =============================================================================
// Link Tests
// =============================================================================

#[test]
fn test_long_link_not_split() {
    let mut ctx = LineBreakingContext::new(40, 50);

    ctx.add_text("我们旨在重现 ");
    ctx.add_inline_element("`https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md`");
    ctx.add_text(" 使用的严格基准测试策略。");

    let formatted = ctx.format();

    assert!(
        formatted.contains("`https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md`"),
        "Long URL should NOT be split at '/' boundaries - it would break the link. Formatted:\n{}",
        formatted
    );

    assert!(
        !formatted.contains("https:/ ") && !formatted.contains("/ "),
        "URL should not contain spaces that would break the link. Formatted:\n{}",
        formatted
    );
}

#[test]
fn test_long_url_link_not_split() {
    let mut ctx = LineBreakingContext::new(40, 50);

    ctx.add_markdown_marker("[");
    ctx.add_text_as_word("https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md");
    ctx.add_markdown_marker("]");
    ctx.add_markdown_marker("(");
    ctx.add_text_as_word("https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md");
    ctx.add_link_close_marker(")");

    let formatted = ctx.format();

    assert!(
        !formatted.contains("\n)"),
        "Closing parenthesis should NOT be on its own line. Formatted:\n{}",
        formatted
    );
}

#[test]
fn test_long_url_link_with_following_text() {
    let mut ctx = LineBreakingContext::new(40, 50);

    ctx.add_markdown_marker("[");
    ctx.add_text_as_word("https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md");
    ctx.add_markdown_marker("]");
    ctx.add_markdown_marker("(");
    ctx.add_text_as_word("https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md");
    ctx.add_link_close_marker(")");
    ctx.add_text("使用的严格基准测试策略。");

    let formatted = ctx.format();

    assert!(
        !formatted.contains("\n)"),
        "Closing parenthesis should NOT be on its own line. Formatted:\n{}",
        formatted
    );

    assert!(
        !formatted.contains("]\n("),
        "`](` should NOT be split across lines. Formatted:\n{}",
        formatted
    );
}

#[test]
fn test_link_with_text_and_long_url() {
    let mut ctx = LineBreakingContext::new(60, 60);

    ctx.add_text("我们旨在重现 ");
    ctx.add_markdown_marker("[");
    ctx.add_text("eBay TSV Utilities");
    ctx.add_markdown_marker("]");
    ctx.add_markdown_marker("(");
    ctx.add_text_as_word("https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md");
    ctx.add_link_close_marker(")");
    ctx.add_text(" 使用的严格基准测试策略。");

    let formatted = ctx.format();

    assert!(
        !formatted.contains("]\n("),
        "`](` should NOT be split across lines. Formatted:\n{}",
        formatted
    );

    assert!(
        formatted.contains("](https://"),
        "URL should be on the same line as `](`. Formatted:\n{}",
        formatted
    );
}

#[test]
fn test_link_with_cjk_punctuation_not_at_line_start() {
    let mut ctx = LineBreakingContext::new(60, 60);

    ctx.add_text("- **HEPMASS** ( 4.8GB): ");
    ctx.add_markdown_marker("[");
    ctx.add_text("link");
    ctx.add_markdown_marker("]");
    ctx.add_markdown_marker("(");
    ctx.add_text_as_word("https://archive.ics.uci.edu/ml/datasets/HEPMASS");
    ctx.add_link_close_marker(")");
    ctx.add_text(" 。测试。");

    let formatted = ctx.format();

    assert!(
        !formatted.contains("\n  。"),
        "CJK period should NOT be at line start. Formatted:\n{}",
        formatted
    );

    assert!(
        formatted.contains(")。"),
        "CJK period should be on the same line as the link. Formatted:\n{}",
        formatted
    );
}

#[test]
fn test_link_with_various_cjk_punctuation() {
    let test_cases = vec![
        ("，", "CJK comma"),
        ("、", "CJK enumeration comma"),
        ("；", "CJK semicolon"),
        ("：", "CJK colon"),
        ("！", "CJK exclamation"),
        ("？", "CJK question"),
        ("）", "CJK right parenthesis"),
        ("】", "CJK right bracket"),
        ("」", "CJK right corner bracket"),
        ("』", "CJK right white corner bracket"),
        ("〉", "CJK right angle bracket"),
        ("》", "CJK right double angle bracket"),
        ("〜", "Japanese wave dash"),
        ("〝", "Japanese double quote open"),
        ("〞", "Japanese double quote close"),
    ];

    for (punct, desc) in test_cases {
        let mut ctx = LineBreakingContext::new(60, 60);

        ctx.add_text("- ");
        ctx.add_markdown_marker("[");
        ctx.add_text("link");
        ctx.add_markdown_marker("]");
        ctx.add_markdown_marker("(");
        ctx.add_text_as_word("https://archive.ics.uci.edu/ml/datasets/HEPMASS");
        ctx.add_link_close_marker(")");
        ctx.add_text(&format!(" {} 测试", punct));

        let formatted = ctx.format();

        let newline_punct = format!("\n  {}", punct);
        assert!(
            !formatted.contains(&newline_punct),
            "{} ({}) should NOT be at line start. Formatted:\n{}",
            desc,
            punct,
            formatted
        );

        let link_punct = format!("){}", punct);
        assert!(
            formatted.contains(&link_punct),
            "{} ({}) should be on the same line as the link. Formatted:\n{}",
            desc,
            punct,
            formatted
        );
    }
}

// =============================================================================
// Markdown Emphasis Tests
// =============================================================================

#[test]
fn test_markdown_marker_not_split_across_lines() {
    let mut ctx = LineBreakingContext::with_prefixes(35, 45, "> ", "> ");

    ctx.add_markdown_marker("**");
    ctx.add_text("保持简单");
    ctx.add_markdown_marker("**");
    ctx.add_text("：tva 的表达式语言设计目标是");
    ctx.add_markdown_marker("**");
    ctx.add_text("简单高效的数据处理");
    ctx.add_markdown_marker("**");
    ctx.add_text("，不是通用编程语言。");

    let formatted = ctx.format();

    assert!(
        formatted.contains("**简单高效的数据处理**"),
        "Emphasized text should stay together. Formatted:\n{}",
        formatted
    );
}

#[test]
fn test_markdown_strong_emphasis_not_split() {
    let mut ctx = LineBreakingContext::with_prefixes(50, 60, "> ", "> ");

    ctx.add_markdown_marker("**");
    ctx.add_text("保持简单");
    ctx.add_markdown_marker("**");
    ctx.add_text("：tva 的表达式语言设计目标是");
    ctx.add_markdown_marker("**");
    ctx.add_text("简单高效的数据处理");
    ctx.add_markdown_marker("**");
    ctx.add_text("，不是通用编程语言。");

    let formatted = ctx.format();

    assert!(
        formatted.contains("**简单高效的数据处理**"),
        "The emphasized phrase should be intact. Formatted:\n{}",
        formatted
    );
}

#[test]
fn test_emphasis_in_middle_of_text_not_split() {
    let mut ctx = LineBreakingContext::new(35, 45);

    ctx.add_text("tva ");
    ctx.add_markdown_marker("**");
    ctx.add_text("只有匿名函数（lambda）");
    ctx.add_markdown_marker("**");
    ctx.add_text("且主要用于 TSV 数据处理");

    let formatted = ctx.format();

    assert!(
        formatted.contains("**只有匿名函数（lambda）**"),
        "The emphasized text should stay together. Formatted:\n{}",
        formatted
    );
}

// =============================================================================
// Full API Tests (using format_commonmark)
// =============================================================================

#[test]
fn test_opening_bracket_no_space_after_full() {
    use clmd::{format_commonmark, parse_document, Options, Plugins};

    let mut options = Options::default();
    options.render.width = 80;
    let input = "**HEPMASS** (\n  4.8GB)";
    let (arena, root) = parse_document(input, &options);
    let mut output = String::new();
    format_commonmark(&arena, root, &options, &mut output, &Plugins::default()).unwrap();

    assert!(
        !output.contains("( 4.8GB)"),
        "There should be no space after `(`. Output:\n{}",
        output
    );

    assert!(
        output.contains("(4.8GB)"),
        "`(` should be directly followed by `4.8GB`. Output:\n{}",
        output
    );
}
