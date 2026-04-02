use clap::{Arg, ArgAction, ArgMatches, Command};

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
            Command::new("images")
                .about("Extract all images")
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
                )
                .arg(
                    Arg::new("format")
                        .long("format")
                        .default_value("markdown")
                        .help("Output format: markdown, json, raw"),
                )
                .arg(
                    Arg::new("language")
                        .long("language")
                        .help("Filter by language (e.g., 'rust', 'python')"),
                ),
        )
        .subcommand(
            Command::new("tables")
                .about("Extract all tables")
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
                        .default_value("markdown")
                        .help("Output format: markdown, csv, json"),
                ),
        )
        .subcommand(
            Command::new("footnotes")
                .about("Extract all footnotes")
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
            Command::new("yaml-front-matter")
                .about("Extract YAML front matter")
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
                    Arg::new("key")
                        .short('k')
                        .long("key")
                        .help("Extract specific key from front matter"),
                ),
        )
        .subcommand(
            Command::new("task-items")
                .about("Extract all task list items")
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
                    Arg::new("checked")
                        .long("checked")
                        .action(ArgAction::SetTrue)
                        .help("Only show checked items"),
                )
                .arg(
                    Arg::new("unchecked")
                        .long("unchecked")
                        .action(ArgAction::SetTrue)
                        .help("Only show unchecked items"),
                ),
        )
}

pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some(("links", sub_matches)) => extract_links(sub_matches, options),
        Some(("images", sub_matches)) => extract_images(sub_matches, options),
        Some(("headings", sub_matches)) => extract_headings(sub_matches, options),
        Some(("code", sub_matches)) => extract_code(sub_matches, options),
        Some(("tables", sub_matches)) => extract_tables(sub_matches, options),
        Some(("footnotes", sub_matches)) => extract_footnotes(sub_matches, options),
        Some(("yaml-front-matter", sub_matches)) => {
            extract_yaml_front_matter(sub_matches)
        }
        Some(("task-items", sub_matches)) => extract_task_items(sub_matches, options),
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

fn extract_images(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("text");

    let (arena, root) = clmd::parse_document(&input, options);

    let mut images = Vec::new();

    for node_id in arena.descendants(root) {
        let node = arena.get(node_id);
        if let NodeValue::Image(image) = &node.value {
            let alt_text = collect_text(&arena, node_id);
            images.push((image.url.to_string(), alt_text));
        }
    }

    let output = match format {
        "json" => {
            let json_images: Vec<_> = images
                .iter()
                .map(|(url, alt)| json!({"url": url, "alt": alt}))
                .collect();
            serde_json::to_string_pretty(&json_images)?
        }
        _ => images
            .iter()
            .map(|(url, alt)| format!("{}\t{}", url, alt))
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
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("markdown");
    let language_filter = matches.get_one::<String>("language").map(|s| s.as_str());

    let (arena, root) = clmd::parse_document(&input, options);

    let mut code_blocks = Vec::new();

    for node_id in arena.descendants(root) {
        let node = arena.get(node_id);
        if let NodeValue::CodeBlock(code_block) = &node.value {
            let lang = code_block.info.to_string();

            // Apply language filter if specified
            if let Some(filter) = language_filter {
                let first_word = lang.split_whitespace().next().unwrap_or("");
                if !first_word.eq_ignore_ascii_case(filter) {
                    continue;
                }
            }

            code_blocks.push((lang, code_block.literal.to_string()));
        }
    }

    let output = match format {
        "json" => {
            let json_blocks: Vec<_> = code_blocks
                .iter()
                .map(|(lang, code)| json!({"language": lang, "code": code}))
                .collect();
            serde_json::to_string_pretty(&json_blocks)?
        }
        "raw" => code_blocks
            .iter()
            .map(|(_, code)| code.as_str())
            .collect::<Vec<_>>()
            .join("\n\n"),
        _ => code_blocks
            .iter()
            .map(|(lang, code)| {
                if lang.is_empty() {
                    format!("```\n{}\n```", code)
                } else {
                    format!("```{lang}\n{}\n```", code)
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
    };

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)
}

fn extract_tables(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("markdown");

    let (arena, root) = clmd::parse_document(&input, options);

    let mut tables = Vec::new();

    for node_id in arena.descendants(root) {
        let node = arena.get(node_id);
        if let NodeValue::Table(_table) = &node.value {
            let mut rows = Vec::new();

            // Extract header row
            if let Some(header_id) = arena.get(node_id).first_child {
                let header_row = extract_table_row(&arena, header_id);
                rows.push(header_row);

                // Extract body rows
                let mut current = arena.get(header_id).next;
                while let Some(current_id) = current {
                    let row = extract_table_row(&arena, current_id);
                    rows.push(row);
                    current = arena.get(current_id).next;
                }
            }

            tables.push(rows);
        }
    }

    let output = match format {
        "json" => serde_json::to_string_pretty(&tables)?,
        "csv" => {
            let mut csv_output = String::new();
            for (i, table) in tables.iter().enumerate() {
                if i > 0 {
                    csv_output.push_str("\n\n");
                }
                for row in table {
                    let csv_row: Vec<String> = row
                        .iter()
                        .map(|cell| {
                            // Escape quotes and wrap in quotes if needed
                            if cell.contains('"')
                                || cell.contains(',')
                                || cell.contains('\n')
                            {
                                format!("\"{}\"", cell.replace('"', "\"\""))
                            } else {
                                cell.clone()
                            }
                        })
                        .collect();
                    csv_output.push_str(&csv_row.join(","));
                    csv_output.push('\n');
                }
            }
            csv_output
        }
        _ => {
            // Markdown format
            let mut md_output = String::new();
            for (i, table) in tables.iter().enumerate() {
                if i > 0 {
                    md_output.push_str("\n\n");
                }

                for (row_idx, row) in table.iter().enumerate() {
                    md_output.push_str("| ");
                    md_output.push_str(&row.join(" | "));
                    md_output.push_str(" |\n");

                    // Add separator after header
                    if row_idx == 0 {
                        let separators: Vec<_> =
                            row.iter().map(|_| "---".to_string()).collect();
                        md_output.push_str("| ");
                        md_output.push_str(&separators.join(" | "));
                        md_output.push_str(" |\n");
                    }
                }
            }
            md_output
        }
    };

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)
}

fn extract_table_row(arena: &clmd::Arena, row_id: clmd::NodeId) -> Vec<String> {
    let mut cells = Vec::new();

    if let Some(first_cell) = arena.get(row_id).first_child {
        let mut current = Some(first_cell);
        while let Some(current_id) = current {
            let cell_text = collect_text(arena, current_id);
            cells.push(cell_text);

            current = arena.get(current_id).next;
        }
    }

    cells
}

fn extract_footnotes(
    matches: &ArgMatches,
    options: &clmd::Options,
) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("text");

    let (arena, root) = clmd::parse_document(&input, options);

    let mut footnotes = Vec::new();

    for node_id in arena.descendants(root) {
        let node = arena.get(node_id);
        if let NodeValue::FootnoteDefinition(footnote) = &node.value {
            let name = footnote.name.to_string();
            let content = collect_text_excluding_children(&arena, node_id);
            footnotes.push((name, content));
        }
    }

    let output = match format {
        "json" => {
            let json_footnotes: Vec<_> = footnotes
                .iter()
                .map(|(name, content)| json!({"name": name, "content": content}))
                .collect();
            serde_json::to_string_pretty(&json_footnotes)?
        }
        _ => footnotes
            .iter()
            .map(|(name, content)| format!("[{}]: {}", name, content))
            .collect::<Vec<_>>()
            .join("\n"),
    };

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)
}

fn extract_yaml_front_matter(matches: &ArgMatches) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;
    let key_filter = matches.get_one::<String>("key").map(|s| s.as_str());

    // Parse YAML front matter from the input
    let front_matter = if let Some(stripped) = input.strip_prefix("---") {
        if let Some(end) = stripped.find("---") {
            let yaml_content = &stripped[..end];
            yaml_content.trim()
        } else {
            ""
        }
    } else {
        ""
    };

    if front_matter.is_empty() {
        return Ok(());
    }

    // If a specific key is requested, try to extract it
    let output = if let Some(key) = key_filter {
        // Simple line-based extraction for basic key: value pairs
        let mut result = String::new();
        for line in front_matter.lines() {
            if let Some((k, v)) = line.split_once(':') {
                if k.trim() == key {
                    result = v.trim().to_string();
                    break;
                }
            }
        }
        result
    } else {
        front_matter.to_string()
    };

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)
}

fn extract_task_items(
    matches: &ArgMatches,
    options: &clmd::Options,
) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("text");
    let only_checked = matches.get_flag("checked");
    let only_unchecked = matches.get_flag("unchecked");

    let (arena, root) = clmd::parse_document(&input, options);

    let mut tasks = Vec::new();

    for node_id in arena.descendants(root) {
        let node = arena.get(node_id);
        if let NodeValue::TaskItem(task_item) = &node.value {
            let is_checked = task_item.symbol.is_some();

            // Apply filters
            if only_checked && !is_checked {
                continue;
            }
            if only_unchecked && is_checked {
                continue;
            }

            let text = collect_text(&arena, node_id);
            tasks.push((is_checked, text));
        }
    }

    let output = match format {
        "json" => {
            let json_tasks: Vec<_> = tasks
                .iter()
                .map(|(checked, text)| json!({"checked": checked, "text": text}))
                .collect();
            serde_json::to_string_pretty(&json_tasks)?
        }
        _ => tasks
            .iter()
            .map(|(checked, text)| {
                let status = if *checked { "[x]" } else { "[ ]" };
                format!("{} {}", status, text)
            })
            .collect::<Vec<_>>()
            .join("\n"),
    };

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

fn collect_text_excluding_children(
    arena: &clmd::Arena,
    node_id: clmd::NodeId,
) -> String {
    let mut text = String::new();
    let node = arena.get(node_id);

    // Only collect text from direct children, not all descendants
    if let Some(first_child) = node.first_child {
        let mut current = Some(first_child);
        while let Some(current_id) = current {
            let child = arena.get(current_id);
            if let NodeValue::Text(t) = &child.value {
                text.push_str(t);
            }

            current = child.next;
        }
    }

    text
}
