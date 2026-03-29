use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::cmd::utils;

pub fn make_subcommand() -> Command {
    Command::new("html")
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

pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
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
