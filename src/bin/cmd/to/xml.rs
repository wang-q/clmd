use clap::{Arg, ArgMatches, Command};

use crate::cmd::utils;

pub fn make_subcommand() -> Command {
    Command::new("xml")
        .about("Convert Markdown to CommonMark XML")
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
}

pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;

    let xml = clmd::markdown_to_commonmark_xml(&input, options);

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &xml)
}
