//! Predefined configuration options for the Markdown parser and renderer (deprecated).
//!
//! This module provides a comprehensive set of `DataKey` constants for configuring
//! the behavior of the Markdown parser and renderer.
//!
//! # Example
//!
//! ```ignore
//! use clmd::config::{MutableDataSet, DataHolder};
//! use clmd::config::options::{SOURCEPOS, SMART, ENABLE_TABLES};
//!
//! let mut options = MutableDataSet::new();
//! options.set(&SOURCEPOS, true);
//! options.set(&SMART, true);
//! options.set(&ENABLE_TABLES, true);
//!
//! assert_eq!(options.get(&SOURCEPOS), true);
//! assert_eq!(options.get(&SMART), true);
//! assert_eq!(options.get(&ENABLE_TABLES), true);
//! ```

use super::{DataHolder, DataKey};

// =============================================================================
// Parse Options
// =============================================================================

/// Include a `data-sourcepos` attribute on all block elements.
///
/// This option adds source position information to the HTML output,
/// making it easier to map rendered output back to the original Markdown.
///
/// # Example
///
/// ```ignore
/// use clmd::config::options::SOURCEPOS;
///
/// // When enabled, HTML output will include data-sourcepos attributes:
/// // <p data-sourcepos="1:1-1:14">Hello world</p>
/// ```
pub const SOURCEPOS: DataKey<bool> = DataKey::with_default("parse.sourcepos", false);

/// Convert straight quotes to curly, `---` to em dashes, `--` to en dashes.
///
/// This enables "smart" punctuation replacement during parsing.
///
/// # Example
///
/// ```ignore
/// use clmd::config::options::SMART;
///
/// // When enabled:
/// // "Hello" -> "Hello" (curly quotes)
/// // --- -> — (em dash)
/// // -- -> – (en dash)
/// ```
pub const SMART: DataKey<bool> = DataKey::with_default("parse.smart", false);

/// Validate UTF-8 in the input before parsing.
///
/// When enabled, the parser will check that the input is valid UTF-8
/// before processing.
pub const VALIDATE_UTF8: DataKey<bool> =
    DataKey::with_default("parse.validate_utf8", false);

/// Default info string for fenced code blocks.
///
/// This is used when a code block has no info string specified.
pub const DEFAULT_INFO_STRING: DataKey<Option<String>> =
    DataKey::with_default("parse.default_info_string", None);

/// Relax tasklist matching to allow any symbol in brackets.
///
/// When enabled, any character can be used to mark a task as complete,
/// not just 'x' or 'X'.
pub const RELAXED_TASKLIST_MATCHING: DataKey<bool> =
    DataKey::with_default("parse.relaxed_tasklist_matching", false);

/// Ignore setext headings in input.
///
/// When enabled, setext-style headings (underlined with `=` or `-`)
/// will be treated as paragraphs and horizontal rules.
pub const IGNORE_SETEXT: DataKey<bool> =
    DataKey::with_default("parse.ignore_setext", false);

/// Leave footnote definitions in place in the document tree.
///
/// When enabled, footnote definitions are not reordered to the end
/// of the document.
pub const LEAVE_FOOTNOTE_DEFINITIONS: DataKey<bool> =
    DataKey::with_default("parse.leave_footnote_definitions", false);

// =============================================================================
// Render Options
// =============================================================================

/// Render `softbreak` elements as hard line breaks.
///
/// When enabled, soft line breaks in the input will be rendered as
/// `<br />` tags in HTML output.
///
/// # Example
///
/// ```ignore
/// use clmd::config::options::HARDBREAKS;
///
/// // Input: "Hello\nWorld"
/// // Without HARDBREAKS: <p>Hello\nWorld</p>
/// // With HARDBREAKS: <p>Hello<br />\nWorld</p>
/// ```
pub const HARDBREAKS: DataKey<bool> = DataKey::with_default("render.hardbreaks", false);

/// Render `softbreak` elements as spaces.
///
/// When enabled, soft line breaks in the input will be rendered as
/// spaces instead of newlines.
pub const NOBREAKS: DataKey<bool> = DataKey::with_default("render.nobreaks", false);

/// Render raw HTML and potentially dangerous links.
///
/// # Security Warning
///
/// Only enable this option if you trust the input completely.
/// Rendering untrusted user input with this option enabled can
/// lead to XSS (Cross-Site Scripting) attacks.
///
/// # Example
///
/// ```ignore
/// use clmd::config::options::UNSAFE;
///
/// // When enabled, raw HTML like <script> tags will be preserved
/// // instead of being escaped or removed.
/// ```
pub const UNSAFE: DataKey<bool> = DataKey::with_default("render.unsafe", false);

/// Escape raw HTML instead of removing it.
///
/// When enabled, raw HTML will be escaped (e.g., `<` becomes `&lt;`)
/// instead of being removed from the output.
pub const ESCAPE: DataKey<bool> = DataKey::with_default("render.escape", false);

/// GitHub-style `<pre lang="xyz">` for fenced code blocks.
///
/// When enabled, code blocks will use the `lang` attribute on `<pre>`
/// tags instead of the `class` attribute on `<code>` tags.
pub const GITHUB_PRE_LANG: DataKey<bool> =
    DataKey::with_default("render.github_pre_lang", false);

/// Enable full info strings for code blocks.
///
/// When enabled, the full info string (not just the language)
/// is preserved in the output.
pub const FULL_INFO_STRING: DataKey<bool> =
    DataKey::with_default("render.full_info_string", false);

/// The wrap column when outputting CommonMark.
///
/// Text will be wrapped at this column when rendering to CommonMark format.
/// A value of 0 disables wrapping.
pub const WRAP_WIDTH: DataKey<usize> = DataKey::with_default("render.wrap_width", 0);

/// List style type for bullet lists.
///
/// Controls which character is used for bullet lists when rendering
/// to CommonMark format.
pub const LIST_STYLE_TYPE: DataKey<ListStyleType> =
    DataKey::with_default("render.list_style_type", ListStyleType::Dash);

/// Prefer fenced code blocks when outputting CommonMark.
///
/// When enabled, code blocks will use fenced style (```)
/// instead of indented style.
pub const PREFER_FENCED: DataKey<bool> =
    DataKey::with_default("render.prefer_fenced", false);

/// Ignore empty links in input.
///
/// When enabled, empty links like `[]()` will be rendered as text
/// instead of as links.
pub const IGNORE_EMPTY_LINKS: DataKey<bool> =
    DataKey::with_default("render.ignore_empty_links", false);

/// Add classes to tasklist output.
///
/// When enabled, tasklists will include CSS classes for styling.
pub const TASKLIST_CLASSES: DataKey<bool> =
    DataKey::with_default("render.tasklist_classes", false);

/// Compact HTML output (no newlines between block elements).
///
/// When enabled, the HTML output will be more compact with fewer
/// whitespace characters.
pub const COMPACT_HTML: DataKey<bool> =
    DataKey::with_default("render.compact_html", false);

// =============================================================================
// Extension Options
// =============================================================================

/// Enable table support (GFM extension).
///
/// Allows parsing of GitHub Flavored Markdown tables.
///
/// # Example
///
/// ```markdown
/// | Header 1 | Header 2 |
/// |----------|----------|
/// | Cell 1   | Cell 2   |
/// ```
pub const ENABLE_TABLES: DataKey<bool> =
    DataKey::with_default("extension.tables", false);

/// Enable footnote support.
///
/// Allows parsing of footnote references and definitions.
///
/// # Example
///
/// ```markdown
/// This is a sentence with a footnote[^1].
///
/// [^1]: This is the footnote text.
/// ```
pub const ENABLE_FOOTNOTES: DataKey<bool> =
    DataKey::with_default("extension.footnotes", false);

/// Enable strikethrough support (GFM extension).
///
/// Allows parsing of strikethrough text using `~~` delimiters.
///
/// # Example
///
/// ```markdown
/// This is ~~deleted~~ text.
/// ```
pub const ENABLE_STRIKETHROUGH: DataKey<bool> =
    DataKey::with_default("extension.strikethrough", false);

/// Enable task list support (GFM extension).
///
/// Allows parsing of task list items with checkboxes.
///
/// # Example
///
/// ```markdown
/// - [x] Completed task
/// - [ ] Incomplete task
/// ```
pub const ENABLE_TASKLISTS: DataKey<bool> =
    DataKey::with_default("extension.tasklists", false);

/// Enable autolink support (GFM extension).
///
/// Automatically converts URLs and email addresses to links.
pub const ENABLE_AUTOLINKS: DataKey<bool> =
    DataKey::with_default("extension.autolinks", false);

/// Enable tag filter (GFM extension).
///
/// Filters out certain HTML tags for security.
pub const ENABLE_TAGFILTER: DataKey<bool> =
    DataKey::with_default("extension.tagfilter", false);

/// Enable superscript support.
///
/// Allows parsing of superscript text using `^` delimiters.
///
/// # Example
///
/// ```markdown
/// E = mc^2^
/// ```
pub const ENABLE_SUPERSCRIPT: DataKey<bool> =
    DataKey::with_default("extension.superscript", false);

/// Enable subscript support.
///
/// Allows parsing of subscript text using `~` delimiters.
/// Note: This conflicts with strikethrough if both are enabled.
///
/// # Example
///
/// ```markdown
/// H~2~O
/// ```
pub const ENABLE_SUBSCRIPT: DataKey<bool> =
    DataKey::with_default("extension.subscript", false);

/// Enable header IDs.
///
/// When set to Some(prefix), adds IDs to headers based on their content.
/// The prefix is prepended to the generated ID.
pub const HEADER_IDS: DataKey<Option<String>> =
    DataKey::with_default("extension.header_ids", None);

/// Enable description lists.
///
/// Allows parsing of definition/description lists.
///
/// # Example
///
/// ```markdown
/// Term
/// : Definition
/// ```
pub const ENABLE_DESCRIPTION_LISTS: DataKey<bool> =
    DataKey::with_default("extension.description_lists", false);

/// Enable front matter support.
///
/// When set to Some(delimiter), allows YAML front matter at the
/// beginning of the document.
pub const FRONT_MATTER_DELIMITER: DataKey<Option<String>> =
    DataKey::with_default("extension.front_matter_delimiter", None);

/// Enable multiline block quotes.
///
/// Allows block quotes that span multiple lines using `>>>` delimiters.
pub const ENABLE_MULTILINE_BLOCK_QUOTES: DataKey<bool> =
    DataKey::with_default("extension.multiline_block_quotes", false);

/// Enable GitHub-style alerts.
///
/// Allows parsing of GitHub-style alert boxes.
///
/// # Example
///
/// ```markdown
/// > [!NOTE]
/// > This is a note.
/// ```
pub const ENABLE_ALERTS: DataKey<bool> =
    DataKey::with_default("extension.alerts", false);

/// Enable math support using dollar syntax.
///
/// Allows parsing of math expressions using `$` and `$$` delimiters.
pub const ENABLE_MATH_DOLLARS: DataKey<bool> =
    DataKey::with_default("extension.math_dollars", false);

/// Enable math support using code syntax.
///
/// Allows parsing of math expressions using `$`code`` and `math` code blocks.
pub const ENABLE_MATH_CODE: DataKey<bool> =
    DataKey::with_default("extension.math_code", false);

/// Enable wikilinks with title after pipe.
///
/// Allows parsing of wikilinks like `[[url|title]]`.
pub const ENABLE_WIKILINKS_TITLE_AFTER_PIPE: DataKey<bool> =
    DataKey::with_default("extension.wikilinks_title_after_pipe", false);

/// Enable wikilinks with title before pipe.
///
/// Allows parsing of wikilinks like `[[title|url]]`.
pub const ENABLE_WIKILINKS_TITLE_BEFORE_PIPE: DataKey<bool> =
    DataKey::with_default("extension.wikilinks_title_before_pipe", false);

/// Enable underline support.
///
/// Allows parsing of underlined text using `__` delimiters.
pub const ENABLE_UNDERLINE: DataKey<bool> =
    DataKey::with_default("extension.underline", false);

/// Enable spoiler support.
///
/// Allows parsing of spoiler text using `||` delimiters.
pub const ENABLE_SPOILER: DataKey<bool> =
    DataKey::with_default("extension.spoiler", false);

/// Enable greentext support.
///
/// Requires a space after `>` for blockquotes.
pub const ENABLE_GREENTEXT: DataKey<bool> =
    DataKey::with_default("extension.greentext", false);

/// Enable highlight/mark support.
///
/// Allows parsing of highlighted text using `==` delimiters.
pub const ENABLE_HIGHLIGHT: DataKey<bool> =
    DataKey::with_default("extension.highlight", false);

/// Enable insert support.
///
/// Allows parsing of inserted text using `++` delimiters.
pub const ENABLE_INSERT: DataKey<bool> =
    DataKey::with_default("extension.insert", false);

/// Enable CJK-friendly emphasis.
///
/// Recognizes emphasis patterns that are common in CJK text.
pub const ENABLE_CJK_FRIENDLY_EMPHASIS: DataKey<bool> =
    DataKey::with_default("extension.cjk_friendly_emphasis", false);

/// Enable subtext support.
///
/// Allows parsing of subtext using `-#` syntax.
pub const ENABLE_SUBTEXT: DataKey<bool> =
    DataKey::with_default("extension.subtext", false);

// =============================================================================
// Additional Parse Options
// =============================================================================

/// Whether tasklist items can be parsed in table cells.
///
/// At present, the tasklist item must be the only content in the cell.
/// Both tables and tasklists must be enabled for this to work.
pub const TASKLIST_IN_TABLE: DataKey<bool> =
    DataKey::with_default("parse.tasklist_in_table", false);

/// Relax parsing of autolinks.
///
/// Allows links to be detected inside brackets and allow all URL schemes.
pub const RELAXED_AUTOLINKS: DataKey<bool> =
    DataKey::with_default("parse.relaxed_autolinks", false);

/// Leave escaped characters in an `Escaped` node in the document tree.
pub const ESCAPED_CHAR_SPANS: DataKey<bool> =
    DataKey::with_default("parse.escaped_char_spans", false);

// =============================================================================
// Additional Render Options
// =============================================================================

/// Include source position attributes in HTML and XML output.
///
/// Sourcepos information is reliable for core block items excluding
/// lists and list items, all inlines, and most extensions.
pub const SOURCEPOS_RENDER: DataKey<bool> =
    DataKey::with_default("render.sourcepos", false);

/// Enables GFM quirks in HTML output which break CommonMark compatibility.
///
/// This changes how nested emphasis is rendered to match GitHub's behavior.
pub const GFM_QUIRKS: DataKey<bool> = DataKey::with_default("render.gfm_quirks", false);

/// Render the image as a figure element with the title as its caption.
pub const FIGURE_WITH_CAPTION: DataKey<bool> =
    DataKey::with_default("render.figure_with_caption", false);

/// Render ordered list with a minimum marker width.
///
/// Having a width lower than 3 doesn't do anything.
pub const OL_WIDTH: DataKey<usize> = DataKey::with_default("render.ol_width", 0);

/// Wrap escaped characters in a `<span>` to allow any
/// post-processing to recognize them.
pub const ESCAPED_CHAR_SPANS_RENDER: DataKey<bool> =
    DataKey::with_default("render.escaped_char_spans", false);

// =============================================================================
// Types
// =============================================================================

/// Style type for bullet lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListStyleType {
    /// Use `-` for bullet lists.
    #[default]
    Dash,
    /// Use `+` for bullet lists.
    Plus,
    /// Use `*` for bullet lists.
    Star,
}

/// A comprehensive options container for parsing and rendering.
///
/// This struct provides a convenient way to manage all options
/// using the DataKey system.
///
/// # Example
///
/// ```ignore
/// use clmd::config::options::Options;
/// use clmd::config::options::{SMART, ENABLE_TABLES};
///
/// let mut options = Options::new();
/// options.set(&SMART, true);
/// options.set(&ENABLE_TABLES, true);
///
/// assert_eq!(options.get(&SMART), true);
/// assert_eq!(options.get(&ENABLE_TABLES), true);
/// ```
#[derive(Debug, Default)]
pub struct Options {
    data: super::MutableDataSet,
}

impl Options {
    /// Create a new options container with default values.
    pub fn new() -> Self {
        Self {
            data: super::MutableDataSet::new(),
        }
    }

    /// Get a value by key.
    ///
    /// Returns the default value if the key is not explicitly set.
    pub fn get<T: Clone + 'static>(&self, key: &DataKey<T>) -> T {
        self.data.get(key)
    }

    /// Set a value for a key.
    pub fn set<T: Clone + 'static>(&mut self, key: &DataKey<T>, value: T) {
        self.data.set(key, value);
    }

    /// Check if a key has been explicitly set.
    pub fn contains<T: Clone + 'static>(&self, key: &DataKey<T>) -> bool {
        self.data.contains(key)
    }

    /// Get the underlying data set.
    pub fn data(&self) -> &super::MutableDataSet {
        &self.data
    }

    /// Get a mutable reference to the underlying data set.
    pub fn data_mut(&mut self) -> &mut super::MutableDataSet {
        &mut self.data
    }
}

/// Parse-specific options.
///
/// This is a type-safe wrapper around Options for parse-time configuration.
#[derive(Debug, Default)]
pub struct ParseOptions {
    inner: Options,
}

impl ParseOptions {
    /// Create new parse options with defaults.
    pub fn new() -> Self {
        Self {
            inner: Options::new(),
        }
    }

    /// Get a value by key.
    pub fn get<T: Clone + 'static>(&self, key: &DataKey<T>) -> T {
        self.inner.get(key)
    }

    /// Set a value for a key.
    pub fn set<T: Clone + 'static>(&mut self, key: &DataKey<T>, value: T) {
        self.inner.set(key, value);
    }

    /// Convert to general Options.
    pub fn into_options(self) -> Options {
        self.inner
    }

    /// Get the underlying options.
    pub fn options(&self) -> &Options {
        &self.inner
    }
}

/// Render-specific options.
///
/// This is a type-safe wrapper around Options for render-time configuration.
#[derive(Debug, Default)]
pub struct RenderOptions {
    inner: Options,
}

impl RenderOptions {
    /// Create new render options with defaults.
    pub fn new() -> Self {
        Self {
            inner: Options::new(),
        }
    }

    /// Get a value by key.
    pub fn get<T: Clone + 'static>(&self, key: &DataKey<T>) -> T {
        self.inner.get(key)
    }

    /// Set a value for a key.
    pub fn set<T: Clone + 'static>(&mut self, key: &DataKey<T>, value: T) {
        self.inner.set(key, value);
    }

    /// Convert to general Options.
    pub fn into_options(self) -> Options {
        self.inner
    }

    /// Get the underlying options.
    pub fn options(&self) -> &Options {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = Options::new();
        assert!(!options.get(&SOURCEPOS));
        assert!(!options.get(&SMART));
        assert!(!options.get(&HARDBREAKS));
        assert!(!options.get(&ENABLE_TABLES));
    }

    #[test]
    fn test_set_and_get() {
        let mut options = Options::new();
        options.set(&SOURCEPOS, true);
        options.set(&SMART, true);
        options.set(&ENABLE_TABLES, true);

        assert!(options.get(&SOURCEPOS));
        assert!(options.get(&SMART));
        assert!(options.get(&ENABLE_TABLES));
    }

    #[test]
    fn test_contains() {
        let mut options = Options::new();
        assert!(!options.contains(&SOURCEPOS));

        options.set(&SOURCEPOS, true);
        assert!(options.contains(&SOURCEPOS));
    }

    #[test]
    fn test_parse_options() {
        let mut options = ParseOptions::new();
        options.set(&SOURCEPOS, true);
        options.set(&SMART, true);

        assert!(options.get(&SOURCEPOS));
        assert!(options.get(&SMART));
    }

    #[test]
    fn test_render_options() {
        let mut options = RenderOptions::new();
        options.set(&HARDBREAKS, true);
        options.set(&UNSAFE, false);

        assert!(options.get(&HARDBREAKS));
        assert!(!options.get(&UNSAFE));
    }

    #[test]
    fn test_list_style_type() {
        let mut options = Options::new();
        assert_eq!(options.get(&LIST_STYLE_TYPE), ListStyleType::Dash);

        options.set(&LIST_STYLE_TYPE, ListStyleType::Plus);
        assert_eq!(options.get(&LIST_STYLE_TYPE), ListStyleType::Plus);
    }

    #[test]
    fn test_option_groups() {
        // Test that we can organize options logically
        let mut options = Options::new();

        // Parse options
        options.set(&SOURCEPOS, true);
        options.set(&SMART, true);

        // Render options
        options.set(&HARDBREAKS, true);
        options.set(&GITHUB_PRE_LANG, true);

        // Extension options
        options.set(&ENABLE_TABLES, true);
        options.set(&ENABLE_STRIKETHROUGH, true);

        assert!(options.get(&SOURCEPOS));
        assert!(options.get(&HARDBREAKS));
        assert!(options.get(&ENABLE_TABLES));
    }
}
