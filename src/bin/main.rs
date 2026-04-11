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
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path (TOML format)"),
        )
        .arg(
            Arg::new("extension")
                .short('e')
                .long("extension")
                .action(ArgAction::Append)
                .help("Enable extensions (table, strikethrough, tasklist, footnotes, tagfilter, superscript, subscript, underline, highlight, math, wikilink, spoiler, alerts)"),
        )
        .arg(
            Arg::new("safe")
                .long("safe")
                .action(ArgAction::SetTrue)
                .help("Enable safe mode (filter dangerous HTML)"),
        )
        .subcommand(cmd::to::make_subcommand())
        .subcommand(cmd::extract::make_subcommand())
        .subcommand(cmd::stats::make_subcommand())
        .subcommand(cmd::toc::make_subcommand())
        .subcommand(cmd::fmt::make_subcommand())
        .subcommand(cmd::validate::make_subcommand())
        .subcommand(cmd::transform::make_subcommand())
        .subcommand(cmd::complete::make_subcommand())
        .after_help(
            r###"Subcommand groups:

* Conversion: to
* Formatting: fmt
* Extraction: extract (links, images, headings, code, tables, footnotes, yaml-front-matter, task-items)
* Analysis: stats, validate
* Transformation: transform (shift-headings, normalize-links, strip)
* Utilities: toc, complete

Configuration:
  clmd looks for a configuration file at:
    - $XDG_CONFIG_HOME/clmd/config.toml
    - ~/.config/clmd/config.toml

Examples:
  clmd to html README.md
  clmd to latex input.md -o output.tex
  clmd extract links input.md
  clmd extract tables input.md --format csv
  clmd stats input.md --readability
  clmd validate input.md --strict
  clmd transform shift-headings input.md -s -1
  clmd complete bash > /etc/bash_completion.d/clmd
  clmd fmt input.md
  clmd toc input.md
  clmd -c /path/to/config.toml to html input.md
"###,
        );

    let matches = app.get_matches();

    // Build options from configuration file first
    let mut options = clmd::Options::default();

    // Load configuration file if specified or found at default location
    if let Some(config_path) = matches.get_one::<String>("config") {
        match clmd::context::Config::from_file(config_path) {
            Ok(config) => {
                config.apply_to_options(&mut options);
            }
            Err(e) => {
                eprintln!("Error: failed to load config file '{}': {}", config_path, e);
                std::process::exit(1);
            }
        }
    } else if let Some(config) = clmd::context::Config::load_default() {
        config.apply_to_options(&mut options);
    }

    // All extensions are enabled by default

    // Handle safe mode (command line overrides config file)
    if matches.get_flag("safe") {
        options.render.r#unsafe = false;
    }

    match matches.subcommand() {
        Some(("to", sub_matches)) => cmd::to::execute(sub_matches, &options),
        Some(("extract", sub_matches)) => cmd::extract::execute(sub_matches, &options),
        Some(("stats", sub_matches)) => cmd::stats::execute(sub_matches, &options),
        Some(("toc", sub_matches)) => cmd::toc::execute(sub_matches, &options),
        Some(("fmt", sub_matches)) => cmd::fmt::execute(sub_matches, &options),
        Some(("validate", sub_matches)) => cmd::validate::execute(sub_matches, &options),
        Some(("transform", sub_matches)) => {
            cmd::transform::execute(sub_matches, &options)
        }
        Some(("complete", sub_matches)) => cmd::complete::execute(sub_matches, &options),
        _ => unreachable!(),
    }
}
