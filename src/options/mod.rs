//! Configuration for the parser and renderer. Extensions affect both.
//!
//! This module provides a comrak-style Options API for configuring
//! Markdown parsing and rendering behavior.
//!
//! # Example
//!
//! ```ignore
//! use clmd::Options;
//!
//! let mut options = Options::default();
//! options.extension.table = true;
//! options.extension.strikethrough = true;
//! options.render.hardbreaks = true;
//! ```

mod extension;
pub mod format;
mod parse;
pub mod plugins;
mod render;
pub mod serde;
mod traits;
mod types;

pub use extension::Extension;
pub use format::{
    Alignment, BlockQuoteMarker, BulletMarker, CodeFenceMarker, DiscretionaryText,
    ElementPlacement, ElementPlacementSort, FormatFlags, FormatOptions, HeadingStyle,
    ListSpacing, NumberedMarker, TrailingMarker,
};
pub use parse::ParseOptions;
pub use plugins::{Plugins, RenderPlugins};
pub use render::RenderOptions;
pub use serde::{
    Config, ExtensionConfig, FormatConfig, ParseConfig, ReaderConfig, RenderConfig,
    SyntaxConfig, TransformConfig, WriterConfig,
};
pub use traits::{
    BrokenLinkCallback, BrokenLinkReference, ResolvedReference, URLRewriter,
};
pub use types::{InputFormat, ListStyleType, OutputFormat, WrapOption};

use arbitrary::Arbitrary;
use bon::Builder;

/// Umbrella options struct for the Markdown parser and renderer.
///
/// This struct provides a convenient way to configure all aspects of
/// Markdown parsing and rendering.
///
/// The lifetime parameter `'c` allows options to hold references to
/// external data such as URL rewriters and broken link callbacks.
///
/// # Example
///
/// ```ignore
/// use clmd::Options;
///
/// let mut options = Options::default();
/// options.extension.table = true;
/// options.extension.strikethrough = true;
///
/// let html = clmd::markdown_to_html("Hello **world**!", &options);
/// assert!(html.contains("<strong>world</strong>"));
/// ```
#[derive(Debug, Clone, Builder, Arbitrary, Default)]
pub struct Options<'c> {
    /// Enable CommonMark extensions.
    pub extension: Extension<'c>,

    /// Configure parse-time options.
    pub parse: ParseOptions<'c>,

    /// Configure render-time options.
    pub render: RenderOptions,

    /// Configure format-time options.
    #[builder(default)]
    pub format: FormatOptions,
}

/// Options for document readers.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReaderOptions {
    /// The input format.
    pub format: InputFormat,
    /// Whether to parse smart punctuation.
    pub smart: bool,
    /// Whether to enable extensions.
    pub extensions: crate::ext::flags::ExtensionFlags,
}

impl ReaderOptions {
    /// Convert to parser Options.
    pub fn to_parser_options(&self) -> Options<'_> {
        let mut options = Options::default();
        options.parse.smart = self.smart;
        options
    }
}

/// Options for document writers.
#[derive(Debug, Clone, Copy, Default)]
pub struct WriterOptions {
    /// The output format.
    pub format: OutputFormat,
    /// Whether to wrap text.
    pub wrap: WrapOption,
    /// The wrap width.
    pub width: usize,
    /// Whether to enable extensions.
    pub extensions: crate::ext::flags::ExtensionFlags,
    /// Whether to output source positions.
    pub output_sourcepos: bool,
}

impl WriterOptions {
    /// Convert to render Options.
    pub fn to_render_options(&self) -> Options<'_> {
        let mut options = Options::default();
        options.render.width = self.width;
        options.render.sourcepos = self.output_sourcepos;
        options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = Options::default();
        assert!(!options.extension.strikethrough);
        assert!(!options.extension.table);
        assert!(!options.parse.smart);
        assert!(!options.render.hardbreaks);
    }

    #[test]
    fn test_options_builder() {
        let options = Options::builder()
            .extension(Extension::default())
            .parse(ParseOptions::default())
            .render(RenderOptions::default())
            .build();
        assert!(!options.extension.table);
    }

    #[test]
    fn test_reader_options_default() {
        let opts = ReaderOptions::default();
        assert_eq!(opts.format, InputFormat::Markdown);
        assert!(!opts.smart);
    }

    #[test]
    fn test_reader_options_to_parser_options() {
        let opts = ReaderOptions::default();
        let _parser_opts = opts.to_parser_options();
    }

    #[test]
    fn test_writer_options_default() {
        let opts = WriterOptions::default();
        assert_eq!(opts.format, OutputFormat::Markdown);
        assert_eq!(opts.width, 0);
        assert!(!opts.output_sourcepos);
    }

    #[test]
    fn test_writer_options_to_render_options() {
        let opts = WriterOptions::default();
        let _render_opts = opts.to_render_options();
    }
}
