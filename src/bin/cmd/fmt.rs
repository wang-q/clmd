use clap::{Arg, ArgAction, ArgMatches, Command};

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
        .arg(
            Arg::new("cjk-spacing")
                .long("cjk-spacing")
                .action(ArgAction::SetTrue)
                .help("Add spaces between CJK characters and English/numbers"),
        )
}

pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let in_place = matches.get_flag("in-place");
    let backup = matches.get_flag("backup");

    // Validate arguments
    if in_place && matches.get_one::<String>("output").is_some() {
        return Err(anyhow::anyhow!("Cannot use --in-place with --output"));
    }

    if in_place && input_path.is_none() {
        return Err(anyhow::anyhow!("--in-place requires an input file"));
    }

    let input = utils::read_input(input_path)?;

    // Parse width argument
    let width = matches
        .get_one::<String>("width")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(80);

    // Get cjk-spacing flag
    let cjk_spacing = matches.get_flag("cjk-spacing");

    // Create options with width setting and enable extensions
    let mut options = options.clone();
    options.render.width = width;
    options.render.cjk_spacing = cjk_spacing;
    options.extension.table = true;
    options.extension.tasklist = true;

    let cm = clmd::markdown_to_commonmark(&input, &options);

    if in_place {
        let path = input_path.unwrap();

        // Create backup if requested
        if backup {
            let backup_path = format!("{}.bak", path);
            std::fs::copy(path, &backup_path).map_err(|e| {
                anyhow::anyhow!("Failed to create backup '{}': {}", backup_path, e)
            })?;
        }

        // Write to temporary file first for atomic operation
        let temp_path = format!("{}.tmp", path);
        std::fs::write(&temp_path, &cm).map_err(|e| {
            anyhow::anyhow!("Failed to write temp file '{}': {}", temp_path, e)
        })?;

        // Atomically rename temp file to target file
        std::fs::rename(&temp_path, path).map_err(|e| {
            anyhow::anyhow!("Failed to rename temp file to '{}': {}", path, e)
        })?;

        Ok(())
    } else {
        let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
        utils::write_output(output_path, &cm)
    }
}
