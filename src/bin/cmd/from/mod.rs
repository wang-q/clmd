use clap::{ArgMatches, Command};

pub mod html;

pub fn make_subcommand() -> Command {
    Command::new("from")
        .about("Convert other formats to Markdown")
        .subcommand_required(true)
        .subcommand(html::make_subcommand())
}

pub fn execute(matches: &ArgMatches, _options: &clmd::Options) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some(("html", sub_matches)) => html::execute(sub_matches),
        _ => unreachable!(),
    }
}
