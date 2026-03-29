use clap::{crate_authors, crate_version, Arg, ArgAction, ColorChoice, Command};

mod cmd;

fn main() -> anyhow::Result<()> {
    let app = Command::new("clmd")
        .version(crate_version!())
        .author(crate_authors!())
        .about("clmd: CommonMark Markdown processor")
        .propagate_version(true)
        .arg_required_else_help(true)
        .color(ColorChoice::Auto)
        .arg(
            Arg::new("extension")
                .short('e')
                .long("extension")
                .action(ArgAction::Append)
                .help("Enable extensions (table, strikethrough, tasklist, footnotes)"),
        )
        .arg(
            Arg::new("safe")
                .long("safe")
                .action(ArgAction::SetTrue)
                .help("Enable safe mode (filter dangerous HTML)"),
        )
        .subcommand(cmd::to::make_subcommand())
        .subcommand(cmd::from::make_subcommand())
        .subcommand(cmd::extract::make_subcommand())
        .subcommand(cmd::stats::make_subcommand())
        .subcommand(cmd::toc::make_subcommand())
        .subcommand(cmd::fmt::make_subcommand())
        .after_help(
            r###"Subcommand groups:

* Conversion: to (html, xml), from (html)
* Formatting: fmt
* Extraction: extract (links, headings, code)
* Analysis: stats
* Utilities: toc

Examples:
  clmd to html README.md
  clmd fmt input.md
  clmd stats input.md
  clmd toc input.md
"###,
        );

    let matches = app.get_matches();

    // Build options from global flags
    let mut options = clmd::Options::default();

    // Handle extensions
    if let Some(extensions) = matches.get_many::<String>("extension") {
        for ext in extensions {
            match ext.as_str() {
                "table" => options.extension.table = true,
                "strikethrough" => options.extension.strikethrough = true,
                "tasklist" => options.extension.tasklist = true,
                "footnotes" => options.extension.footnotes = true,
                "autolink" => options.extension.autolink = true,
                "tagfilter" => options.extension.tagfilter = true,
                _ => eprintln!("Warning: unknown extension '{}'", ext),
            }
        }
    }

    // Handle safe mode
    if matches.get_flag("safe") {
        options.render.r#unsafe = false;
    }

    match matches.subcommand() {
        Some(("to", sub_matches)) => cmd::to::execute(sub_matches, &options),
        Some(("from", sub_matches)) => cmd::from::execute(sub_matches, &options),
        Some(("extract", sub_matches)) => cmd::extract::execute(sub_matches, &options),
        Some(("stats", sub_matches)) => cmd::stats::execute(sub_matches, &options),
        Some(("toc", sub_matches)) => cmd::toc::execute(sub_matches, &options),
        Some(("fmt", sub_matches)) => cmd::fmt::execute(sub_matches, &options),
        _ => unreachable!(),
    }
}
