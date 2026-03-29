use clap::{ArgMatches, Command};

pub mod html;
pub mod xml;

pub fn make_subcommand() -> Command {
    Command::new("to")
        .about("Convert Markdown to various formats")
        .subcommand_required(true)
        .subcommand(html::make_subcommand())
        .subcommand(xml::make_subcommand())
}

pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some(("html", sub_matches)) => html::execute(sub_matches, options),
        Some(("xml", sub_matches)) => xml::execute(sub_matches, options),
        _ => unreachable!(),
    }
}
