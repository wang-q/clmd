//! Markdown extensions
//!
//! This module contains various extensions to the CommonMark specification,
//! including GitHub Flavored Markdown (GFM) features and other commonly used extensions.

/// Abbreviation extension
///
/// Allows defining abbreviations that will be wrapped in <abbr> tags.
/// Syntax: `*[HTML]: Hyper Text Markup Language`
pub mod abbreviation;

/// Attributes extension
///
/// Adds support for adding IDs, classes, and custom attributes to Markdown elements.
/// Syntax: `# Heading {#id .class key=value}`
pub mod attributes;

/// Autolink extension (GFM)
///
/// Automatically converts URLs and email addresses to links.
/// Syntax: `https://example.com` or `email@example.com`
pub mod autolink;

/// Definition list extension
///
/// Adds support for definition lists.
/// Syntax:
/// ```markdown
/// Term
/// : Definition
/// ```
pub mod definition;

/// Footnote extension
///
/// Adds support for footnotes.
/// Syntax: `[^1]` and `[^1]: Footnote content`
pub mod footnotes;

/// Strikethrough extension (GFM)
///
/// Adds support for strikethrough text.
/// Syntax: `~~deleted~~`
pub mod strikethrough;

/// Tables extension (GFM)
///
/// Adds support for tables.
/// Syntax:
/// ```markdown
/// | Header | Header |
/// |--------|--------|
/// | Cell   | Cell   |
/// ```
pub mod tables;

/// Task list extension (GFM)
///
/// Adds support for task list items with checkboxes.
/// Syntax: `- [ ] Unchecked` or `- [x] Checked`
pub mod tasklist;

/// Table of Contents extension
///
/// Generates a table of contents from document headings.
/// Syntax: `[TOC]`
pub mod toc;

/// YAML Front Matter extension
///
/// Adds support for YAML metadata at the beginning of documents.
/// Syntax: `---\ntitle: My Doc\n---`
pub mod yaml_front_matter;
