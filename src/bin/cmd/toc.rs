use clap::{Arg, ArgMatches, Command};

use crate::cmd::utils;
use clmd::core::nodes::NodeValue;

pub fn make_subcommand() -> Command {
    Command::new("toc")
        .about("Generate table of contents from Markdown headings")
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
            Arg::new("levels")
                .short('l')
                .long("levels")
                .default_value("1-6")
                .help("Heading level range (e.g., 1-3, 2-4)"),
        )
        .arg(
            Arg::new("numbered")
                .long("numbered")
                .action(clap::ArgAction::SetTrue)
                .help("Add numbering to TOC entries"),
        )
        .arg(
            Arg::new("links")
                .long("links")
                .action(clap::ArgAction::SetTrue)
                .help("Generate anchor links"),
        )
}

pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;
    let levels = matches
        .get_one::<String>("levels")
        .map(|s| s.as_str())
        .unwrap_or("1-6");
    let numbered = matches.get_flag("numbered");
    let links = matches.get_flag("links");

    let (min_level, max_level) = parse_levels(levels)?;

    let (arena, root) = clmd::parse_document(&input, options);

    let mut headings = Vec::new();

    for node_id in arena.descendants(root) {
        let node = arena.get(node_id);
        if let NodeValue::Heading(heading) = &node.value {
            let level = heading.level;
            if level >= min_level && level <= max_level {
                let text = collect_text(&arena, node_id);
                headings.push((level, text));
            }
        }
    }

    if headings.is_empty() {
        return Ok(());
    }

    // Adjust levels to make the first heading level 1 in TOC
    let min_heading_level = headings.iter().map(|(l, _)| *l).min().unwrap_or(1);

    let mut output = String::new();
    let mut counters = vec![0; 7]; // Track numbering for each level

    for (level, text) in headings {
        let adjusted_level = level - min_heading_level + 1;
        let indent = "  ".repeat((adjusted_level - 1) as usize);

        if numbered {
            counters[level as usize] += 1;
            // Reset counters for lower levels
            for counter in counters.iter_mut().skip(level as usize + 1) {
                *counter = 0;
            }
            let number = format_number(&counters, level as usize);
            if links {
                let slug = utils::slugify(&text);
                output.push_str(&format!(
                    "{}- [{} {}](#{})\n",
                    indent, number, text, slug
                ));
            } else {
                output.push_str(&format!("{}- {} {}\n", indent, number, text));
            }
        } else if links {
            let slug = utils::slugify(&text);
            output.push_str(&format!("{}- [{}](#{})\n", indent, text, slug));
        } else {
            output.push_str(&format!("{}- {}\n", indent, text));
        }
    }

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)
}

fn parse_levels(levels: &str) -> anyhow::Result<(u8, u8)> {
    if let Some((min, max)) = levels.split_once('-') {
        let min: u8 = min.parse()?;
        let max: u8 = max.parse()?;
        Ok((min.clamp(1, 6), max.clamp(1, 6)))
    } else {
        let level: u8 = levels.parse()?;
        Ok((level.clamp(1, 6), level.clamp(1, 6)))
    }
}

fn format_number(counters: &[usize], level: usize) -> String {
    let parts: Vec<_> = counters[1..=level]
        .iter()
        .take_while(|&&c| c > 0)
        .map(|c| c.to_string())
        .collect();
    parts.join(".")
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
