use clap::{Arg, ArgMatches, Command};

use crate::cmd::utils;

pub fn make_subcommand() -> Command {
    Command::new("html")
        .about("Convert HTML to Markdown")
        .arg(
            Arg::new("input")
                .help("Input HTML file (default: stdin)")
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file (default: stdout)"),
        )
}

pub fn execute(matches: &ArgMatches) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;

    let md = clmd::from::html_to_markdown(&input);

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &md)
}
