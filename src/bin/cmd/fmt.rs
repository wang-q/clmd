use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::cmd::toc::{collect_headings, generate_toc, TocOptions};
use crate::cmd::utils;

pub fn make_subcommand() -> Command {
    Command::new("fmt")
        .about("Format Markdown to canonical CommonMark/GFM")
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
            Arg::new("in-place")
                .short('i')
                .long("in-place")
                .action(ArgAction::SetTrue)
                .help("Edit file in-place (cannot be used with --output)"),
        )
        .arg(
            Arg::new("backup")
                .short('b')
                .long("backup")
                .action(ArgAction::SetTrue)
                .help("Create backup file when using --in-place"),
        )
        .arg(
            Arg::new("width")
                .long("width")
                .default_value("80")
                .help("Line width limit for wrapping"),
        )
}

pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let in_place = matches.get_flag("in-place");
    let backup = matches.get_flag("backup");

    if in_place && matches.get_one::<String>("output").is_some() {
        return Err(anyhow::anyhow!("Cannot use --in-place with --output"));
    }

    if in_place && input_path.is_none() {
        return Err(anyhow::anyhow!("--in-place requires an input file"));
    }

    let input = utils::read_input(input_path)?;

    let toc_info = utils::find_toc_boundaries(&input);

    let input_for_format = if let Some(ref toc_info) = toc_info {
        remove_toc_content(&input, toc_info)
    } else {
        input.clone()
    };

    let width = matches
        .get_one::<String>("width")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(80);

    let mut fmt_options = options.clone();
    fmt_options.render.width = width;
    fmt_options.extension.table = true;
    fmt_options.extension.tasklist = true;

    let mut cm = clmd::markdown_to_commonmark(
        &input_for_format,
        &fmt_options,
        &clmd::Plugins::default(),
    );

    if let Some(toc_info) = toc_info {
        cm = reinsert_toc(&cm, &toc_info, &fmt_options);
    }

    if in_place {
        let path = input_path.unwrap();

        if backup {
            let backup_path = format!("{}.bak", path);
            std::fs::copy(path, &backup_path).map_err(|e| {
                anyhow::anyhow!("Failed to create backup '{}': {}", backup_path, e)
            })?;
        }

        let temp_path = format!("{}.tmp", path);
        std::fs::write(&temp_path, &cm).map_err(|e| {
            anyhow::anyhow!("Failed to write temp file '{}': {}", temp_path, e)
        })?;

        std::fs::rename(&temp_path, path).map_err(|e| {
            anyhow::anyhow!("Failed to rename temp file to '{}': {}", path, e)
        })?;

        Ok(())
    } else {
        let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
        utils::write_output(output_path, &cm)
    }
}

fn remove_toc_content(input: &str, toc_info: &utils::TocBoundaries) -> String {
    let marker_line_start = input
        .find(&toc_info.marker_line)
        .map(|pos| input[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0))
        .unwrap_or(0);

    let mut result = String::new();
    result.push_str(&input[..marker_line_start]);
    result.push_str(&input[toc_info.content_end..]);
    result
}

fn reinsert_toc(
    formatted: &str,
    toc_info: &utils::TocBoundaries,
    options: &clmd::Options,
) -> String {
    let toc_options = TocOptions::parse(&toc_info.marker_line);

    let (arena, root) = clmd::parse_document(formatted, options);
    let headings =
        collect_headings(&arena, root, toc_options.min_level, toc_options.max_level);

    let new_toc = generate_toc(&headings, &toc_options);

    if new_toc.is_empty() {
        return formatted.to_string();
    }

    let mut result = String::new();
    result.push_str(&toc_info.marker_line);
    result.push_str("\n\n");
    result.push_str(&new_toc);
    result.push_str(formatted);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_toc_content() {
        let input = "# Title\n\n[TOC]: #\n\n- [Old](#old)\n\n## Section\n\nContent";
        let toc_info = utils::TocBoundaries {
            marker_line: "[TOC]: #".to_string(),
            content_end: 35,
        };

        let result = remove_toc_content(input, &toc_info);
        assert!(!result.contains("[TOC]: #"));
        assert!(!result.contains("[Old](#old)"));
    }

    #[test]
    fn test_reinsert_toc() {
        let formatted = "# Title\n\n## Section\n\nContent";
        let toc_info = utils::TocBoundaries {
            marker_line: "[TOC]: #".to_string(),
            content_end: 0,
        };
        let options = clmd::Options::default();

        let result = reinsert_toc(formatted, &toc_info, &options);
        assert!(result.contains("[TOC]: #"));
        assert!(result.contains("[Title](#title)"));
        assert!(result.contains("[Section](#section)"));
    }
}
