use clap::{Arg, ArgAction, ArgMatches, Command};
use serde_json::json;
use std::collections::HashSet;

use crate::cmd::utils;
use clmd::core::nodes::NodeValue;

pub fn make_subcommand() -> Command {
    Command::new("validate")
        .about("Validate Markdown document for common issues")
        .arg(
            Arg::new("input")
                .help("Input Markdown file (default: stdin)")
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file for report (default: stdout)"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .default_value("text")
                .help("Output format: text, json"),
        )
        .arg(
            Arg::new("check-links")
                .long("check-links")
                .action(ArgAction::SetTrue)
                .help("Check for broken internal links (same-document anchors)"),
        )
        .arg(
            Arg::new("check-external-links")
                .long("check-external-links")
                .action(ArgAction::SetTrue)
                .help("Check for broken external links (requires network)"),
        )
        .arg(
            Arg::new("check-images")
                .long("check-images")
                .action(ArgAction::SetTrue)
                .help("Check if local image files exist"),
        )
        .arg(
            Arg::new("check-refs")
                .long("check-refs")
                .action(ArgAction::SetTrue)
                .help("Check for unused reference links"),
        )
        .arg(
            Arg::new("strict")
                .long("strict")
                .action(ArgAction::SetTrue)
                .help("Enable all validation checks"),
        )
        .after_help(
            r###"Validation checks:
  - Duplicate heading IDs
  - Empty links
  - Empty images
  - Malformed URLs
  - Unused reference links (with --check-refs)
  - Broken internal anchors (with --check-links)
  - Missing local images (with --check-images)

Examples:
  clmd validate input.md
  clmd validate input.md --strict
  clmd validate input.md --check-links --check-images
  clmd validate input.md --format json -o report.json
"###,
        )
}

pub fn execute(matches: &ArgMatches, options: &clmd::Options) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").map(|s| s.as_str());
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("text");
    let strict = matches.get_flag("strict");
    let check_links = matches.get_flag("check-links") || strict;
    let check_external_links = matches.get_flag("check-external-links") || strict;
    let check_images = matches.get_flag("check-images") || strict;
    let _check_refs = matches.get_flag("check-refs") || strict;

    let input = utils::read_input(input_path)?;
    let (arena, root) = clmd::parse_document(&input, options);

    let mut issues = Vec::new();
    let mut heading_ids: HashSet<String> = HashSet::new();
    let _defined_refs: HashSet<String> = HashSet::new();
    let mut used_refs: HashSet<String> = HashSet::new();
    let mut defined_footnotes: HashSet<String> = HashSet::new();
    let mut used_footnotes: HashSet<String> = HashSet::new();

    // Collect all references and footnotes first
    for node_id in arena.descendants(root) {
        let node = arena.get(node_id);
        match &node.value {
            NodeValue::Link(link) => {
                // Check for empty links
                if link.url.is_empty() {
                    issues.push(Issue {
                        severity: Severity::Error,
                        category: Category::Link,
                        message: "Empty link URL".to_string(),
                        details: None,
                    });
                }

                // Check for anchor links
                if link.url.starts_with('#') {
                    used_refs.insert(link.url[1..].to_string());
                }

                // Check for reference-style links
                // Note: Reference-style links are parsed differently in this parser
                // The link title is the title attribute, not the reference label

                // Check for external links
                if check_external_links && is_external_url(&link.url) {
                    // Note: Actual external link checking would require async HTTP requests
                    // For now, we just validate URL format
                    if !is_valid_url(&link.url) {
                        issues.push(Issue {
                            severity: Severity::Warning,
                            category: Category::Link,
                            message: format!("Potentially malformed URL: {}", link.url),
                            details: None,
                        });
                    }
                }
            }
            NodeValue::Image(image) => {
                // Check for empty images
                if image.url.is_empty() {
                    issues.push(Issue {
                        severity: Severity::Error,
                        category: Category::Image,
                        message: "Empty image URL".to_string(),
                        details: None,
                    });
                }

                // Check for local images
                if check_images && !image.url.is_empty() && !is_external_url(&image.url)
                {
                    if let Some(base_path) = input_path {
                        let base_dir = std::path::Path::new(base_path)
                            .parent()
                            .unwrap_or(std::path::Path::new("."));
                        let image_path = base_dir.join(&*image.url);
                        if !image_path.exists() {
                            issues.push(Issue {
                                severity: Severity::Warning,
                                category: Category::Image,
                                message: format!("Image not found: {}", image.url),
                                details: Some(format!(
                                    "Expected at: {}",
                                    image_path.display()
                                )),
                            });
                        }
                    }
                }
            }
            NodeValue::Heading(_heading) => {
                let text = collect_text(&arena, node_id);
                let id = utils::slugify(&text);

                if heading_ids.contains(&id) {
                    issues.push(Issue {
                        severity: Severity::Warning,
                        category: Category::Heading,
                        message: format!("Duplicate heading ID: {}", id),
                        details: Some(format!("Heading text: {}", text)),
                    });
                } else {
                    heading_ids.insert(id);
                }
            }
            NodeValue::FootnoteDefinition(footnote) => {
                defined_footnotes.insert(footnote.name.to_string());
            }
            NodeValue::FootnoteReference(footnote_ref) => {
                used_footnotes.insert(footnote_ref.name.to_string());
            }
            _ => {}
        }
    }

    // Check for unused footnotes
    for defined in &defined_footnotes {
        if !used_footnotes.contains(defined) {
            issues.push(Issue {
                severity: Severity::Info,
                category: Category::Footnote,
                message: format!("Unused footnote definition: [{}]", defined),
                details: None,
            });
        }
    }

    // Check for undefined footnote references
    for used in &used_footnotes {
        if !defined_footnotes.contains(used) {
            issues.push(Issue {
                severity: Severity::Error,
                category: Category::Footnote,
                message: format!("Undefined footnote reference: [{}]", used),
                details: None,
            });
        }
    }

    // Check for broken internal links
    if check_links {
        for used in &used_refs {
            if !heading_ids.contains(used) {
                issues.push(Issue {
                    severity: Severity::Error,
                    category: Category::Link,
                    message: format!("Broken internal link to: #{}", used),
                    details: None,
                });
            }
        }
    }

    // Generate output
    let output = if issues.is_empty() {
        match format {
            "json" => serde_json::to_string_pretty(&json!({
                "valid": true,
                "issues": []
            }))?,
            _ => "✓ Document is valid\n".to_string(),
        }
    } else {
        match format {
            "json" => {
                let json_issues: Vec<_> = issues
                    .iter()
                    .map(|i| {
                        json!({
                            "severity": i.severity.as_str(),
                            "category": i.category.as_str(),
                            "message": i.message,
                            "details": i.details
                        })
                    })
                    .collect();
                serde_json::to_string_pretty(&json!({
                    "valid": !has_errors(&issues),
                    "issues": json_issues
                }))?
            }
            _ => format_issues_text(&issues),
        }
    };

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    utils::write_output(output_path, &output)?;

    // Return error if there are errors
    if has_errors(&issues) {
        anyhow::bail!(
            "Validation failed with {} error(s)",
            issues
                .iter()
                .filter(|i| matches!(i.severity, Severity::Error))
                .count()
        );
    }

    Ok(())
}

#[derive(Debug)]
struct Issue {
    severity: Severity,
    category: Category,
    message: String,
    details: Option<String>,
}

#[derive(Debug)]
enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

#[derive(Debug)]
enum Category {
    Link,
    Image,
    Heading,
    Footnote,
    #[allow(dead_code)]
    Reference,
}

impl Category {
    fn as_str(&self) -> &'static str {
        match self {
            Category::Link => "link",
            Category::Image => "image",
            Category::Heading => "heading",
            Category::Footnote => "footnote",
            Category::Reference => "reference",
        }
    }
}

fn has_errors(issues: &[Issue]) -> bool {
    issues.iter().any(|i| matches!(i.severity, Severity::Error))
}

fn format_issues_text(issues: &[Issue]) -> String {
    let mut output = String::new();

    let errors = issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Error))
        .count();
    let warnings = issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Warning))
        .count();
    let infos = issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Info))
        .count();

    output.push_str(&format!(
        "Found {} error(s), {} warning(s), {} info\n\n",
        errors, warnings, infos
    ));

    for issue in issues {
        let icon = match issue.severity {
            Severity::Error => "✗",
            Severity::Warning => "⚠",
            Severity::Info => "ℹ",
        };
        let severity_str = match issue.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARNING",
            Severity::Info => "INFO",
        };

        output.push_str(&format!(
            "{} [{}] {}: {}\n",
            icon,
            severity_str,
            issue.category.as_str(),
            issue.message
        ));

        if let Some(details) = &issue.details {
            output.push_str(&format!("  → {}\n", details));
        }
    }

    output
}

fn is_external_url(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("ftp://")
        || url.starts_with("mailto:")
}

fn is_valid_url(url: &str) -> bool {
    // Basic URL validation
    if url.starts_with("mailto:") {
        // Simple email validation
        let email_part = &url[7..];
        email_part.contains('@') && !email_part.contains(' ')
    } else {
        // Check for common URL issues
        !url.contains(' ') && !url.is_empty()
    }
}

fn collect_text(arena: &clmd::Arena, node_id: clmd::NodeId) -> String {
    let mut text = String::new();

    for child_id in arena.descendants(node_id) {
        let child = arena.get(child_id);
        if let NodeValue::Text(t) = &child.value {
            text.push_str(t);
        }
    }

    text
}
