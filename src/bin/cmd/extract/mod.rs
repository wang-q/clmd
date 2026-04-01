use clap::{Arg, ArgMatches, Command};

use crate::cmd::utils;
use clmd::core::nodes::NodeValue;
use serde_json::json;

pub fn make_subcommand() -> Command {
    Command::new("extract")
        .about("Extract specific elements from Markdown")
        .subcommand_required(true)
        .subcommand(
            Command::new("links")
                .about("Extract all links")
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
                ),
        )
        .subcommand(
            Command::new("headings")
                .about("Extract all headings")
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
                    Arg::new("level")
                        .short('l')
                        .long("level")
                        .help("Filter by heading level (1-6)"),
                )
                .arg(
                    Arg::new("format")
                        .long("format")
                        .default_value("text")
                        .help("Output format: text, json"),
                ),
        )
        .subcommand(
            Command::new("code")
                .about("Extract all code blocks")
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
                ),
        )
}

pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some(("links", sub_matches)) => extract_links(sub_matches, options),
        Some(("headings", sub_matches)) => extract_headings(sub_matches, options),
        Some(("code", sub_matches)) => extract_code(sub_matches, options),
        _ => unreachable!(),
    }
}

fn extract_links(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("text");

    let (arena, root) = clmd::parse_document(&input, options);

    let mut links = Vec::new();

    for node_id in arena.descendants(root) {
        let node = arena.get(node_id);
        if let NodeValue::Link(link) = &node.value {
            let text = collect_text(&arena, node_id);
            links.push((text, link.url.to_string()));
        }
    }

    let output = match format {
        "json" => {
            let json_links: Vec<_> = links
                .iter()
                .map(|(text, url)| json!({"text": text, "url": url}))
                .collect();
            serde_json::to_string_pretty(&json_links)?
        }
        _ => links
            .iter()
            .map(|(text, url)| format!("{}\t{}", text, url))
            .collect::<Vec<_>>()
            .join("\n"),
    };

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)
}

fn extract_headings(
    matches: &ArgMatches,
    options: &clmd::Options,
) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("text");
    let level_filter: Option<u8> = matches
        .get_one::<String>("level")
        .and_then(|s| s.parse().ok());

    let (arena, root) = clmd::parse_document(&input, options);

    let mut headings = Vec::new();

    for node_id in arena.descendants(root) {
        let node = arena.get(node_id);
        if let NodeValue::Heading(heading) = &node.value {
            let level = heading.level;
            if let Some(filter) = level_filter {
                if level != filter {
                    continue;
                }
            }
            let text = collect_text(&arena, node_id);
            headings.push((level, text));
        }
    }

    let output = match format {
        "json" => {
            let json_headings: Vec<_> = headings
                .iter()
                .map(|(level, text)| json!({"level": level, "text": text}))
                .collect();
            serde_json::to_string_pretty(&json_headings)?
        }
        _ => headings
            .iter()
            .map(|(level, text)| format!("{}\t{}", level, text))
            .collect::<Vec<_>>()
            .join("\n"),
    };

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)
}

fn extract_code(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;

    let (arena, root) = clmd::parse_document(&input, options);

    let mut code_blocks = Vec::new();

    for node_id in arena.descendants(root) {
        let node = arena.get(node_id);
        if let NodeValue::CodeBlock(code_block) = &node.value {
            code_blocks
                .push((code_block.info.to_string(), code_block.literal.to_string()));
        }
    }

    let output = code_blocks
        .iter()
        .map(|(lang, code)| {
            if lang.is_empty() {
                format!("```\n{}\n```", code)
            } else {
                format!("```{lang}\n{}\n```", code)
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)
}

fn collect_text(arena: &clmd::Arena, node_id: clmd::NodeId) -> String {
    let mut text = String::new();

    for child_id in arena.descendants(node_id) {
        let child = arena.get(child_id);
        if let NodeValue::Text(t) = &child.value {
            text.push_str(t);
        }
    }

    text
}
