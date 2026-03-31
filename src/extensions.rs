//! Markdown extensions management using bitflags.
//!
//! This module provides a unified way to manage Markdown extensions,
//! inspired by Pandoc's extension system and pulldown-cmark's bitflags approach.
//!
//! # Example
//!
//! ```ignore
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
    /// ```ignore
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

        /// East Asian line breaks extension.
        ///
        /// Ignores newlines between East Asian characters.
        const EAST_ASIAN_LINE_BREAKS = 1 << 30;

        /// Raw attribute extension.
        ///
        /// Enables raw content with format specification:
        /// ```markdown
        /// `code`{=html}
        /// ```
        const RAW_ATTRIBUTE = 1 << 31;

        /// Fenced divs extension.
        ///
        /// Enables Pandoc-style fenced divs:
        /// ```markdown
        /// ::: warning
        /// This is a warning.
        /// :::
        /// ```
        const FENCED_DIVS = 1 << 32;

        /// Bracketed spans extension.
        ///
        /// Enables bracketed spans with attributes:
        /// ```markdown
        /// [text]{#id .class}
        /// ```
        const BRACKETED_SPANS = 1 << 33;

        /// Citations extension.
        ///
        /// Enables citation syntax:
        /// ```markdown
        /// [@smith2000]
        /// [see @doe2010, pp. 23-25]
        /// ```
        const CITATIONS = 1 << 34;

        /// YAML metadata block extension.
        ///
        /// Enables YAML metadata block at the beginning or end of document.
        const YAML_METADATA_BLOCK = 1 << 35;

        /// Hard line breaks extension.
        ///
        /// Treats newlines as hard line breaks.
        const HARD_LINE_BREAKS = 1 << 36;

        /// Strikeout extension.
        ///
        /// Alternative to strikethrough using `~~` delimiters.
        const STRIKEOUT = 1 << 37;

        /// Pipe tables extension.
        ///
        /// Explicit pipe table support (same as tables but explicit).
        const PIPE_TABLES = 1 << 38;

        /// Grid tables extension.
        ///
        /// Enables grid table syntax:
        /// ```markdown
        /// +---+---+---+
        /// | A | B | C |
        /// +---+---+---+
        /// ```
        const GRID_TABLES = 1 << 39;

        /// Multiline tables extension.
        ///
        /// Enables multiline table cells.
        const MULTILINE_TABLES = 1 << 40;

        /// Implicit figures extension.
        ///
        /// Images with alt text only become figures.
        const IMPLICIT_FIGURES = 1 << 41;

        /// Link attributes extension.
        ///
        /// Enables attributes on links:
        /// ```markdown
        /// [link](url){#id .class}
        /// ```
        const LINK_ATTRIBUTES = 1 << 42;

        /// Autoidentifiers extension.
        ///
        /// Automatically generate header identifiers from text.
        const AUTO_IDENTIFIERS = 1 << 43;

        /// Compact lists extension.
        ///
        /// Use compact list style when possible.
        const COMPACT_LISTS = 1 << 44;

        /// Fancy lists extension.
        ///
        /// Enables fancy list markers (uppercase letters, Roman numerals).
        const FANCY_LISTS = 1 << 45;

        /// Start number extension.
        ///
        /// Respect the starting number of ordered lists.
        const START_NUMBER = 1 << 46;

        /// Intraword emphasis extension.
        ///
        /// Allows emphasis within words.
        const INTRAWORD_EMPHASIS = 1 << 47;

        /// All symbols escaped extension.
        ///
        /// Escape all symbols that could be special.
        const ALL_SYMBOLS_ESCAPED = 1 << 48;

        /// Angle brackets escaped extension.
        ///
        /// Escape `<` and `>` as `\<` and `\>`.
        const ANGLE_BRACKETS_ESCAPED = 1 << 49;

        /// Raw HTML extension.
        ///
        /// Allow raw HTML in output.
        const RAW_HTML = 1 << 50;

        /// Raw TeX extension.
        ///
        /// Allow raw TeX in output.
        const RAW_TEX = 1 << 51;

        /// Tex math single backslash extension.
        ///
        /// Use `\(...\)` for inline and `\[...\]` for display math.
        const TEX_MATH_SINGLE_BACKSLASH = 1 << 52;

        /// Tex math double backslash extension.
        ///
        /// Use `\\(...\\)` for inline and `\\[...\\]` for display math.
        const TEX_MATH_DOUBLE_BACKSLASH = 1 << 53;

        /// Latex math extension.
        ///
        /// Use LaTeX math environments.
        const LATEX_MATH = 1 << 54;

        /// Emoji extension.
        ///
        /// Convert emoji shortcodes and Unicode emoji.
        const EMOJI = 1 << 55;

        /// Pandoc title block extension.
        ///
        /// Enables Pandoc-style title block at document start.
        const PANDOC_TITLE_BLOCK = 1 << 56;

        /// MMD title block extension.
        ///
        /// Enables MultiMarkdown-style title block.
        const MMD_TITLE_BLOCK = 1 << 57;

        /// Example lists extension.
        ///
        /// Enables example list numbering (@).
        const EXAMPLE_LISTS = 1 << 58;

        /// Line blocks extension.
        ///
        /// Enables line block syntax (| at start of line).
        const LINE_BLOCKS = 1 << 59;

        /// Blank before blockquote extension.
        ///
        /// Require blank line before blockquote.
        const BLANK_BEFORE_BLOCKQUOTE = 1 << 60;

        /// Blank before header extension.
        ///
        /// Require blank line before header.
        const BLANK_BEFORE_HEADER = 1 << 61;

        /// Indent code extension.
        ///
        /// Use indented code blocks.
        const INDENTED_CODE = 1 << 62;

        /// Backtick code extension.
        ///
        /// Use fenced code blocks with backticks.
        const BACKTICK_CODE = 1 << 63;
    }
}

impl Extensions {
    /// Check if any GFM extensions are enabled.
    ///
    /// # Example
    ///
    /// ```ignore
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

    /// Get extensions for a specific format.
    ///
    /// Returns the default set of extensions for the given format.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::extensions::Extensions;
    ///
    /// let gfm = Extensions::for_format("gfm");
    /// assert!(gfm.contains(Extensions::TABLES));
    ///
    /// let cm = Extensions::for_format("commonmark");
    /// assert!(!cm.contains(Extensions::TABLES));
    /// ```
    pub fn for_format(format: &str) -> Self {
        match format {
            // GitHub Flavored Markdown
            "gfm" | "markdown_github" => Self::gfm(),
            // CommonMark (no extensions)
            "commonmark" | "cm" => Self::empty(),
            // Standard Markdown with documentation extensions
            "markdown" | "md" => Self::doc(),
            // Pandoc Markdown (rich set of extensions)
            "pandoc" | "markdown_pandoc" => Self::pandoc(),
            // MultiMarkdown
            "mmd" | "markdown_mmd" => Self::mmd(),
            // PHP Markdown Extra
            "markdown_phpextra" | "phpextra" => Self::phpextra(),
            // Strict Markdown (no extensions)
            "strict" | "markdown_strict" => Self::strict(),
            // Default to empty
            _ => Self::empty(),
        }
    }

    /// Get Pandoc Markdown extensions.
    ///
    /// This includes a rich set of extensions commonly used with Pandoc.
    pub fn pandoc() -> Self {
        Self::doc()
            .union(Extensions::FENCED_DIVS)
            .union(Extensions::BRACKETED_SPANS)
            .union(Extensions::CITATIONS)
            .union(Extensions::YAML_METADATA_BLOCK)
            .union(Extensions::PIPE_TABLES)
            .union(Extensions::GRID_TABLES)
            .union(Extensions::IMPLICIT_FIGURES)
            .union(Extensions::LINK_ATTRIBUTES)
            .union(Extensions::AUTO_IDENTIFIERS)
            .union(Extensions::FANCY_LISTS)
            .union(Extensions::START_NUMBER)
            .union(Extensions::INTRAWORD_EMPHASIS)
            .union(Extensions::RAW_ATTRIBUTE)
            .union(Extensions::PANDOC_TITLE_BLOCK)
            .union(Extensions::EXAMPLE_LISTS)
            .union(Extensions::LINE_BLOCKS)
            .union(Extensions::MATH_DOLLARS)
    }

    /// Get MultiMarkdown extensions.
    ///
    /// This includes extensions supported by MultiMarkdown.
    pub fn mmd() -> Self {
        Self::gfm()
            .union(Extensions::FOOTNOTES)
            .union(Extensions::TABLES)
            .union(Extensions::MATH_DOLLARS)
            .union(Extensions::HEADER_IDS)
            .union(Extensions::ATTRIBUTES)
            .union(Extensions::MMD_TITLE_BLOCK)
            .union(Extensions::LINK_ATTRIBUTES)
    }

    /// Get PHP Markdown Extra extensions.
    ///
    /// This includes extensions supported by PHP Markdown Extra.
    pub fn phpextra() -> Self {
        Self::TABLES
            .union(Extensions::FOOTNOTES)
            .union(Extensions::ABBREVIATIONS)
            .union(Extensions::HEADER_IDS)
            .union(Extensions::ATTRIBUTES)
            .union(Extensions::DESCRIPTION_LISTS)
            .union(Extensions::BACKTICK_CODE)
    }

    /// Get strict CommonMark (no extensions).
    ///
    /// This returns an empty extension set for strict CommonMark compliance.
    pub fn strict() -> Self {
        Self::empty()
    }

    /// Get GFM extensions.
    ///
    /// This is the same as `GFM_EXTENSIONS`.
    pub fn gfm() -> Self {
        GFM_EXTENSIONS
    }

    /// Get documentation extensions.
    ///
    /// This is the same as `DOC_EXTENSIONS`.
    pub fn doc() -> Self {
        DOC_EXTENSIONS
    }

    /// Get all available extensions.
    ///
    /// This is the same as `ALL_EXTENSIONS`.
    pub fn all_extensions() -> Self {
        ALL_EXTENSIONS
    }

    /// Get no extensions.
    ///
    /// This is the same as `NO_EXTENSIONS`.
    pub fn no_extensions() -> Self {
        NO_EXTENSIONS
    }

    /// Enable a specific extension.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::extensions::Extensions;
    ///
    /// let ext = Extensions::empty().enable_extension(Extensions::TABLES);
    /// assert!(ext.contains(Extensions::TABLES));
    /// ```
    pub fn enable_extension(&self, ext: Extensions) -> Self {
        *self | ext
    }

    /// Disable a specific extension.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::extensions::Extensions;
    ///
    /// let ext = Extensions::TABLES | Extensions::STRIKETHROUGH;
    /// let ext = ext.disable_extension(Extensions::TABLES);
    /// assert!(!ext.contains(Extensions::TABLES));
    /// assert!(ext.contains(Extensions::STRIKETHROUGH));
    /// ```
    pub fn disable_extension(&self, ext: Extensions) -> Self {
        *self - ext
    }

    /// Check if a specific extension is enabled.
    ///
    /// This is a more explicit alias for `contains()`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::extensions::Extensions;
    ///
    /// let ext = Extensions::TABLES | Extensions::STRIKETHROUGH;
    /// assert!(ext.extension_enabled(Extensions::TABLES));
    /// assert!(!ext.extension_enabled(Extensions::FOOTNOTES));
    /// ```
    pub fn extension_enabled(&self, ext: Extensions) -> bool {
        self.contains(ext)
    }

    /// Combine extensions with another set.
    ///
    /// This is equivalent to the `|` operator.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::extensions::Extensions;
    ///
    /// let ext = Extensions::TABLES.combine_extensions(Extensions::FOOTNOTES);
    /// assert!(ext.contains(Extensions::TABLES));
    /// assert!(ext.contains(Extensions::FOOTNOTES));
    /// ```
    pub fn combine_extensions(&self, other: Extensions) -> Self {
        *self | other
    }

    /// Get the difference between two extension sets.
    ///
    /// Returns extensions that are in `self` but not in `other`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::extensions::Extensions;
    ///
    /// let ext = Extensions::TABLES | Extensions::STRIKETHROUGH | Extensions::FOOTNOTES;
    /// let diff = ext.difference_extensions(Extensions::STRIKETHROUGH);
    /// assert!(diff.contains(Extensions::TABLES));
    /// assert!(!diff.contains(Extensions::STRIKETHROUGH));
    /// assert!(diff.contains(Extensions::FOOTNOTES));
    /// ```
    pub fn difference_extensions(&self, other: Extensions) -> Self {
        *self - other
    }

    /// Get the intersection of two extension sets.
    ///
    /// Returns extensions that are in both sets.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::extensions::Extensions;
    ///
    /// let ext1 = Extensions::TABLES | Extensions::STRIKETHROUGH;
    /// let ext2 = Extensions::TABLES | Extensions::FOOTNOTES;
    /// let intersection = ext1.intersection_extensions(ext2);
    /// assert!(intersection.contains(Extensions::TABLES));
    /// assert!(!intersection.contains(Extensions::STRIKETHROUGH));
    /// assert!(!intersection.contains(Extensions::FOOTNOTES));
    /// ```
    pub fn intersection_extensions(&self, other: Extensions) -> Self {
        *self & other
    }

    /// Toggle an extension (enable if disabled, disable if enabled).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::extensions::Extensions;
    ///
    /// let ext = Extensions::TABLES;
    /// let ext = ext.toggle_extension(Extensions::TABLES);
    /// assert!(!ext.contains(Extensions::TABLES));
    /// let ext = ext.toggle_extension(Extensions::TABLES);
    /// assert!(ext.contains(Extensions::TABLES));
    /// ```
    pub fn toggle_extension(&self, ext: Extensions) -> Self {
        *self ^ ext
    }

    /// Get the default extensions for a format.
    ///
    /// This is an alias for `for_format()` with a more explicit name.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::extensions::Extensions;
    ///
    /// let gfm = Extensions::get_default_extensions("gfm");
    /// assert!(gfm.contains(Extensions::TABLES));
    /// ```
    pub fn get_default_extensions(format: &str) -> Self {
        Self::for_format(format)
    }

    /// Get all available extensions.
    ///
    /// This is an alias for `all_extensions()`.
    pub fn get_all_extensions() -> Self {
        ALL_EXTENSIONS
    }

    /// Get the WikiLinks mode if either wikilinks option is enabled.
    ///
    /// # Example
    ///
    /// ```ignore
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
    /// ```ignore
    /// use clmd::extensions::Extensions;
    ///
    /// let ext = Extensions::TABLES | Extensions::STRIKETHROUGH;
    /// let names = ext.to_names();
    /// assert!(names.contains(&"tables"));
    /// assert!(names.contains(&"strikethrough"));
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
        if self.contains(Extensions::EAST_ASIAN_LINE_BREAKS) {
            names.push("east_asian_line_breaks");
        }
        if self.contains(Extensions::RAW_ATTRIBUTE) {
            names.push("raw_attribute");
        }
        if self.contains(Extensions::FENCED_DIVS) {
            names.push("fenced_divs");
        }
        if self.contains(Extensions::BRACKETED_SPANS) {
            names.push("bracketed_spans");
        }
        if self.contains(Extensions::CITATIONS) {
            names.push("citations");
        }
        if self.contains(Extensions::YAML_METADATA_BLOCK) {
            names.push("yaml_metadata_block");
        }
        if self.contains(Extensions::HARD_LINE_BREAKS) {
            names.push("hard_line_breaks");
        }
        if self.contains(Extensions::STRIKEOUT) {
            names.push("strikeout");
        }
        if self.contains(Extensions::PIPE_TABLES) {
            names.push("pipe_tables");
        }
        if self.contains(Extensions::GRID_TABLES) {
            names.push("grid_tables");
        }
        if self.contains(Extensions::MULTILINE_TABLES) {
            names.push("multiline_tables");
        }
        if self.contains(Extensions::IMPLICIT_FIGURES) {
            names.push("implicit_figures");
        }
        if self.contains(Extensions::LINK_ATTRIBUTES) {
            names.push("link_attributes");
        }
        if self.contains(Extensions::AUTO_IDENTIFIERS) {
            names.push("auto_identifiers");
        }
        if self.contains(Extensions::COMPACT_LISTS) {
            names.push("compact_lists");
        }
        if self.contains(Extensions::FANCY_LISTS) {
            names.push("fancy_lists");
        }
        if self.contains(Extensions::START_NUMBER) {
            names.push("start_number");
        }
        if self.contains(Extensions::INTRAWORD_EMPHASIS) {
            names.push("intraword_emphasis");
        }
        if self.contains(Extensions::ALL_SYMBOLS_ESCAPED) {
            names.push("all_symbols_escaped");
        }
        if self.contains(Extensions::ANGLE_BRACKETS_ESCAPED) {
            names.push("angle_brackets_escaped");
        }
        if self.contains(Extensions::RAW_HTML) {
            names.push("raw_html");
        }
        if self.contains(Extensions::RAW_TEX) {
            names.push("raw_tex");
        }
        if self.contains(Extensions::TEX_MATH_SINGLE_BACKSLASH) {
            names.push("tex_math_single_backslash");
        }
        if self.contains(Extensions::TEX_MATH_DOUBLE_BACKSLASH) {
            names.push("tex_math_double_backslash");
        }
        if self.contains(Extensions::LATEX_MATH) {
            names.push("latex_math");
        }
        if self.contains(Extensions::EMOJI) {
            names.push("emoji");
        }
        if self.contains(Extensions::PANDOC_TITLE_BLOCK) {
            names.push("pandoc_title_block");
        }
        if self.contains(Extensions::MMD_TITLE_BLOCK) {
            names.push("mmd_title_block");
        }
        if self.contains(Extensions::EXAMPLE_LISTS) {
            names.push("example_lists");
        }
        if self.contains(Extensions::LINE_BLOCKS) {
            names.push("line_blocks");
        }
        if self.contains(Extensions::BLANK_BEFORE_BLOCKQUOTE) {
            names.push("blank_before_blockquote");
        }
        if self.contains(Extensions::BLANK_BEFORE_HEADER) {
            names.push("blank_before_header");
        }
        if self.contains(Extensions::INDENTED_CODE) {
            names.push("indented_code");
        }
        if self.contains(Extensions::BACKTICK_CODE) {
            names.push("backtick_code");
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
    /// ```ignore
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
                "yaml_front_matter" | "front_matter" | "yaml" => {
                    Extensions::YAML_FRONT_MATTER
                }
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
                "cjk_friendly_emphasis" | "cjk_emphasis" => {
                    Extensions::CJK_FRIENDLY_EMPHASIS
                }
                "subtext" => Extensions::SUBTEXT,
                "inline_footnotes" | "inline_footnote" => Extensions::INLINE_FOOTNOTES,
                "east_asian_line_breaks" | "east_asian" => {
                    Extensions::EAST_ASIAN_LINE_BREAKS
                }
                "raw_attribute" | "raw_attr" => Extensions::RAW_ATTRIBUTE,
                "fenced_divs" | "fenced_div" => Extensions::FENCED_DIVS,
                "bracketed_spans" | "bracketed_span" => Extensions::BRACKETED_SPANS,
                "citations" | "citation" => Extensions::CITATIONS,
                "yaml_metadata_block" | "yaml_metadata" => {
                    Extensions::YAML_METADATA_BLOCK
                }
                "hard_line_breaks" | "hardbreaks" => Extensions::HARD_LINE_BREAKS,
                "strikeout" => Extensions::STRIKEOUT,
                "pipe_tables" | "pipe_table" => Extensions::PIPE_TABLES,
                "grid_tables" | "grid_table" => Extensions::GRID_TABLES,
                "multiline_tables" | "multiline_table" => Extensions::MULTILINE_TABLES,
                "implicit_figures" | "implicit_figure" => Extensions::IMPLICIT_FIGURES,
                "link_attributes" | "link_attr" => Extensions::LINK_ATTRIBUTES,
                "auto_identifiers" | "auto_id" => Extensions::AUTO_IDENTIFIERS,
                "compact_lists" | "compact_list" => Extensions::COMPACT_LISTS,
                "fancy_lists" | "fancy_list" => Extensions::FANCY_LISTS,
                "start_number" => Extensions::START_NUMBER,
                "intraword_emphasis" | "intraword" => Extensions::INTRAWORD_EMPHASIS,
                "all_symbols_escaped" | "all_symbols" => Extensions::ALL_SYMBOLS_ESCAPED,
                "angle_brackets_escaped" | "angle_brackets" => {
                    Extensions::ANGLE_BRACKETS_ESCAPED
                }
                "raw_html" => Extensions::RAW_HTML,
                "raw_tex" => Extensions::RAW_TEX,
                "tex_math_single_backslash" | "tex_math_single" => {
                    Extensions::TEX_MATH_SINGLE_BACKSLASH
                }
                "tex_math_double_backslash" | "tex_math_double" => {
                    Extensions::TEX_MATH_DOUBLE_BACKSLASH
                }
                "latex_math" | "latex" => Extensions::LATEX_MATH,
                "emoji" => Extensions::EMOJI,
                "pandoc_title_block" | "pandoc_title" => Extensions::PANDOC_TITLE_BLOCK,
                "mmd_title_block" | "mmd_title" => Extensions::MMD_TITLE_BLOCK,
                "example_lists" | "example_list" => Extensions::EXAMPLE_LISTS,
                "line_blocks" | "line_block" => Extensions::LINE_BLOCKS,
                "blank_before_blockquote" | "blank_before_quote" => {
                    Extensions::BLANK_BEFORE_BLOCKQUOTE
                }
                "blank_before_header" | "blank_before_heading" => {
                    Extensions::BLANK_BEFORE_HEADER
                }
                "indented_code" => Extensions::INDENTED_CODE,
                "backtick_code" => Extensions::BACKTICK_CODE,
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

/// Extension difference for format specification.
///
/// This structure represents the difference between two extension sets,
/// similar to Pandoc's ExtensionsDiff. It tracks which extensions to add
/// and which to remove from a base set.
///
/// This is useful when parsing format specifications like `markdown+smart-tasklists`
/// where `+smart` means "add smart extension" and `-tasklists` means "remove tasklists".
///
/// # Example
///
/// ```
/// use clmd::extensions::{Extensions, ExtensionsDiff};
///
/// let diff = ExtensionsDiff {
///     add: Extensions::SMART_PUNCTUATION | Extensions::FOOTNOTES,
///     remove: Extensions::TASKLISTS,
/// };
///
/// let base = Extensions::TABLES | Extensions::TASKLISTS;
/// let result = diff.apply(base);
///
/// assert!(result.contains(Extensions::TABLES));
/// assert!(result.contains(Extensions::SMART_PUNCTUATION));
/// assert!(result.contains(Extensions::FOOTNOTES));
/// assert!(!result.contains(Extensions::TASKLISTS));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExtensionsDiff {
    /// Extensions to add.
    pub add: Extensions,
    /// Extensions to remove.
    pub remove: Extensions,
}

impl ExtensionsDiff {
    /// Create a new empty diff.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a diff that adds extensions.
    pub fn add(ext: Extensions) -> Self {
        Self {
            add: ext,
            remove: Extensions::empty(),
        }
    }

    /// Create a diff that removes extensions.
    pub fn remove(ext: Extensions) -> Self {
        Self {
            add: Extensions::empty(),
            remove: ext,
        }
    }

    /// Apply this diff to a base set of extensions.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::extensions::{Extensions, ExtensionsDiff};
    ///
    /// let diff = ExtensionsDiff::add(Extensions::FOOTNOTES);
    /// let base = Extensions::TABLES;
    /// let result = diff.apply(base);
    ///
    /// assert!(result.contains(Extensions::TABLES));
    /// assert!(result.contains(Extensions::FOOTNOTES));
    /// ```
    pub fn apply(&self, base: Extensions) -> Extensions {
        (base | self.add) - self.remove
    }

    /// Combine two diffs.
    ///
    /// The result will add all extensions that either diff adds (except those
    /// removed by either), and remove all extensions that either diff removes
    /// (except those added by either).
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::extensions::{Extensions, ExtensionsDiff};
    ///
    /// let diff1 = ExtensionsDiff::add(Extensions::FOOTNOTES);
    /// let diff2 = ExtensionsDiff::remove(Extensions::TASKLISTS);
    /// let combined = diff1.combine(diff2);
    ///
    /// let base = Extensions::TABLES | Extensions::TASKLISTS;
    /// let result = combined.apply(base);
    ///
    /// assert!(result.contains(Extensions::TABLES));
    /// assert!(result.contains(Extensions::FOOTNOTES));
    /// assert!(!result.contains(Extensions::TASKLISTS));
    /// ```
    pub fn combine(&self, other: ExtensionsDiff) -> Self {
        Self {
            add: (self.add | other.add) - self.remove - other.remove,
            remove: (self.remove | other.remove) - self.add - other.add,
        }
    }

    /// Parse a diff from a format specification string.
    ///
    /// Format: `+ext1-ext2+ext3` or `ext1,-ext2,+ext3`
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::extensions::{Extensions, ExtensionsDiff};
    ///
    /// let diff = ExtensionsDiff::parse("+smart-tasklists+footnotes").unwrap();
    /// assert!(diff.add.contains(Extensions::SMART_PUNCTUATION));
    /// assert!(diff.add.contains(Extensions::FOOTNOTES));
    /// assert!(diff.remove.contains(Extensions::TASKLISTS));
    /// ```
    pub fn parse(s: &str) -> Result<Self, String> {
        let mut diff = Self::new();

        // Split by + or -, keeping the delimiters
        let mut current = String::new();
        let mut sign = '+';

        for c in s.chars() {
            if c == '+' || c == '-' {
                // Process the previous extension
                if !current.is_empty() {
                    let ext = Self::parse_extension(&current)?;
                    if sign == '+' {
                        diff.add |= ext;
                    } else {
                        diff.remove |= ext;
                    }
                    current.clear();
                }
                sign = c;
            } else {
                current.push(c);
            }
        }

        // Process the last extension
        if !current.is_empty() {
            let ext = Self::parse_extension(&current)?;
            if sign == '+' {
                diff.add |= ext;
            } else {
                diff.remove |= ext;
            }
        }

        Ok(diff)
    }

    /// Parse a single extension name.
    fn parse_extension(name: &str) -> Result<Extensions, String> {
        let name = name.trim();
        if name.is_empty() {
            return Ok(Extensions::empty());
        }

        // Use the same parsing logic as Extensions::from_str
        let ext = Extensions::from_str(name)?;
        Ok(ext)
    }

    /// Check if this diff is empty (no changes).
    pub fn is_empty(&self) -> bool {
        self.add.is_empty() && self.remove.is_empty()
    }
}

/// Parse a flavored format specification.
///
/// This function parses format strings like `markdown+smart-tasklists` or
/// `gfm-hard_line_breaks` and returns the base format with extension differences.
///
/// # Example
///
/// ```ignore
/// use clmd::extensions::parse_flavored_format;
///
/// let (base_ext, diff) = parse_flavored_format("markdown+smart-tasklists").unwrap();
/// // base_ext is the default markdown extensions
/// // diff.add contains SMART_PUNCTUATION and FOOTNOTES
/// // diff.remove contains TASKLISTS
/// ```
pub fn parse_flavored_format(s: &str) -> Result<(Extensions, ExtensionsDiff), String> {
    // Split the string into format name and extension modifiers
    let parts: Vec<&str> = s.split(|c| c == '+' || c == '-').collect();

    if parts.is_empty() {
        return Err("Empty format specification".to_string());
    }

    // Get the base format
    let format_name = parts[0].trim();
    let base_ext = Extensions::for_format(format_name);

    // Parse the extension diff
    let diff = ExtensionsDiff::parse(&s[format_name.len()..])?;

    Ok((base_ext, diff))
}

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

    #[test]
    fn test_enable_extension() {
        let ext = Extensions::empty().enable_extension(Extensions::TABLES);
        assert!(ext.contains(Extensions::TABLES));
        assert!(!ext.contains(Extensions::STRIKETHROUGH));
    }

    #[test]
    fn test_disable_extension() {
        let ext = Extensions::TABLES | Extensions::STRIKETHROUGH;
        let ext = ext.disable_extension(Extensions::TABLES);
        assert!(!ext.contains(Extensions::TABLES));
        assert!(ext.contains(Extensions::STRIKETHROUGH));
    }

    #[test]
    fn test_extension_enabled() {
        let ext = Extensions::TABLES | Extensions::STRIKETHROUGH;
        assert!(ext.extension_enabled(Extensions::TABLES));
        assert!(!ext.extension_enabled(Extensions::FOOTNOTES));
    }

    #[test]
    fn test_difference_extensions() {
        let ext = Extensions::TABLES | Extensions::STRIKETHROUGH | Extensions::FOOTNOTES;
        let diff = ext.difference_extensions(Extensions::STRIKETHROUGH);
        assert!(diff.contains(Extensions::TABLES));
        assert!(!diff.contains(Extensions::STRIKETHROUGH));
        assert!(diff.contains(Extensions::FOOTNOTES));
    }

    #[test]
    fn test_intersection_extensions() {
        let ext1 = Extensions::TABLES | Extensions::STRIKETHROUGH;
        let ext2 = Extensions::TABLES | Extensions::FOOTNOTES;
        let intersection = ext1.intersection_extensions(ext2);
        assert!(intersection.contains(Extensions::TABLES));
        assert!(!intersection.contains(Extensions::STRIKETHROUGH));
        assert!(!intersection.contains(Extensions::FOOTNOTES));
    }

    #[test]
    fn test_toggle_extension() {
        let ext = Extensions::TABLES;
        let ext = ext.toggle_extension(Extensions::TABLES);
        assert!(!ext.contains(Extensions::TABLES));
        let ext = ext.toggle_extension(Extensions::TABLES);
        assert!(ext.contains(Extensions::TABLES));
    }

    #[test]
    fn test_new_extensions() {
        // Test that new extensions can be created and combined
        let ext = Extensions::FENCED_DIVS
            | Extensions::BRACKETED_SPANS
            | Extensions::CITATIONS
            | Extensions::YAML_METADATA_BLOCK
            | Extensions::RAW_ATTRIBUTE
            | Extensions::EMOJI;

        assert!(ext.contains(Extensions::FENCED_DIVS));
        assert!(ext.contains(Extensions::BRACKETED_SPANS));
        assert!(ext.contains(Extensions::CITATIONS));
        assert!(ext.contains(Extensions::YAML_METADATA_BLOCK));
        assert!(ext.contains(Extensions::RAW_ATTRIBUTE));
        assert!(ext.contains(Extensions::EMOJI));
    }

    #[test]
    fn test_pandoc_extensions() {
        let ext = Extensions::pandoc();
        assert!(ext.contains(Extensions::FENCED_DIVS));
        assert!(ext.contains(Extensions::CITATIONS));
        assert!(ext.contains(Extensions::YAML_METADATA_BLOCK));
        assert!(ext.contains(Extensions::TABLES));
    }

    #[test]
    fn test_mmd_extensions() {
        let ext = Extensions::mmd();
        assert!(ext.contains(Extensions::TABLES));
        assert!(ext.contains(Extensions::FOOTNOTES));
        assert!(ext.contains(Extensions::MMD_TITLE_BLOCK));
    }

    #[test]
    fn test_phpextra_extensions() {
        let ext = Extensions::phpextra();
        assert!(ext.contains(Extensions::TABLES));
        assert!(ext.contains(Extensions::FOOTNOTES));
        assert!(ext.contains(Extensions::DESCRIPTION_LISTS));
    }

    #[test]
    fn test_strict_extensions() {
        let ext = Extensions::strict();
        assert_eq!(ext, Extensions::empty());
    }

    #[test]
    fn test_format_extensions() {
        assert!(Extensions::for_format("pandoc").contains(Extensions::FENCED_DIVS));
        assert!(Extensions::for_format("mmd").contains(Extensions::MMD_TITLE_BLOCK));
        assert!(Extensions::for_format("phpextra").contains(Extensions::TABLES));
        assert_eq!(Extensions::for_format("strict"), Extensions::empty());
    }

    #[test]
    fn test_parse_new_extensions() {
        let ext = Extensions::from_str("fenced_divs,citations,emoji").unwrap();
        assert!(ext.contains(Extensions::FENCED_DIVS));
        assert!(ext.contains(Extensions::CITATIONS));
        assert!(ext.contains(Extensions::EMOJI));
    }

    // ExtensionsDiff tests
    #[test]
    fn test_extensions_diff_new() {
        let diff = ExtensionsDiff::new();
        assert!(diff.is_empty());
        assert!(diff.add.is_empty());
        assert!(diff.remove.is_empty());
    }

    #[test]
    fn test_extensions_diff_add() {
        let diff = ExtensionsDiff::add(Extensions::FOOTNOTES);
        assert!(!diff.is_empty());
        assert!(diff.add.contains(Extensions::FOOTNOTES));
        assert!(diff.remove.is_empty());
    }

    #[test]
    fn test_extensions_diff_remove() {
        let diff = ExtensionsDiff::remove(Extensions::TASKLISTS);
        assert!(!diff.is_empty());
        assert!(diff.remove.contains(Extensions::TASKLISTS));
        assert!(diff.add.is_empty());
    }

    #[test]
    fn test_extensions_diff_apply() {
        let diff = ExtensionsDiff {
            add: Extensions::FOOTNOTES,
            remove: Extensions::TASKLISTS,
        };
        let base = Extensions::TABLES | Extensions::TASKLISTS;
        let result = diff.apply(base);

        assert!(result.contains(Extensions::TABLES));
        assert!(result.contains(Extensions::FOOTNOTES));
        assert!(!result.contains(Extensions::TASKLISTS));
    }

    #[test]
    fn test_extensions_diff_combine() {
        let diff1 = ExtensionsDiff::add(Extensions::FOOTNOTES);
        let diff2 = ExtensionsDiff::remove(Extensions::TASKLISTS);
        let combined = diff1.combine(diff2);

        let base = Extensions::TABLES | Extensions::TASKLISTS;
        let result = combined.apply(base);

        assert!(result.contains(Extensions::TABLES));
        assert!(result.contains(Extensions::FOOTNOTES));
        assert!(!result.contains(Extensions::TASKLISTS));
    }

    #[test]
    fn test_extensions_diff_parse() {
        let diff = ExtensionsDiff::parse("+smart-tasklists+footnotes").unwrap();
        assert!(diff.add.contains(Extensions::SMART_PUNCTUATION));
        assert!(diff.add.contains(Extensions::FOOTNOTES));
        assert!(diff.remove.contains(Extensions::TASKLISTS));
    }

    #[test]
    fn test_extensions_diff_parse_empty() {
        let diff = ExtensionsDiff::parse("").unwrap();
        assert!(diff.is_empty());
    }

    #[test]
    fn test_parse_flavored_format() {
        let (base, diff) = parse_flavored_format("markdown+smart-tasklists").unwrap();

        // Base should be markdown (DOC_EXTENSIONS)
        assert!(base.contains(Extensions::TABLES));

        // Diff should add smart and footnotes, remove tasklists
        assert!(diff.add.contains(Extensions::SMART_PUNCTUATION));
        assert!(diff.remove.contains(Extensions::TASKLISTS));

        // Apply the diff
        let result = diff.apply(base);
        assert!(result.contains(Extensions::SMART_PUNCTUATION));
        assert!(!result.contains(Extensions::TASKLISTS));
    }

    #[test]
    fn test_parse_flavored_format_gfm() {
        let (base, diff) = parse_flavored_format("gfm-hard_line_breaks").unwrap();

        // Base should be GFM
        assert!(base.contains(Extensions::TABLES));
        assert!(base.contains(Extensions::TASKLISTS));

        // Diff should remove hard_line_breaks (which might not be in GFM by default)
        // But the parsing should work
        assert!(!diff.is_empty());
    }
}
