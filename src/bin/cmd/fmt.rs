use clap::{Arg, ArgMatches, Command};

use crate::cmd::utils;

pub fn make_subcommand() -> Command {
    Command::new("fmt")
        .about("Format Markdown to canonical CommonMark")
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

    let cm = clmd::markdown_to_commonmark(&input, options);

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &cm)
}
