use clap::{Arg, ArgAction, ArgMatches, Command};
use serde_json::json;
use std::collections::HashMap;

use crate::cmd::utils;
use clmd::core::nodes::NodeValue;

pub fn make_subcommand() -> Command {
    Command::new("stats")
        .about("Show statistics about Markdown document")
        .arg(
            Arg::new("input")
                .help("Input Markdown file (default: stdin)")
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file (default: stdout)"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .default_value("text")
                .help("Output format: text, json"),
        )
        .arg(
            Arg::new("readability")
                .long("readability")
                .action(ArgAction::SetTrue)
                .help("Include readability metrics"),
        )
        .arg(
            Arg::new("code-stats")
                .long("code-stats")
                .action(ArgAction::SetTrue)
                .help("Include code block language statistics"),
        )
        .after_help(
            r###"Statistics include:
  - Basic counts: lines, words, characters, bytes
  - Structure: headings, links, images, lists, tables
  - Code: code blocks, inline code
  - Readability (with --readability): reading time, Flesch-Kincaid score
  - Code languages (with --code-stats): breakdown by programming language

Examples:
  clmd stats input.md
  clmd stats input.md --readability
  clmd stats input.md --code-stats
  clmd stats input.md --format json
"###,
        )
}

pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("text");
    let include_readability = matches.get_flag("readability");
    let include_code_stats = matches.get_flag("code-stats");

    let (arena, root) = clmd::parse_document(&input, options);

    // Count lines and basic text stats
    let mut stats = Stats {
        lines: input.lines().count(),
        words: utils::count_words(&input),
        characters: utils::count_chars(&input),
        bytes: input.len(),
        ..Stats::default()
    };

    // Track code languages
    let mut code_languages: HashMap<String, usize> = HashMap::new();

    // Traverse AST for element counts
    for node_id in arena.descendants(root) {
        let node = arena.get(node_id);
        match &node.value {
            NodeValue::Heading(h) => {
                stats.headings += 1;
                match h.level {
                    1 => stats.headings_h1 += 1,
                    2 => stats.headings_h2 += 1,
                    3 => stats.headings_h3 += 1,
                    4 => stats.headings_h4 += 1,
                    5 => stats.headings_h5 += 1,
                    6 => stats.headings_h6 += 1,
                    _ => {}
                }
            }
            NodeValue::Link(_) => stats.links += 1,
            NodeValue::Image(_) => stats.images += 1,
            NodeValue::CodeBlock(code_block) => {
                stats.code_blocks += 1;

                // Track code language
                if include_code_stats {
                    let lang = code_block.info.split_whitespace().next().unwrap_or("");
                    let lang_key = if lang.is_empty() {
                        "text".to_string()
                    } else {
                        lang.to_lowercase()
                    };
                    *code_languages.entry(lang_key).or_insert(0) += 1;
                }
            }
            NodeValue::Code(_) => stats.inline_code += 1,
            NodeValue::List(_) => stats.lists += 1,
            NodeValue::Item(_) => stats.list_items += 1,
            NodeValue::BlockQuote => stats.blockquotes += 1,
            NodeValue::ThematicBreak => stats.thematic_breaks += 1,
            NodeValue::Table(_) => stats.tables += 1,
            NodeValue::TaskItem(task_item) => {
                stats.task_items += 1;
                let is_checked = task_item.symbol.is_some();
                if is_checked {
                    stats.task_items_checked += 1;
                } else {
                    stats.task_items_unchecked += 1;
                }
            }
            _ => {}
        }
    }

    // Calculate readability metrics
    let readability = if include_readability {
        Some(calculate_readability(&input, stats.words, stats.sentences))
    } else {
        None
    };

    // Estimate reading time (average 200 words per minute)
    let reading_time_minutes = stats.words as f64 / 200.0;
    let reading_time = if reading_time_minutes < 1.0 {
        format!("{:.0} seconds", reading_time_minutes * 60.0)
    } else {
        format!("{:.1} minutes", reading_time_minutes)
    };

    let output = match format {
        "json" => {
            let mut json_obj = json!({
                "basic": {
                    "lines": stats.lines,
                    "words": stats.words,
                    "characters": stats.characters,
                    "bytes": stats.bytes,
                },
                "structure": {
                    "headings": {
                        "total": stats.headings,
                        "h1": stats.headings_h1,
                        "h2": stats.headings_h2,
                        "h3": stats.headings_h3,
                        "h4": stats.headings_h4,
                        "h5": stats.headings_h5,
                        "h6": stats.headings_h6,
                    },
                    "links": stats.links,
                    "images": stats.images,
                    "lists": stats.lists,
                    "list_items": stats.list_items,
                    "blockquotes": stats.blockquotes,
                    "thematic_breaks": stats.thematic_breaks,
                    "tables": stats.tables,
                },
                "code": {
                    "code_blocks": stats.code_blocks,
                    "inline_code": stats.inline_code,
                },
                "tasks": {
                    "total": stats.task_items,
                    "checked": stats.task_items_checked,
                    "unchecked": stats.task_items_unchecked,
                },
                "reading_time": reading_time,
            });

            if let Some(r) = readability {
                json_obj["readability"] = json!({
                    "flesch_kincaid_grade": r.flesch_kincaid_grade,
                    "flesch_reading_ease": r.flesch_reading_ease,
                    "sentences": r.sentences,
                    "syllables": r.syllables,
                    "complexity": r.complexity,
                });
            }

            if include_code_stats {
                let mut lang_vec: Vec<_> = code_languages.iter().collect();
                lang_vec.sort_by(|a, b| b.1.cmp(a.1));
                json_obj["code_languages"] = json!(lang_vec
                    .iter()
                    .map(|(k, v)| json!({"language": k, "count": v}))
                    .collect::<Vec<_>>());
            }

            serde_json::to_string_pretty(&json_obj)?
        }
        _ => {
            let mut output = String::new();

            output.push_str("=== Basic Statistics ===\n");
            output.push_str(&format!("Lines:       {}\n", stats.lines));
            output.push_str(&format!("Words:       {}\n", stats.words));
            output.push_str(&format!("Characters:  {}\n", stats.characters));
            output.push_str(&format!("Bytes:       {}\n", stats.bytes));
            output.push_str(&format!("Reading time: {}\n", reading_time));
            output.push('\n');

            output.push_str("=== Document Structure ===\n");
            output.push_str(&format!(
                "Headings:    {} (h1: {}, h2: {}, h3: {}, h4: {}, h5: {}, h6: {})\n",
                stats.headings,
                stats.headings_h1,
                stats.headings_h2,
                stats.headings_h3,
                stats.headings_h4,
                stats.headings_h5,
                stats.headings_h6
            ));
            output.push_str(&format!("Links:       {}\n", stats.links));
            output.push_str(&format!("Images:      {}\n", stats.images));
            output.push_str(&format!("Lists:       {}\n", stats.lists));
            output.push_str(&format!("List items:  {}\n", stats.list_items));
            output.push_str(&format!("Blockquotes: {}\n", stats.blockquotes));
            output.push_str(&format!("Tables:      {}\n", stats.tables));
            output.push('\n');

            output.push_str("=== Code Statistics ===\n");
            output.push_str(&format!("Code blocks:  {}\n", stats.code_blocks));
            output.push_str(&format!("Inline code:  {}\n", stats.inline_code));

            if include_code_stats && !code_languages.is_empty() {
                output.push_str("\nCode languages:\n");
                let mut lang_vec: Vec<_> = code_languages.iter().collect();
                lang_vec.sort_by(|a, b| b.1.cmp(a.1));
                for (lang, count) in lang_vec {
                    output.push_str(&format!("  {}: {}\n", lang, count));
                }
            }
            output.push('\n');

            if stats.task_items > 0 {
                output.push_str("=== Task List Statistics ===\n");
                output.push_str(&format!("Total:      {}\n", stats.task_items));
                output.push_str(&format!(
                    "Checked:    {} ({:.1}%)\n",
                    stats.task_items_checked,
                    (stats.task_items_checked as f64 / stats.task_items as f64) * 100.0
                ));
                output.push_str(&format!(
                    "Unchecked:  {} ({:.1}%)\n",
                    stats.task_items_unchecked,
                    (stats.task_items_unchecked as f64 / stats.task_items as f64)
                        * 100.0
                ));
                output.push('\n');
            }

            if let Some(r) = readability {
                output.push_str("=== Readability Metrics ===\n");
                output.push_str(&format!(
                    "Flesch-Kincaid Grade: {:.1}\n",
                    r.flesch_kincaid_grade
                ));
                output.push_str(&format!(
                    "Flesch Reading Ease:  {:.1}\n",
                    r.flesch_reading_ease
                ));
                output.push_str(&format!("Sentences:            {}\n", r.sentences));
                output.push_str(&format!("Syllables:            {}\n", r.syllables));
                output.push_str(&format!("Complexity:           {}\n", r.complexity));
            }

            output
        }
    };

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)
}

#[derive(Default)]
struct Stats {
    lines: usize,
    words: usize,
    characters: usize,
    bytes: usize,
    headings: usize,
    headings_h1: usize,
    headings_h2: usize,
    headings_h3: usize,
    headings_h4: usize,
    headings_h5: usize,
    headings_h6: usize,
    links: usize,
    images: usize,
    code_blocks: usize,
    inline_code: usize,
    lists: usize,
    list_items: usize,
    blockquotes: usize,
    thematic_breaks: usize,
    tables: usize,
    task_items: usize,
    task_items_checked: usize,
    task_items_unchecked: usize,
    sentences: usize,
}

struct ReadabilityMetrics {
    flesch_kincaid_grade: f64,
    flesch_reading_ease: f64,
    sentences: usize,
    syllables: usize,
    complexity: String,
}

fn calculate_readability(
    text: &str,
    words: usize,
    sentences: usize,
) -> ReadabilityMetrics {
    let sentences = if sentences == 0 {
        // Estimate sentences by counting sentence-ending punctuation
        text.matches('.').count() + text.matches('!').count() + text.matches('?').count()
    } else {
        sentences
    };

    let sentences = sentences.max(1);
    let words = words.max(1);

    // Estimate syllables (rough approximation)
    let syllables = estimate_syllables(text);

    // Flesch-Kincaid Grade Level
    // Formula: 0.39 * (words/sentences) + 11.8 * (syllables/words) - 15.59
    let flesch_kincaid_grade = 0.39 * (words as f64 / sentences as f64)
        + 11.8 * (syllables as f64 / words as f64)
        - 15.59;

    // Flesch Reading Ease
    // Formula: 206.835 - 1.015 * (words/sentences) - 84.6 * (syllables/words)
    let flesch_reading_ease = 206.835
        - 1.015 * (words as f64 / sentences as f64)
        - 84.6 * (syllables as f64 / words as f64);

    // Determine complexity level
    let complexity = if flesch_reading_ease >= 90.0 {
        "Very Easy".to_string()
    } else if flesch_reading_ease >= 80.0 {
        "Easy".to_string()
    } else if flesch_reading_ease >= 70.0 {
        "Fairly Easy".to_string()
    } else if flesch_reading_ease >= 60.0 {
        "Standard".to_string()
    } else if flesch_reading_ease >= 50.0 {
        "Fairly Difficult".to_string()
    } else if flesch_reading_ease >= 30.0 {
        "Difficult".to_string()
    } else {
        "Very Difficult".to_string()
    };

    ReadabilityMetrics {
        flesch_kincaid_grade: flesch_kincaid_grade.max(0.0),
        flesch_reading_ease,
        sentences,
        syllables,
        complexity,
    }
}

fn estimate_syllables(text: &str) -> usize {
    let mut count = 0;

    for word in text.split_whitespace() {
        let word = word.to_lowercase();
        let word: String = word.chars().filter(|c| c.is_alphabetic()).collect();

        if word.is_empty() {
            continue;
        }

        // Count vowel groups
        let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
        let mut prev_was_vowel = false;
        let mut syllables = 0;

        for ch in word.chars() {
            let is_vowel = vowels.contains(&ch);
            if is_vowel && !prev_was_vowel {
                syllables += 1;
            }
            prev_was_vowel = is_vowel;
        }

        // Handle silent e
        if word.ends_with('e') && syllables > 1 {
            syllables -= 1;
        }

        // Every word has at least one syllable
        count += syllables.max(1);
    }

    count
}
