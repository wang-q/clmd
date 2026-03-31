//! Document conversion pipeline for clmd.
//!
//! This module provides a flexible pipeline system for document conversion,
//! inspired by Pandoc's conversion architecture. The pipeline consists of:
//!
//! 1. **Reader** - Parses input into the document AST
//! 2. **Transforms** - Applies transformations to the document
//! 3. **Writer** - Renders the document to the output format
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

use crate::arena::{NodeArena, NodeId};
use crate::error::{ClmdError, ClmdResult, Position};
use crate::extensions::{ExtensionConfig, ExtensionRegistry, Extensions};
use crate::options::Options;
use crate::readers::{Document, Reader, ReaderRegistry};
use crate::writers::{Writer, WriterError, WriterRegistry};
use std::collections::HashMap;

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
    /// Writers for the output format.
    writer: Box<dyn Writer>,
    /// Transforms to apply.
    transforms: Vec<Box<dyn Transform>>,
    /// Extensions to enable.
    extensions: Extensions,
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("input_format", &self.input_format)
            .field("output_format", &self.output_format)
            .field("extensions", &self.extensions)
            .field("transforms_count", &self.transforms.len())
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
            transforms: Vec::new(),
            extensions: Extensions::empty(),
        }
    }

    /// Add a transform to the pipeline.
    pub fn add_transform(&mut self, transform: Box<dyn Transform>) {
        self.transforms.push(transform);
    }

    /// Set extensions for the pipeline.
    pub fn with_extensions(mut self, extensions: Extensions) -> Self {
        self.extensions = extensions;
        self
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
    pub fn convert(&self, input: &str, options: &Options) -> ClmdResult<String> {
        // Step 1: Read the input
        let mut doc = self.reader.read(input, options).map_err(|e| {
            ClmdError::parse_error(Position::start(), format!("Read error: {}", e))
        })?;

        // Step 2: Apply transforms
        for transform in &self.transforms {
            transform.transform(&mut doc).map_err(|e| {
                ClmdError::transform_error(format!("Transform error: {}", e))
            })?;
        }

        // Step 3: Write the output
        self.writer
            .write(&doc.arena, doc.root, options)
            .map_err(|e| ClmdError::io_error(format!("Write error: {}", e)))
    }

    /// Convert with a custom arena (for advanced use cases).
    pub fn convert_with_arena(
        &self,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> ClmdResult<String> {
        self.writer
            .write(arena, root, options)
            .map_err(|e| ClmdError::io_error(format!("Write error: {}", e)))
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

/// Trait for document transforms.
///
/// Implement this trait to create custom document transformations.
pub trait Transform {
    /// Transform the document.
    ///
    /// # Arguments
    ///
    /// * `doc` - The document to transform
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, or an error if transformation fails.
    fn transform(&self, doc: &mut Document) -> TransformResult<()>;

    /// Get the name of this transform.
    fn name(&self) -> &'static str;
}

/// Result type for transform operations.
pub type TransformResult<T> = Result<T, TransformError>;

/// Error type for transform operations.
#[derive(Debug, Clone)]
pub enum TransformError {
    /// Generic error message.
    Message(String),
    /// Invalid document structure.
    InvalidStructure(String),
    /// Missing required element.
    MissingElement(String),
}

impl TransformError {
    /// Create a new transform error.
    pub fn new<S: Into<String>>(msg: S) -> Self {
        Self::Message(msg.into())
    }

    /// Create an invalid structure error.
    pub fn invalid_structure<S: Into<String>>(msg: S) -> Self {
        Self::InvalidStructure(msg.into())
    }

    /// Create a missing element error.
    pub fn missing_element<S: Into<String>>(msg: S) -> Self {
        Self::MissingElement(msg.into())
    }
}

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(msg) => write!(f, "{}", msg),
            Self::InvalidStructure(msg) => write!(f, "Invalid structure: {}", msg),
            Self::MissingElement(msg) => write!(f, "Missing element: {}", msg),
        }
    }
}

impl std::error::Error for TransformError {}

/// Header shift transform - adjusts header levels.
#[derive(Debug, Clone, Copy)]
pub struct HeaderShiftTransform {
    /// Amount to shift (positive = increase level, negative = decrease).
    shift: i32,
}

impl HeaderShiftTransform {
    /// Create a new header shift transform.
    pub fn new(shift: i32) -> Self {
        Self { shift }
    }
}

impl Transform for HeaderShiftTransform {
    fn transform(&self, doc: &mut Document) -> TransformResult<()> {
        use crate::nodes::NodeValue;

        // Iterate through all nodes and shift headers
        for (_node_id, node) in doc.arena.iter_mut() {
            if let NodeValue::Heading(ref mut heading) = node.value {
                let new_level = (heading.level as i32 + self.shift).clamp(1, 6) as u8;
                heading.level = new_level;
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "header_shift"
    }
}

/// Link fixup transform - fixes internal links.
#[derive(Debug, Clone)]
pub struct LinkFixupTransform {
    /// Base URL for relative links.
    base_url: String,
}

impl LinkFixupTransform {
    /// Create a new link fixup transform.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}

impl Transform for LinkFixupTransform {
    fn transform(&self, doc: &mut Document) -> TransformResult<()> {
        use crate::nodes::NodeValue;

        for (_node_id, node) in doc.arena.iter_mut() {
            if let NodeValue::Link(ref mut link) = node.value {
                if link.url.starts_with("./") || link.url.starts_with("../") {
                    link.url = format!("{}{}", self.base_url, link.url);
                }
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "link_fixup"
    }
}

/// Builder for creating pipelines.
pub struct PipelineBuilder {
    input_format: Option<String>,
    output_format: Option<String>,
    transforms: Vec<Box<dyn Transform>>,
    extensions: Extensions,
    reader_registry: ReaderRegistry,
    writer_registry: WriterRegistry,
}

impl std::fmt::Debug for PipelineBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineBuilder")
            .field("input_format", &self.input_format)
            .field("output_format", &self.output_format)
            .field("extensions", &self.extensions)
            .field("transforms_count", &self.transforms.len())
            .finish_non_exhaustive()
    }
}

impl PipelineBuilder {
    /// Create a new pipeline builder.
    pub fn new() -> Self {
        Self {
            input_format: None,
            output_format: None,
            transforms: Vec::new(),
            extensions: Extensions::empty(),
            reader_registry: ReaderRegistry::new(),
            writer_registry: WriterRegistry::new(),
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

    /// Add a transform.
    pub fn with_transform(mut self, transform: Box<dyn Transform>) -> Self {
        self.transforms.push(transform);
        self
    }

    /// Add multiple transforms.
    pub fn with_transforms(mut self, transforms: Vec<Box<dyn Transform>>) -> Self {
        self.transforms.extend(transforms);
        self
    }

    /// Set extensions.
    pub fn with_extensions(mut self, extensions: Extensions) -> Self {
        self.extensions = extensions;
        self
    }

    /// Enable an extension.
    pub fn enable_extension(mut self, extension: Extensions) -> Self {
        self.extensions |= extension;
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

        let reader = self
            .reader_registry
            .get(&input_format)
            .ok_or_else(|| ClmdError::unknown_reader(&input_format))?
            .as_reader();

        let writer = self
            .writer_registry
            .get(&output_format)
            .ok_or_else(|| ClmdError::unknown_writer(&output_format))?
            .as_writer();

        Ok(Pipeline {
            input_format,
            output_format,
            reader,
            writer,
            transforms: self.transforms,
            extensions: self.extensions,
        })
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for getting boxed readers/writers.
pub trait AsBoxed {
    /// Convert to a boxed reader.
    fn as_reader(self) -> Box<dyn Reader>;
    /// Convert to a boxed writer.
    fn as_writer(self) -> Box<dyn Writer>;
}

impl AsBoxed for &dyn Reader {
    fn as_reader(self) -> Box<dyn Reader> {
        // This is a workaround - in practice, you'd clone or recreate the reader
        // For now, we'll use a factory pattern
        match self.format_name() {
            "markdown" => Box::new(crate::readers::MarkdownReader::new()),
            "html" => Box::new(crate::readers::HtmlReader::new()),
            _ => panic!("Unknown reader format"),
        }
    }

    fn as_writer(self) -> Box<dyn Writer> {
        panic!("Cannot convert reader to writer");
    }
}

impl AsBoxed for &dyn Writer {
    fn as_reader(self) -> Box<dyn Reader> {
        panic!("Cannot convert writer to reader");
    }

    fn as_writer(self) -> Box<dyn Writer> {
        match self.format_name() {
            "html" => Box::new(crate::writers::HtmlWriter::new()),
            "xml" => Box::new(crate::writers::XmlWriter::new()),
            "commonmark" => Box::new(crate::writers::CommonMarkWriter::new()),
            _ => panic!("Unknown writer format"),
        }
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

    #[test]
    fn test_header_shift_transform() {
        let transform = HeaderShiftTransform::new(1);
        assert_eq!(transform.name(), "header_shift");
    }

    #[test]
    fn test_link_fixup_transform() {
        let transform = LinkFixupTransform::new("https://example.com/");
        assert_eq!(transform.name(), "link_fixup");
    }

    #[test]
    fn test_transform_error() {
        let err = TransformError::new("test error");
        assert_eq!(err.to_string(), "test error");

        let err = TransformError::invalid_structure("bad structure");
        assert!(err.to_string().contains("Invalid structure"));

        let err = TransformError::missing_element("missing");
        assert!(err.to_string().contains("Missing element"));
    }

    #[test]
    fn test_pipeline_with_extensions() {
        use crate::extensions::Extensions;

        let pipeline = PipelineBuilder::new()
            .from("markdown")
            .to("html")
            .enable_extension(Extensions::TABLE)
            .enable_extension(Extensions::STRIKETHROUGH)
            .build()
            .unwrap();

        // The extensions should be stored in the pipeline
        // (actual usage depends on reader/writer implementation)
        assert_eq!(pipeline.input_format(), "markdown");
    }
}
