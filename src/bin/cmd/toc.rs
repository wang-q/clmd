use clap::{Arg, ArgMatches, Command};
use regex::Regex;

use crate::cmd::utils;
use clmd::core::nodes::NodeValue;

/// TOC generation options
#[derive(Debug, Clone)]
pub struct TocOptions {
    pub min_level: u8,
    pub max_level: u8,
    pub numbered: bool,
    pub links: bool,
}

impl Default for TocOptions {
    fn default() -> Self {
        Self {
            min_level: 1,
            max_level: 6,
            numbered: false,
            links: true,
        }
    }
}

impl TocOptions {
    pub fn parse(marker: &str) -> Self {
        let mut options = Self::default();

        let re = Regex::new(r"\[TOC\s*([^\]]*)\]\s*:").unwrap();
        if let Some(caps) = re.captures(marker) {
            if let Some(opts_str) = caps.get(1) {
                let opts = opts_str.as_str().trim();

                for part in opts.split_whitespace() {
                    if part.starts_with("levels=") {
                        if let Some(levels) = part.strip_prefix("levels=") {
                            if let Ok((min, max)) = parse_levels(levels) {
                                options.min_level = min;
                                options.max_level = max;
                            }
                        }
                    } else if part == "numbered" {
                        options.numbered = true;
                    } else if part == "no-links" {
                        options.links = false;
                    }
                }
            }
        }

        options
    }
}

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
    let headings = collect_headings(&arena, root, min_level, max_level);

    if headings.is_empty() {
        return Ok(());
    }

    let toc_options = TocOptions {
        min_level,
        max_level,
        numbered,
        links,
    };
    let output = generate_toc(&headings, &toc_options);

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)
}

pub fn collect_headings(
    arena: &clmd::Arena,
    root: clmd::NodeId,
    min_level: u8,
    max_level: u8,
) -> Vec<(u8, String)> {
    let mut headings = Vec::new();

    for node_id in arena.descendants(root) {
        let node = arena.get(node_id);
        if let NodeValue::Heading(heading) = &node.value {
            let level = heading.level;
            if level >= min_level && level <= max_level {
                let text = collect_text(arena, node_id);
                headings.push((level, text));
            }
        }
    }

    headings
}

pub fn generate_toc(headings: &[(u8, String)], options: &TocOptions) -> String {
    if headings.is_empty() {
        return String::new();
    }

    let min_heading_level = headings.iter().map(|(l, _)| *l).min().unwrap_or(1);

    let mut output = String::new();
    let mut counters = vec![0; 7];

    for (level, text) in headings {
        let adjusted_level = level - min_heading_level + 1;
        let indent = "  ".repeat((adjusted_level - 1) as usize);

        if options.numbered {
            counters[adjusted_level as usize] += 1;
            for counter in counters.iter_mut().skip(adjusted_level as usize + 1) {
                *counter = 0;
            }
            let number = format_number_for_level(&counters, adjusted_level as usize);
            if options.links {
                let slug = utils::slugify(text);
                output.push_str(&format!(
                    "{}- [{} {}](#{})\n",
                    indent, number, text, slug
                ));
            } else {
                output.push_str(&format!("{}- {} {}\n", indent, number, text));
            }
        } else if options.links {
            let slug = utils::slugify(text);
            output.push_str(&format!("{}- [{}](#{})\n", indent, text, slug));
        } else {
            output.push_str(&format!("{}- {}\n", indent, text));
        }
    }

    output
}

pub fn parse_levels(levels: &str) -> anyhow::Result<(u8, u8)> {
    if let Some((min, max)) = levels.split_once('-') {
        let min: u8 = min.parse()?;
        let max: u8 = max.parse()?;
        Ok((min.clamp(1, 6), max.clamp(1, 6)))
    } else {
        let level: u8 = levels.parse()?;
        Ok((level.clamp(1, 6), level.clamp(1, 6)))
    }
}

fn format_number_for_level(counters: &[usize], adjusted_level: usize) -> String {
    let parts: Vec<_> = counters[1..=adjusted_level]
        .iter()
        .map(|c| c.to_string())
        .collect();
    parts.join(".")
}

pub fn collect_text(arena: &clmd::Arena, node_id: clmd::NodeId) -> String {
    let mut text = String::new();

    for child_id in arena.descendants(node_id) {
        let child = arena.get(child_id);
        if let NodeValue::Text(t) = &child.value {
            text.push_str(t);
        }
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toc_options_default() {
        let opts = TocOptions::default();
        assert_eq!(opts.min_level, 1);
        assert_eq!(opts.max_level, 6);
        assert!(!opts.numbered);
        assert!(opts.links);
    }

    #[test]
    fn test_toc_options_parse_default() {
        let opts = TocOptions::parse("[TOC]: #");
        assert_eq!(opts.min_level, 1);
        assert_eq!(opts.max_level, 6);
        assert!(!opts.numbered);
        assert!(opts.links);
    }

    #[test]
    fn test_toc_options_parse_levels() {
        let opts = TocOptions::parse("[TOC levels=2-4]: #");
        assert_eq!(opts.min_level, 2);
        assert_eq!(opts.max_level, 4);
    }

    #[test]
    fn test_toc_options_parse_numbered() {
        let opts = TocOptions::parse("[TOC numbered]: #");
        assert!(opts.numbered);
    }

    #[test]
    fn test_toc_options_parse_combined() {
        let opts = TocOptions::parse("[TOC levels=2-3 numbered]: #");
        assert_eq!(opts.min_level, 2);
        assert_eq!(opts.max_level, 3);
        assert!(opts.numbered);
    }

    #[test]
    fn test_toc_options_parse_no_links() {
        let opts = TocOptions::parse("[TOC no-links]: #");
        assert!(!opts.links);
    }

    #[test]
    fn test_parse_levels_range() {
        assert_eq!(parse_levels("1-6").unwrap(), (1, 6));
        assert_eq!(parse_levels("2-4").unwrap(), (2, 4));
    }

    #[test]
    fn test_parse_levels_single() {
        assert_eq!(parse_levels("2").unwrap(), (2, 2));
    }

    #[test]
    fn test_parse_levels_clamp() {
        assert_eq!(parse_levels("0-7").unwrap(), (1, 6));
    }
}
