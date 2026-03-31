//! Document conversion pipeline for clmd.
//!
//! This module provides a flexible pipeline system for document conversion,
//! inspired by Pandoc's conversion architecture.
//!
//! # Example
//!
//! ```
//! use clmd::pipeline::{Pipeline, PipelineBuilder};
//! use clmd::Options;
//!
//! let pipeline = PipelineBuilder::new()
//!     .from("markdown")
//!     .to("html")
//!     .build()
//!     .unwrap();
//!
//! let output = pipeline.convert("# Hello World", &Options::default()).unwrap();
//! assert!(output.contains("<h1>"));
//! ```

use crate::error::{ClmdError, ClmdResult, Position};
use crate::filter::{Filter, FilterChain};
use crate::options::Options;
use crate::readers::{Reader, ReaderInput, ReaderOptions, ReaderRegistry};
use crate::writers::{Writer, WriterOptions, WriterRegistry};

/// A document conversion pipeline.
///
/// The pipeline orchestrates the conversion process from input to output,
/// applying any configured transforms along the way.
pub struct Pipeline {
    /// Input format.
    input_format: String,
    /// Output format.
    output_format: String,
    /// Reader for the input format.
    reader: Box<dyn Reader>,
    /// Writer for the output format.
    writer: Box<dyn Writer>,
    /// Filter chain for document transformation.
    filter_chain: FilterChain,
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("input_format", &self.input_format)
            .field("output_format", &self.output_format)
            .finish_non_exhaustive()
    }
}

impl Pipeline {
    /// Create a new pipeline with the given reader and writer.
    pub fn new(
        input_format: impl Into<String>,
        output_format: impl Into<String>,
        reader: Box<dyn Reader>,
        writer: Box<dyn Writer>,
    ) -> Self {
        Self {
            input_format: input_format.into(),
            output_format: output_format.into(),
            reader,
            writer,
            filter_chain: FilterChain::new(),
        }
    }

    /// Create a new pipeline with a filter chain.
    pub fn with_filters(
        input_format: impl Into<String>,
        output_format: impl Into<String>,
        reader: Box<dyn Reader>,
        writer: Box<dyn Writer>,
        filter_chain: FilterChain,
    ) -> Self {
        Self {
            input_format: input_format.into(),
            output_format: output_format.into(),
            reader,
            writer,
            filter_chain,
        }
    }

    /// Convert input to output.
    ///
    /// # Arguments
    ///
    /// * `input` - The input text to convert
    /// * `options` - Conversion options
    ///
    /// # Returns
    ///
    /// The converted output string, or an error if conversion fails.
    pub fn convert(&self, input: &str, _options: &Options) -> ClmdResult<String> {
        // Step 1: Read the input
        let reader_options = ReaderOptions::default();
        let reader_input = ReaderInput::text(input);
        let (mut arena, root) = self
            .reader
            .read(&reader_input, &reader_options)
            .map_err(|e| {
                ClmdError::parse_error(Position::start(), format!("Read error: {}", e))
            })?;

        // Step 2: Apply filters
        if !self.filter_chain.is_empty() {
            self.filter_chain
                .apply(&mut arena, root)
                .map_err(|e| ClmdError::filter_error(format!("Filter error: {}", e)))?;
        }

        // Step 3: Write the output
        let writer_options = WriterOptions::default();
        self.writer
            .write_text(&arena, root, &writer_options)
            .map_err(|e| ClmdError::io_error(format!("Write error: {}", e)))
    }

    /// Get the filter chain.
    pub fn filter_chain(&self) -> &FilterChain {
        &self.filter_chain
    }

    /// Add a filter to the pipeline.
    pub fn add_filter(&mut self, filter: Filter) -> &mut Self {
        self.filter_chain.add(filter);
        self
    }

    /// Get the input format.
    pub fn input_format(&self) -> &str {
        &self.input_format
    }

    /// Get the output format.
    pub fn output_format(&self) -> &str {
        &self.output_format
    }
}

/// Builder for creating pipelines.
pub struct PipelineBuilder {
    input_format: Option<String>,
    output_format: Option<String>,
    reader_registry: ReaderRegistry,
    writer_registry: WriterRegistry,
    filter_chain: FilterChain,
}

impl std::fmt::Debug for PipelineBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineBuilder")
            .field("input_format", &self.input_format)
            .field("output_format", &self.output_format)
            .finish_non_exhaustive()
    }
}

impl PipelineBuilder {
    /// Create a new pipeline builder.
    pub fn new() -> Self {
        Self {
            input_format: None,
            output_format: None,
            reader_registry: ReaderRegistry::with_defaults(),
            writer_registry: WriterRegistry::with_defaults(),
            filter_chain: FilterChain::new(),
        }
    }

    /// Set the input format.
    pub fn from(mut self, format: impl Into<String>) -> Self {
        self.input_format = Some(format.into());
        self
    }

    /// Set the output format.
    pub fn to(mut self, format: impl Into<String>) -> Self {
        self.output_format = Some(format.into());
        self
    }

    /// Add a filter to the pipeline.
    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filter_chain.add(filter);
        self
    }

    /// Add multiple filters to the pipeline.
    pub fn with_filters(mut self, filters: impl IntoIterator<Item = Filter>) -> Self {
        self.filter_chain.extend(filters);
        self
    }

    /// Set the filter chain.
    pub fn with_filter_chain(mut self, filter_chain: FilterChain) -> Self {
        self.filter_chain = filter_chain;
        self
    }

    /// Build the pipeline.
    pub fn build(self) -> ClmdResult<Pipeline> {
        let input_format = self
            .input_format
            .ok_or_else(|| ClmdError::config_error("Input format not specified"))?;

        let output_format = self
            .output_format
            .ok_or_else(|| ClmdError::config_error("Output format not specified"))?;

        // Verify formats are available
        let _reader = self
            .reader_registry
            .get(&input_format)
            .ok_or_else(|| ClmdError::unknown_reader(&input_format))?;

        let _writer = self
            .writer_registry
            .get(&output_format)
            .ok_or_else(|| ClmdError::unknown_writer(&output_format))?;

        // Create boxed versions
        let boxed_reader = create_reader(&input_format)?;
        let boxed_writer = create_writer(&output_format)?;

        Ok(Pipeline {
            input_format,
            output_format,
            reader: boxed_reader,
            writer: boxed_writer,
            filter_chain: self.filter_chain,
        })
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a boxed reader by format name.
fn create_reader(format: &str) -> ClmdResult<Box<dyn Reader>> {
    use crate::readers::registry::{CommonMarkReader, HtmlReader, MarkdownReader};

    match format.to_lowercase().as_str() {
        "markdown" => Ok(Box::new(MarkdownReader::new())),
        "html" => Ok(Box::new(HtmlReader::new())),
        "commonmark" => Ok(Box::new(CommonMarkReader::new())),
        _ => Err(ClmdError::unknown_reader(format)),
    }
}

/// Create a boxed writer by format name.
fn create_writer(format: &str) -> ClmdResult<Box<dyn Writer>> {
    use crate::writers::registry::{CommonMarkWriter, HtmlWriter, XmlWriter};

    match format.to_lowercase().as_str() {
        "html" => Ok(Box::new(HtmlWriter::new())),
        "xml" => Ok(Box::new(XmlWriter::new())),
        "commonmark" | "markdown" => Ok(Box::new(CommonMarkWriter::new())),
        _ => Err(ClmdError::unknown_writer(format)),
    }
}

/// Convenience function to convert between formats.
///
/// # Arguments
///
/// * `input` - The input text
/// * `from` - The input format
/// * `to` - The output format
/// * `options` - Conversion options
///
/// # Returns
///
/// The converted output string, or an error if conversion fails.
pub fn convert(
    input: &str,
    from: impl Into<String>,
    to: impl Into<String>,
    options: &Options,
) -> ClmdResult<String> {
    let pipeline = PipelineBuilder::new().from(from).to(to).build()?;

    pipeline.convert(input, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_builder() {
        let pipeline = PipelineBuilder::new().from("markdown").to("html").build();

        assert!(pipeline.is_ok());
        let pipeline = pipeline.unwrap();
        assert_eq!(pipeline.input_format(), "markdown");
        assert_eq!(pipeline.output_format(), "html");
    }

    #[test]
    fn test_pipeline_builder_missing_input() {
        let pipeline = PipelineBuilder::new().to("html").build();
        assert!(pipeline.is_err());
    }

    #[test]
    fn test_pipeline_builder_missing_output() {
        let pipeline = PipelineBuilder::new().from("markdown").build();
        assert!(pipeline.is_err());
    }

    #[test]
    fn test_pipeline_convert() {
        let pipeline = PipelineBuilder::new()
            .from("markdown")
            .to("html")
            .build()
            .unwrap();

        let output = pipeline.convert("# Hello", &Options::default()).unwrap();
        assert!(output.contains("<h1>"));
    }

    #[test]
    fn test_convert_function() {
        let output =
            convert("# Hello", "markdown", "html", &Options::default()).unwrap();
        assert!(output.contains("<h1>"));
    }
}
