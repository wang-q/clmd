use clap::{Arg, ArgMatches, Command};
use serde_json::json;

use crate::cmd::utils;
use clmd::nodes::NodeValue;

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
}

pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("text");

    let (arena, root) = clmd::parse_document(&input, options);

    // Count lines and basic text stats
    let mut stats = Stats {
        lines: input.lines().count(),
        words: utils::count_words(&input),
        characters: utils::count_chars(&input),
        bytes: input.len(),
        ..Stats::default()
    };

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
            NodeValue::CodeBlock(_) => stats.code_blocks += 1,
            NodeValue::Code(_) => stats.inline_code += 1,
            NodeValue::List(_) => stats.lists += 1,
            NodeValue::Item(_) => stats.list_items += 1,
            NodeValue::BlockQuote => stats.blockquotes += 1,
            NodeValue::ThematicBreak => stats.thematic_breaks += 1,
            NodeValue::Table(_) => stats.tables += 1,
            NodeValue::TaskItem(_) => stats.task_items += 1,
            _ => {}
        }
    }

    let output = match format {
        "json" => serde_json::to_string_pretty(&json!({
            "lines": stats.lines,
            "words": stats.words,
            "characters": stats.characters,
            "bytes": stats.bytes,
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
            "code_blocks": stats.code_blocks,
            "inline_code": stats.inline_code,
            "lists": stats.lists,
            "list_items": stats.list_items,
            "blockquotes": stats.blockquotes,
            "thematic_breaks": stats.thematic_breaks,
            "tables": stats.tables,
            "task_items": stats.task_items,
        }))?,
        _ => format!(
            "Lines: {}\n\
             Words: {}\n\
             Characters: {}\n\
             Bytes: {}\n\
             Headings: {} (h1: {}, h2: {}, h3: {}, h4: {}, h5: {}, h6: {})\n\
             Links: {}\n\
             Images: {}\n\
             Code blocks: {}\n\
             Inline code: {}\n\
             Lists: {}\n\
             List items: {}\n\
             Blockquotes: {}\n\
             Thematic breaks: {}\n\
             Tables: {}\n\
             Task items: {}",
            stats.lines,
            stats.words,
            stats.characters,
            stats.bytes,
            stats.headings,
            stats.headings_h1,
            stats.headings_h2,
            stats.headings_h3,
            stats.headings_h4,
            stats.headings_h5,
            stats.headings_h6,
            stats.links,
            stats.images,
            stats.code_blocks,
            stats.inline_code,
            stats.lists,
            stats.list_items,
            stats.blockquotes,
            stats.thematic_breaks,
            stats.tables,
            stats.task_items,
        ),
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
}
