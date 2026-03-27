//! Unified renderer trait and implementations
//!
//! This module provides a common trait for all renderers, allowing for
//! consistent rendering interfaces across different output formats.
//!
//! # Example
//!
//! ```
//! use clmd::render::renderer::{Renderer, HtmlRenderer};
//! use clmd::arena::NodeArena;
//! use clmd::blocks::BlockParser;
//!
//! let mut arena = NodeArena::new();
//! let doc = BlockParser::parse(&mut arena, "# Hello\n\nWorld");
//!
//! let renderer = HtmlRenderer::new();
//! let html = renderer.render(&arena, doc);
//! assert!(html.contains("<h1>Hello</h1>"));
//! ```

use crate::arena::{NodeArena, NodeId};
use crate::config::options::Options;
use std::fmt::Write;

/// A unified trait for rendering AST to various output formats.
///
/// This trait provides a common interface for all renderers, making it
/// easy to switch between output formats or implement custom renderers.
///
/// # Example
///
/// ```
/// use clmd::render::renderer::Renderer;
/// use clmd::arena::{NodeArena, NodeId};
/// use clmd::blocks::BlockParser;
///
/// fn render_with_any_renderer<R: Renderer>(renderer: &R, arena: &NodeArena, root: NodeId) -> String {
///     renderer.render(arena, root)
/// }
/// ```
pub trait Renderer {
    /// Render the AST starting from the given root node.
    ///
    /// # Arguments
    ///
    /// * `arena` - The node arena containing the AST
    /// * `root` - The root node ID to start rendering from
    ///
    /// # Returns
    ///
    /// The rendered output as a String
    fn render(&self, arena: &NodeArena, root: NodeId) -> String;

    /// Render the AST with options.
    ///
    /// # Arguments
    ///
    /// * `arena` - The node arena containing the AST
    /// * `root` - The root node ID to start rendering from
    /// * `options` - Rendering options
    ///
    /// # Returns
    ///
    /// The rendered output as a String
    fn render_with_options(&self, arena: &NodeArena, root: NodeId, options: &Options) -> String;

    /// Get the name of this renderer.
    fn name(&self) -> &'static str;

    /// Get the MIME type of the output format.
    fn mime_type(&self) -> &'static str;
}

/// HTML renderer implementing the Renderer trait.
///
/// This renderer generates HTML output from the AST.
///
/// # Example
///
/// ```
/// use clmd::render::renderer::{Renderer, HtmlRenderer};
/// use clmd::arena::NodeArena;
/// use clmd::blocks::BlockParser;
///
/// let mut arena = NodeArena::new();
/// let doc = BlockParser::parse(&mut arena, "# Hello\n\nWorld");
///
/// let renderer = HtmlRenderer::new();
/// let html = renderer.render(&arena, doc);
/// assert!(html.contains("<h1>Hello</h1>"));
/// ```
#[derive(Debug, Default, Clone)]
pub struct HtmlRenderer;

impl HtmlRenderer {
    /// Create a new HTML renderer.
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for HtmlRenderer {
    fn render(&self, arena: &NodeArena, root: NodeId) -> String {
        let options = Options::new();
        self.render_with_options(arena, root, &options)
    }

    fn render_with_options(&self, arena: &NodeArena, root: NodeId, _options: &Options) -> String {
        crate::render::html::render(arena, root, crate::options::DEFAULT)
    }

    fn name(&self) -> &'static str {
        "HTML"
    }

    fn mime_type(&self) -> &'static str {
        "text/html"
    }
}

/// CommonMark renderer implementing the Renderer trait.
///
/// This renderer generates CommonMark (Markdown) output from the AST,
/// useful for round-trip conversion.
///
/// # Example
///
/// ```
/// use clmd::render::renderer::{Renderer, CommonMarkRenderer};
/// use clmd::arena::NodeArena;
/// use clmd::blocks::BlockParser;
///
/// let mut arena = NodeArena::new();
/// let doc = BlockParser::parse(&mut arena, "# Hello\n\nWorld");
///
/// let renderer = CommonMarkRenderer::new();
/// let cm = renderer.render(&arena, doc);
/// assert!(cm.contains("# Hello"));
/// ```
#[derive(Debug, Default, Clone)]
pub struct CommonMarkRenderer;

impl CommonMarkRenderer {
    /// Create a new CommonMark renderer.
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for CommonMarkRenderer {
    fn render(&self, arena: &NodeArena, root: NodeId) -> String {
        let options = Options::new();
        self.render_with_options(arena, root, &options)
    }

    fn render_with_options(&self, arena: &NodeArena, root: NodeId, _options: &Options) -> String {
        crate::render::commonmark::render(arena, root, crate::options::DEFAULT)
    }

    fn name(&self) -> &'static str {
        "CommonMark"
    }

    fn mime_type(&self) -> &'static str {
        "text/markdown"
    }
}

/// XML renderer implementing the Renderer trait.
///
/// This renderer generates XML output from the AST, following the
/// CommonMark DTD.
///
/// # Example
///
/// ```
/// use clmd::render::renderer::{Renderer, XmlRenderer};
/// use clmd::arena::NodeArena;
/// use clmd::blocks::BlockParser;
///
/// let mut arena = NodeArena::new();
/// let doc = BlockParser::parse(&mut arena, "# Hello\n\nWorld");
///
/// let renderer = XmlRenderer::new();
/// let xml = renderer.render(&arena, doc);
/// assert!(xml.contains("<document>"));
/// ```
#[derive(Debug, Default, Clone)]
pub struct XmlRenderer;

impl XmlRenderer {
    /// Create a new XML renderer.
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for XmlRenderer {
    fn render(&self, arena: &NodeArena, root: NodeId) -> String {
        let options = Options::new();
        self.render_with_options(arena, root, &options)
    }

    fn render_with_options(&self, arena: &NodeArena, root: NodeId, _options: &Options) -> String {
        crate::render::xml::render(arena, root, crate::options::DEFAULT)
    }

    fn name(&self) -> &'static str {
        "XML"
    }

    fn mime_type(&self) -> &'static str {
        "application/xml"
    }
}

/// LaTeX renderer implementing the Renderer trait.
///
/// This renderer generates LaTeX output from the AST.
#[derive(Debug, Default, Clone)]
pub struct LatexRenderer;

impl LatexRenderer {
    /// Create a new LaTeX renderer.
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for LatexRenderer {
    fn render(&self, arena: &NodeArena, root: NodeId) -> String {
        let options = Options::new();
        self.render_with_options(arena, root, &options)
    }

    fn render_with_options(&self, arena: &NodeArena, root: NodeId, _options: &Options) -> String {
        crate::render::latex::render(arena, root, crate::options::DEFAULT)
    }

    fn name(&self) -> &'static str {
        "LaTeX"
    }

    fn mime_type(&self) -> &'static str {
        "application/x-latex"
    }
}

/// Man page renderer implementing the Renderer trait.
///
/// This renderer generates Unix manual page output from the AST.
#[derive(Debug, Default, Clone)]
pub struct ManRenderer;

impl ManRenderer {
    /// Create a new Man page renderer.
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for ManRenderer {
    fn render(&self, arena: &NodeArena, root: NodeId) -> String {
        let options = Options::new();
        self.render_with_options(arena, root, &options)
    }

    fn render_with_options(&self, arena: &NodeArena, root: NodeId, _options: &Options) -> String {
        crate::render::man::render(arena, root, crate::options::DEFAULT)
    }

    fn name(&self) -> &'static str {
        "Man"
    }

    fn mime_type(&self) -> &'static str {
        "application/x-troff-man"
    }
}

/// A streaming renderer that writes output incrementally.
///
/// This trait is useful for rendering large documents without
/// keeping the entire output in memory.
///
/// # Example
///
/// ```
/// use clmd::render::renderer::{StreamingRenderer, HtmlRenderer};
/// use clmd::arena::NodeArena;
/// use clmd::blocks::BlockParser;
///
/// let mut arena = NodeArena::new();
/// let doc = BlockParser::parse(&mut arena, "# Hello\n\nWorld");
///
/// let renderer = HtmlRenderer::new();
/// let mut output = String::new();
/// renderer.render_to(&mut output, &arena, doc).unwrap();
/// ```
pub trait StreamingRenderer: Renderer {
    /// Render the AST to a writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer to write output to
    /// * `arena` - The node arena containing the AST
    /// * `root` - The root node ID to start rendering from
    ///
    /// # Returns
    ///
    /// Result indicating success or an IO error
    fn render_to<W: Write>(
        &self,
        writer: &mut W,
        arena: &NodeArena,
        root: NodeId,
    ) -> Result<(), std::fmt::Error>;

    /// Render the AST to a writer with options.
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer to write output to
    /// * `arena` - The node arena containing the AST
    /// * `root` - The root node ID to start rendering from
    /// * `options` - Rendering options
    ///
    /// # Returns
    ///
    /// Result indicating success or an IO error
    fn render_to_with_options<W: Write>(
        &self,
        writer: &mut W,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> Result<(), std::fmt::Error>;
}

impl StreamingRenderer for HtmlRenderer {
    fn render_to<W: Write>(
        &self,
        writer: &mut W,
        arena: &NodeArena,
        root: NodeId,
    ) -> Result<(), std::fmt::Error> {
        let options = Options::new();
        self.render_to_with_options(writer, arena, root, &options)
    }

    fn render_to_with_options<W: Write>(
        &self,
        writer: &mut W,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> Result<(), std::fmt::Error> {
        let output = self.render_with_options(arena, root, options);
        writer.write_str(&output)
    }
}

impl StreamingRenderer for CommonMarkRenderer {
    fn render_to<W: Write>(
        &self,
        writer: &mut W,
        arena: &NodeArena,
        root: NodeId,
    ) -> Result<(), std::fmt::Error> {
        let options = Options::new();
        self.render_to_with_options(writer, arena, root, &options)
    }

    fn render_to_with_options<W: Write>(
        &self,
        writer: &mut W,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> Result<(), std::fmt::Error> {
        let output = self.render_with_options(arena, root, options);
        writer.write_str(&output)
    }
}

impl StreamingRenderer for XmlRenderer {
    fn render_to<W: Write>(
        &self,
        writer: &mut W,
        arena: &NodeArena,
        root: NodeId,
    ) -> Result<(), std::fmt::Error> {
        let options = Options::new();
        self.render_to_with_options(writer, arena, root, &options)
    }

    fn render_to_with_options<W: Write>(
        &self,
        writer: &mut W,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> Result<(), std::fmt::Error> {
        let output = self.render_with_options(arena, root, options);
        writer.write_str(&output)
    }
}

impl StreamingRenderer for LatexRenderer {
    fn render_to<W: Write>(
        &self,
        writer: &mut W,
        arena: &NodeArena,
        root: NodeId,
    ) -> Result<(), std::fmt::Error> {
        let options = Options::new();
        self.render_to_with_options(writer, arena, root, &options)
    }

    fn render_to_with_options<W: Write>(
        &self,
        writer: &mut W,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> Result<(), std::fmt::Error> {
        let output = self.render_with_options(arena, root, options);
        writer.write_str(&output)
    }
}

impl StreamingRenderer for ManRenderer {
    fn render_to<W: Write>(
        &self,
        writer: &mut W,
        arena: &NodeArena,
        root: NodeId,
    ) -> Result<(), std::fmt::Error> {
        let options = Options::new();
        self.render_to_with_options(writer, arena, root, &options)
    }

    fn render_to_with_options<W: Write>(
        &self,
        writer: &mut W,
        arena: &NodeArena,
        root: NodeId,
        options: &Options,
    ) -> Result<(), std::fmt::Error> {
        let output = self.render_with_options(arena, root, options);
        writer.write_str(&output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::BlockParser;

    fn create_test_document() -> (NodeArena, NodeId) {
        let mut arena = NodeArena::new();
        let doc = BlockParser::parse(&mut arena, "# Hello\n\nWorld");
        (arena, doc)
    }

    #[test]
    fn test_html_renderer() {
        let (arena, doc) = create_test_document();
        let renderer = HtmlRenderer::new();
        
        let html = renderer.render(&arena, doc);
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<p>World</p>"));
        
        assert_eq!(renderer.name(), "HTML");
        assert_eq!(renderer.mime_type(), "text/html");
    }

    #[test]
    fn test_commonmark_renderer() {
        let (arena, doc) = create_test_document();
        let renderer = CommonMarkRenderer::new();
        
        let cm = renderer.render(&arena, doc);
        assert!(cm.contains("# Hello"));
        assert!(cm.contains("World"));
        
        assert_eq!(renderer.name(), "CommonMark");
        assert_eq!(renderer.mime_type(), "text/markdown");
    }

    #[test]
    fn test_xml_renderer() {
        let (arena, doc) = create_test_document();
        let renderer = XmlRenderer::new();
        
        let xml = renderer.render(&arena, doc);
        assert!(xml.contains("<document>"));
        assert!(xml.contains("<heading"));
        assert!(xml.contains("<paragraph>"));
        
        assert_eq!(renderer.name(), "XML");
        assert_eq!(renderer.mime_type(), "application/xml");
    }

    #[test]
    fn test_latex_renderer() {
        let (arena, doc) = create_test_document();
        let renderer = LatexRenderer::new();
        
        let latex = renderer.render(&arena, doc);
        assert!(latex.contains("\\section"));
        
        assert_eq!(renderer.name(), "LaTeX");
    }

    #[test]
    fn test_man_renderer() {
        let (arena, doc) = create_test_document();
        let renderer = ManRenderer::new();
        
        let man = renderer.render(&arena, doc);
        assert!(man.contains(".SH"));
        
        assert_eq!(renderer.name(), "Man");
    }

    #[test]
    fn test_streaming_html_renderer() {
        let (arena, doc) = create_test_document();
        let renderer = HtmlRenderer::new();
        
        let mut output = String::new();
        renderer.render_to(&mut output, &arena, doc).unwrap();
        
        assert!(output.contains("<h1>Hello</h1>"));
        assert!(output.contains("<p>World</p>"));
    }

    #[test]
    fn test_renderer_trait_object() {
        let (arena, doc) = create_test_document();
        
        fn render_with_renderer<R: Renderer>(renderer: &R, arena: &NodeArena, root: NodeId) -> String {
            renderer.render(arena, root)
        }
        
        let html_renderer = HtmlRenderer::new();
        let html = render_with_renderer(&html_renderer, &arena, doc);
        assert!(html.contains("<h1>"));
        
        let cm_renderer = CommonMarkRenderer::new();
        let cm = render_with_renderer(&cm_renderer, &arena, doc);
        assert!(cm.contains("# Hello"));
    }

    #[test]
    fn test_renderer_with_options() {
        let (arena, doc) = create_test_document();
        let renderer = HtmlRenderer::new();
        let options = Options::new();
        
        let html = renderer.render_with_options(&arena, doc, &options);
        assert!(html.contains("<h1>Hello</h1>"));
    }
}
