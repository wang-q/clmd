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
                .help("Enable extensions (table, strikethrough, tasklist, footnotes, tagfilter)"),
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

Configuration:
  clmd looks for a configuration file at:
    - $XDG_CONFIG_HOME/clmd/config.toml
    - ~/.config/clmd/config.toml

Examples:
  clmd to html README.md
  clmd fmt input.md
  clmd stats input.md
  clmd toc input.md
  clmd -c /path/to/config.toml to html input.md
"###,
        );

    let matches = app.get_matches();

    // Build options from configuration file first
    let mut options = clmd::Options::default();

    // Load configuration file if specified or found at default location
    if let Some(config_path) = matches.get_one::<String>("config") {
        match clmd::context::config::Config::from_file(config_path) {
            Ok(config) => {
                config.apply_to_options(&mut options);
            }
            Err(e) => {
                eprintln!("Error: failed to load config file '{}': {}", config_path, e);
                std::process::exit(1);
            }
        }
    } else if let Some(config) = clmd::context::config::Config::load_default() {
        config.apply_to_options(&mut options);
    }

    // Handle extensions (command line overrides config file)
    if let Some(extensions) = matches.get_many::<String>("extension") {
        for ext in extensions {
            match ext.as_str() {
                "table" => options.extension.table = true,
                "strikethrough" => options.extension.strikethrough = true,
                "tasklist" => options.extension.tasklist = true,
                "footnotes" => options.extension.footnotes = true,
                "autolink" => options.extension.autolink = true,
                "tagfilter" => options.extension.tagfilter = true,
                "superscript" => options.extension.superscript = true,
                "subscript" => options.extension.subscript = true,
                "underline" => options.extension.underline = true,
                "highlight" => options.extension.highlight = true,
                "math" => options.extension.math_dollars = true,
                "wikilink" => options.extension.wikilinks_title_after_pipe = true,
                "spoiler" => options.extension.spoiler = true,
                "alerts" => options.extension.alerts = true,
                _ => eprintln!("Warning: unknown extension '{}'", ext),
            }
        }
    }

    // Handle safe mode (command line overrides config file)
    if matches.get_flag("safe") {
        options.render.r#unsafe = false;
        options.extension.tagfilter = true;
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
