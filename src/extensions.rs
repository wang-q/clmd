//! Markdown extensions management using bitflags.
//!
//! This module provides a unified way to manage Markdown extensions,
//! inspired by Pandoc's extension system and pulldown-cmark's bitflags approach.
//!
//! # Example
//!
//! ```
//! use clmd::extensions::{Extensions, GFM_EXTENSIONS, ALL_EXTENSIONS};
//!
//! // Create GFM-compatible extension set
//! let gfm = GFM_EXTENSIONS;
//! assert!(gfm.contains(Extensions::TABLES));
//! assert!(gfm.contains(Extensions::STRIKETHROUGH));
//!
//! // Add additional extensions
//! let extended = gfm | Extensions::FOOTNOTES | Extensions::DESCRIPTION_LISTS;
//!
//! // Check if specific extension is enabled
//! if extended.contains(Extensions::FOOTNOTES) {
//!     // Enable footnote parsing
//! }
//! ```

use bitflags::bitflags;
use std::fmt;
use std::str::FromStr;

bitflags! {
    /// Markdown extension flags.
    ///
    /// Each flag represents a specific Markdown extension that can be enabled
    /// or disabled independently. Extensions can be combined using bitwise
    /// operators.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::extensions::Extensions;
    ///
    /// // Combine multiple extensions
    /// let ext = Extensions::TABLES | Extensions::STRIKETHROUGH | Extensions::TASKLISTS;
    ///
    /// // Check if an extension is enabled
    /// assert!(ext.contains(Extensions::TABLES));
    ///
    /// // Remove an extension
    /// let without_tables = ext - Extensions::TABLES;
    /// assert!(!without_tables.contains(Extensions::TABLES));
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Extensions: u64 {
        // GFM Extensions
        /// Tables extension (GFM).
        ///
        /// Enables pipe table syntax:
        /// ```markdown
        /// | Header | Header |
        /// |--------|--------|
        /// | Cell   | Cell   |
        /// ```
        const TABLES = 1 << 0;

        /// Strikethrough extension (GFM).
        ///
        /// Enables strikethrough syntax: `~~deleted~~`
        const STRIKETHROUGH = 1 << 1;

        /// Task list extension (GFM).
        ///
        /// Enables task list items:
        /// ```markdown
        /// - [ ] Unchecked item
        /// - [x] Checked item
        /// ```
        const TASKLISTS = 1 << 2;

        /// Autolink extension (GFM).
        ///
        /// Automatically converts URLs and email addresses to links.
        const AUTOLINKS = 1 << 3;

        /// Tag filter extension (GFM).
        ///
        /// Filters out certain HTML tags for security.
        const TAGFILTER = 1 << 4;

        // Document Enhancement Extensions
        /// Footnotes extension.
        ///
        /// Enables footnote syntax:
        /// ```markdown
        /// Text with footnote[^1].
        ///
        /// [^1]: Footnote content.
        /// ```
        const FOOTNOTES = 1 << 5;

        /// Description lists extension.
        ///
        /// Enables definition list syntax:
        /// ```markdown
        /// Term
        /// : Definition
        /// ```
        const DESCRIPTION_LISTS = 1 << 6;

        /// Abbreviation extension.
        ///
        /// Enables abbreviation definitions:
        /// ```markdown
        /// *[HTML]: Hyper Text Markup Language
        /// ```
        const ABBREVIATIONS = 1 << 7;

        /// Table of contents extension.
        ///
        /// Enables `[TOC]` placeholder for automatic table of contents generation.
        const TOC = 1 << 8;

        /// YAML front matter extension.
        ///
        /// Enables YAML metadata at the beginning of documents:
        /// ```markdown
        /// ---
        /// title: My Document
        /// ---
        /// ```
        const YAML_FRONT_MATTER = 1 << 9;

        // Typography Extensions
        /// Smart punctuation extension.
        ///
        /// Converts straight quotes to curly, `--` to en-dash, `---` to em-dash,
        /// and `...` to ellipsis.
        const SMART_PUNCTUATION = 1 << 10;

        /// Superscript extension.
        ///
        /// Enables superscript text using `^` delimiters: `x^2^`
        const SUPERSCRIPT = 1 << 11;

        /// Subscript extension.
        ///
        /// Enables subscript text using `~` delimiters: `H~2~O`
        ///
        /// Note: If strikethrough is also enabled, single tilde is reserved
        /// for subscript and double tilde for strikethrough.
        const SUBSCRIPT = 1 << 12;

        /// Highlight extension.
        ///
        /// Enables highlighting using `==` delimiters: `==highlighted==`
        const HIGHLIGHT = 1 << 13;

        /// Underline extension.
        ///
        /// Enables underline using `++` delimiters: `++underlined++`
        const UNDERLINE = 1 << 14;

        /// Insert extension.
        ///
        /// Enables inserted text using `++` delimiters.
        const INSERT = 1 << 15;

        /// Spoiler extension.
        ///
        /// Enables spoiler text using `||` delimiters: `||spoiler||`
        const SPOILER = 1 << 16;

        // Link Extensions
        /// WikiLinks with title after pipe.
        ///
        /// Enables `[[target|title]]` syntax.
        const WIKILINKS_TITLE_AFTER_PIPE = 1 << 17;

        /// WikiLinks with title before pipe.
        ///
        /// Enables `[[title|target]]` syntax.
        const WIKILINKS_TITLE_BEFORE_PIPE = 1 << 18;

        // Math Extensions
        /// Math using dollar syntax.
        ///
        /// Enables `$...$` for inline math and `$$...$$` for display math.
        const MATH_DOLLARS = 1 << 19;

        /// Math using code syntax.
        ///
        /// Enables math in code blocks with specific language tags.
        const MATH_CODE = 1 << 20;

        // Attribute Extensions
        /// Header IDs extension.
        ///
        /// Adds IDs to headers based on their content.
        const HEADER_IDS = 1 << 21;

        /// Attributes extension.
        ///
        /// Enables adding IDs, classes, and custom attributes to elements:
        /// ```markdown
        /// # Heading {#id .class key=value}
        /// ```
        const ATTRIBUTES = 1 << 22;

        // Special Extensions
        /// Shortcodes extension.
        ///
        /// Enables emoji shortcodes: `:thumbsup:` -> 👍
        const SHORTCODES = 1 << 23;

        /// Alerts extension (GitHub-style).
        ///
        /// Enables GitHub-style alert blocks:
        /// ```markdown
        /// > [!NOTE]
        /// > This is a note.
        /// ```
        const ALERTS = 1 << 24;

        /// Multiline block quotes extension.
        ///
        /// Enables block quotes that can contain blank lines.
        const MULTILINE_BLOCK_QUOTES = 1 << 25;

        /// Greentext extension.
        ///
        /// Requires a space after `>` for blockquotes (4chan-style).
        const GREENTEXT = 1 << 26;

        /// CJK-friendly emphasis extension.
        ///
        /// Recognizes emphasis patterns common in CJK text.
        const CJK_FRIENDLY_EMPHASIS = 1 << 27;

        /// Subtext extension.
        ///
        /// Enables block-scoped subscript that acts like a header:
        /// ```markdown
        /// -# subtext
        /// ```
        const SUBTEXT = 1 << 28;

        /// Inline footnotes extension.
        ///
        /// Enables inline footnote syntax `^[content]`.
        const INLINE_FOOTNOTES = 1 << 29;
    }
}

impl Extensions {
    /// Check if any GFM extensions are enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::extensions::{Extensions, GFM_EXTENSIONS};
    ///
    /// assert!(GFM_EXTENSIONS.has_gfm_extensions());
    /// assert!(!Extensions::empty().has_gfm_extensions());
    /// ```
    pub fn has_gfm_extensions(&self) -> bool {
        self.intersects(
            Extensions::TABLES
                | Extensions::STRIKETHROUGH
                | Extensions::TASKLISTS
                | Extensions::AUTOLINKS
                | Extensions::TAGFILTER,
        )
    }

    /// Get the WikiLinks mode if either wikilinks option is enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::extensions::Extensions;
    /// use clmd::parser::options::WikiLinksMode;
    ///
    /// let ext = Extensions::WIKILINKS_TITLE_AFTER_PIPE;
    /// assert_eq!(ext.wikilinks_mode(), Some(WikiLinksMode::UrlFirst));
    /// ```
    pub fn wikilinks_mode(&self) -> Option<crate::parser::options::WikiLinksMode> {
        use crate::parser::options::WikiLinksMode;
        match (
            self.contains(Extensions::WIKILINKS_TITLE_BEFORE_PIPE),
            self.contains(Extensions::WIKILINKS_TITLE_AFTER_PIPE),
        ) {
            (false, false) => None,
            (true, false) => Some(WikiLinksMode::TitleFirst),
            (false, true) => Some(WikiLinksMode::UrlFirst),
            (true, true) => Some(WikiLinksMode::TitleFirst), // Default to title-first if both enabled
        }
    }

    /// Convert to a comma-separated list of extension names.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::extensions::Extensions;
    ///
    /// let ext = Extensions::TABLES | Extensions::STRIKETHROUGH;
    /// let names = ext.to_names();
    /// assert!(names.contains("tables"));
    /// assert!(names.contains("strikethrough"));
    /// ```
    pub fn to_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();

        if self.contains(Extensions::TABLES) {
            names.push("tables");
        }
        if self.contains(Extensions::STRIKETHROUGH) {
            names.push("strikethrough");
        }
        if self.contains(Extensions::TASKLISTS) {
            names.push("tasklists");
        }
        if self.contains(Extensions::AUTOLINKS) {
            names.push("autolinks");
        }
        if self.contains(Extensions::TAGFILTER) {
            names.push("tagfilter");
        }
        if self.contains(Extensions::FOOTNOTES) {
            names.push("footnotes");
        }
        if self.contains(Extensions::DESCRIPTION_LISTS) {
            names.push("description_lists");
        }
        if self.contains(Extensions::ABBREVIATIONS) {
            names.push("abbreviations");
        }
        if self.contains(Extensions::TOC) {
            names.push("toc");
        }
        if self.contains(Extensions::YAML_FRONT_MATTER) {
            names.push("yaml_front_matter");
        }
        if self.contains(Extensions::SMART_PUNCTUATION) {
            names.push("smart_punctuation");
        }
        if self.contains(Extensions::SUPERSCRIPT) {
            names.push("superscript");
        }
        if self.contains(Extensions::SUBSCRIPT) {
            names.push("subscript");
        }
        if self.contains(Extensions::HIGHLIGHT) {
            names.push("highlight");
        }
        if self.contains(Extensions::UNDERLINE) {
            names.push("underline");
        }
        if self.contains(Extensions::INSERT) {
            names.push("insert");
        }
        if self.contains(Extensions::SPOILER) {
            names.push("spoiler");
        }
        if self.contains(Extensions::WIKILINKS_TITLE_AFTER_PIPE) {
            names.push("wikilinks_title_after_pipe");
        }
        if self.contains(Extensions::WIKILINKS_TITLE_BEFORE_PIPE) {
            names.push("wikilinks_title_before_pipe");
        }
        if self.contains(Extensions::MATH_DOLLARS) {
            names.push("math_dollars");
        }
        if self.contains(Extensions::MATH_CODE) {
            names.push("math_code");
        }
        if self.contains(Extensions::HEADER_IDS) {
            names.push("header_ids");
        }
        if self.contains(Extensions::ATTRIBUTES) {
            names.push("attributes");
        }
        if self.contains(Extensions::SHORTCODES) {
            names.push("shortcodes");
        }
        if self.contains(Extensions::ALERTS) {
            names.push("alerts");
        }
        if self.contains(Extensions::MULTILINE_BLOCK_QUOTES) {
            names.push("multiline_block_quotes");
        }
        if self.contains(Extensions::GREENTEXT) {
            names.push("greentext");
        }
        if self.contains(Extensions::CJK_FRIENDLY_EMPHASIS) {
            names.push("cjk_friendly_emphasis");
        }
        if self.contains(Extensions::SUBTEXT) {
            names.push("subtext");
        }
        if self.contains(Extensions::INLINE_FOOTNOTES) {
            names.push("inline_footnotes");
        }

        names
    }
}

impl fmt::Display for Extensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = self.to_names();
        if names.is_empty() {
            write!(f, "none")
        } else {
            write!(f, "{}", names.join(", "))
        }
    }
}

impl FromStr for Extensions {
    type Err = String;

    /// Parse extension names from a comma-separated string.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::extensions::Extensions;
    /// use std::str::FromStr;
    ///
    /// let ext = Extensions::from_str("tables,strikethrough,footnotes").unwrap();
    /// assert!(ext.contains(Extensions::TABLES));
    /// assert!(ext.contains(Extensions::STRIKETHROUGH));
    /// assert!(ext.contains(Extensions::FOOTNOTES));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ext = Extensions::empty();

        for name in s.split(',').map(|s| s.trim()) {
            if name.is_empty() {
                continue;
            }

            let flag = match name {
                "tables" | "table" => Extensions::TABLES,
                "strikethrough" | "strike" => Extensions::STRIKETHROUGH,
                "tasklists" | "tasklist" => Extensions::TASKLISTS,
                "autolinks" | "autolink" => Extensions::AUTOLINKS,
                "tagfilter" => Extensions::TAGFILTER,
                "footnotes" | "footnote" => Extensions::FOOTNOTES,
                "description_lists" | "definition_lists" | "deflists" => {
                    Extensions::DESCRIPTION_LISTS
                }
                "abbreviations" | "abbreviation" | "abbr" => Extensions::ABBREVIATIONS,
                "toc" => Extensions::TOC,
                "yaml_front_matter" | "front_matter" | "yaml" => Extensions::YAML_FRONT_MATTER,
                "smart_punctuation" | "smart" => Extensions::SMART_PUNCTUATION,
                "superscript" | "sup" => Extensions::SUPERSCRIPT,
                "subscript" | "sub" => Extensions::SUBSCRIPT,
                "highlight" | "mark" => Extensions::HIGHLIGHT,
                "underline" => Extensions::UNDERLINE,
                "insert" | "ins" => Extensions::INSERT,
                "spoiler" => Extensions::SPOILER,
                "wikilinks_title_after_pipe" | "wikilinks_after" => {
                    Extensions::WIKILINKS_TITLE_AFTER_PIPE
                }
                "wikilinks_title_before_pipe" | "wikilinks_before" => {
                    Extensions::WIKILINKS_TITLE_BEFORE_PIPE
                }
                "math_dollars" | "math" => Extensions::MATH_DOLLARS,
                "math_code" => Extensions::MATH_CODE,
                "header_ids" | "header_id" => Extensions::HEADER_IDS,
                "attributes" | "attr" => Extensions::ATTRIBUTES,
                "shortcodes" | "shortcode" => Extensions::SHORTCODES,
                "alerts" | "alert" => Extensions::ALERTS,
                "multiline_block_quotes" | "multiline_quotes" => {
                    Extensions::MULTILINE_BLOCK_QUOTES
                }
                "greentext" => Extensions::GREENTEXT,
                "cjk_friendly_emphasis" | "cjk_emphasis" => Extensions::CJK_FRIENDLY_EMPHASIS,
                "subtext" => Extensions::SUBTEXT,
                "inline_footnotes" | "inline_footnote" => Extensions::INLINE_FOOTNOTES,
                _ => return Err(format!("Unknown extension: {}", name)),
            };

            ext |= flag;
        }

        Ok(ext)
    }
}

/// GitHub Flavored Markdown extensions.
///
/// This includes: tables, strikethrough, tasklists, autolinks, and tagfilter.
pub const GFM_EXTENSIONS: Extensions = Extensions::TABLES
    .union(Extensions::STRIKETHROUGH)
    .union(Extensions::TASKLISTS)
    .union(Extensions::AUTOLINKS)
    .union(Extensions::TAGFILTER);

/// Common useful extensions for documentation.
///
/// This includes GFM extensions plus: footnotes, description lists,
/// table of contents, and YAML front matter.
pub const DOC_EXTENSIONS: Extensions = GFM_EXTENSIONS
    .union(Extensions::FOOTNOTES)
    .union(Extensions::DESCRIPTION_LISTS)
    .union(Extensions::TOC)
    .union(Extensions::YAML_FRONT_MATTER);

/// All available extensions.
///
/// Note: Some extensions may conflict with each other (e.g., subscript
/// and strikethrough both use `~` delimiter).
pub const ALL_EXTENSIONS: Extensions = Extensions::all();

/// No extensions enabled.
pub const NO_EXTENSIONS: Extensions = Extensions::empty();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_extensions() {
        let ext = Extensions::empty();
        assert!(!ext.contains(Extensions::TABLES));
        assert!(!ext.contains(Extensions::STRIKETHROUGH));
    }

    #[test]
    fn test_single_extension() {
        let ext = Extensions::TABLES;
        assert!(ext.contains(Extensions::TABLES));
        assert!(!ext.contains(Extensions::STRIKETHROUGH));
    }

    #[test]
    fn test_combine_extensions() {
        let ext = Extensions::TABLES | Extensions::STRIKETHROUGH;
        assert!(ext.contains(Extensions::TABLES));
        assert!(ext.contains(Extensions::STRIKETHROUGH));
        assert!(!ext.contains(Extensions::TASKLISTS));
    }

    #[test]
    fn test_remove_extension() {
        let ext = Extensions::TABLES | Extensions::STRIKETHROUGH;
        let without_tables = ext - Extensions::TABLES;
        assert!(!without_tables.contains(Extensions::TABLES));
        assert!(without_tables.contains(Extensions::STRIKETHROUGH));
    }

    #[test]
    fn test_gfm_extensions() {
        assert!(GFM_EXTENSIONS.contains(Extensions::TABLES));
        assert!(GFM_EXTENSIONS.contains(Extensions::STRIKETHROUGH));
        assert!(GFM_EXTENSIONS.contains(Extensions::TASKLISTS));
        assert!(GFM_EXTENSIONS.contains(Extensions::AUTOLINKS));
        assert!(GFM_EXTENSIONS.contains(Extensions::TAGFILTER));
        assert!(!GFM_EXTENSIONS.contains(Extensions::FOOTNOTES));
    }

    #[test]
    fn test_has_gfm_extensions() {
        assert!(GFM_EXTENSIONS.has_gfm_extensions());
        assert!(!Extensions::empty().has_gfm_extensions());
        assert!(Extensions::TABLES.has_gfm_extensions());
        assert!(!Extensions::FOOTNOTES.has_gfm_extensions());
    }

    #[test]
    fn test_parse_extensions() {
        let ext = Extensions::from_str("tables,strikethrough").unwrap();
        assert!(ext.contains(Extensions::TABLES));
        assert!(ext.contains(Extensions::STRIKETHROUGH));
    }

    #[test]
    fn test_parse_extensions_with_spaces() {
        let ext = Extensions::from_str("tables, strikethrough , footnotes").unwrap();
        assert!(ext.contains(Extensions::TABLES));
        assert!(ext.contains(Extensions::STRIKETHROUGH));
        assert!(ext.contains(Extensions::FOOTNOTES));
    }

    #[test]
    fn test_parse_unknown_extension() {
        let result = Extensions::from_str("tables,unknown");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown"));
    }

    #[test]
    fn test_parse_empty_string() {
        let ext = Extensions::from_str("").unwrap();
        assert_eq!(ext, Extensions::empty());
    }

    #[test]
    fn test_to_names() {
        let ext = Extensions::TABLES | Extensions::STRIKETHROUGH;
        let names = ext.to_names();
        assert!(names.contains(&"tables"));
        assert!(names.contains(&"strikethrough"));
        assert!(!names.contains(&"footnotes"));
    }

    #[test]
    fn test_display() {
        let ext = Extensions::TABLES | Extensions::STRIKETHROUGH;
        let s = format!("{}", ext);
        assert!(s.contains("tables"));
        assert!(s.contains("strikethrough"));
    }

    #[test]
    fn test_display_empty() {
        let ext = Extensions::empty();
        assert_eq!(format!("{}", ext), "none");
    }

    #[test]
    fn test_wikilinks_mode() {
        use crate::parser::options::WikiLinksMode;

        let ext = Extensions::WIKILINKS_TITLE_AFTER_PIPE;
        assert_eq!(ext.wikilinks_mode(), Some(WikiLinksMode::UrlFirst));

        let ext = Extensions::WIKILINKS_TITLE_BEFORE_PIPE;
        assert_eq!(ext.wikilinks_mode(), Some(WikiLinksMode::TitleFirst));

        let ext = Extensions::empty();
        assert_eq!(ext.wikilinks_mode(), None);
    }
}
