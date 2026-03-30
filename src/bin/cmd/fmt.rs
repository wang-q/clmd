use clap::{Arg, ArgMatches, Command};

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
            Arg::new("width")
                .long("width")
                .default_value("80")
                .help("Line width limit for wrapping"),
        )
}

pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;

    // Parse width argument
    let width = matches
        .get_one::<String>("width")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(80);

    // Create options with width setting and enable extensions
    let mut options = options.clone();
    options.render.width = width;
    options.extension.table = true;
    options.extension.tasklist = true;

    let cm = clmd::markdown_to_commonmark(&input, &options);

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &cm)
}
