use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::cmd::utils;

/// Make the convert subcommand.
pub fn make_subcommand() -> Command {
    Command::new("convert")
        .about("Convert between Markdown and other formats")
        .subcommand_required(true)
        .subcommand(make_to_html_subcommand())
        .subcommand(make_to_xml_subcommand())
        .subcommand(make_from_html_subcommand())
}

/// Execute the convert subcommand.
pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some(("to-html", sub_matches)) => execute_to_html(sub_matches, options),
        Some(("to-xml", sub_matches)) => execute_to_xml(sub_matches, options),
        Some(("from-html", sub_matches)) => execute_from_html(sub_matches),
        _ => unreachable!(),
    }
}

// ============================================================================
// to-html subcommand
// ============================================================================

fn make_to_html_subcommand() -> Command {
    Command::new("to-html")
        .about("Convert Markdown to HTML")
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
            Arg::new("full")
                .long("full")
                .action(ArgAction::SetTrue)
                .help("Generate full HTML document with <html><body> tags"),
        )
        .arg(
            Arg::new("hardbreaks")
                .long("hardbreaks")
                .action(ArgAction::SetTrue)
                .help("Convert newlines to <br>"),
        )
}

fn execute_to_html(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;

    let mut opts = options.clone();
    if matches.get_flag("hardbreaks") {
        opts.render.hardbreaks = true;
    }

    let html = clmd::markdown_to_html(&input, &opts);

    let output = if matches.get_flag("full") {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>Markdown Document</title>
</head>
<body>
{}
</body>
</html>"#,
            html
        )
    } else {
        html
    };

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)
}

// ============================================================================
// to-xml subcommand
// ============================================================================

fn make_to_xml_subcommand() -> Command {
    Command::new("to-xml")
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

fn execute_to_xml(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;

    let xml = clmd::markdown_to_commonmark_xml(&input, options);

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &xml)
}

// ============================================================================
// from-html subcommand
// ============================================================================

fn make_from_html_subcommand() -> Command {
    Command::new("from-html")
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

fn execute_from_html(matches: &ArgMatches) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let input = utils::read_input(input_path)?;

    let md = clmd::io::convert::html_to_markdown(&input);

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &md)
}
