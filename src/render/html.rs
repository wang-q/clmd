//! HTML rendering for the CommonMark AST.
//!
//! This module provides functions for rendering CommonMark documents as HTML,
//! inspired by comrak's design.
//!
//! # Example
//!
//! ```
//! use clmd::{Arena, parse_document, Options, format_html};
//! use std::fmt::Write;
//!
//! let arena = Arena::new();
//! let options = Options::default();
//! let root = parse_document(&arena, "# Hello\n\nWorld", &options);
//!
//! let mut html = String::new();
//! format_html(root, &options, &mut html).unwrap();
//!
//! assert!(html.contains("<h1>"));
//! assert!(html.contains("<p>"));
//! ```

use std::fmt::{self, Write};

use crate::nodes::{
    AstNode, ListType, NodeCode, NodeCodeBlock, NodeFootnoteDefinition,
    NodeFootnoteReference, NodeHeading, NodeHtmlBlock, NodeLink, NodeList, NodeMath,
    NodeTable, NodeTaskItem, NodeValue,
};
use crate::parser::options::{Options, Plugins};

// Re-export context types
mod context {
    use super::*;

    /// Context for HTML rendering.
    ///
    /// This struct holds the rendering options, plugins, and output buffer,
    /// providing a unified interface for HTML rendering operations.
    pub struct Context<'o, 'c: 'o> {
        /// The options for rendering
        pub options: &'o Options<'c>,
        /// The plugins for rendering
        pub plugins: &'o Plugins<'o>,
        /// The output buffer
        output: &'o mut dyn Write,
        /// Current footnote index
        pub footnote_ix: u32,
        /// Track the last character written for cr() logic
        last_char: Option<char>,
    }

    impl<'o, 'c: 'o> fmt::Debug for Context<'o, 'c> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Context")
                .field("options", &self.options)
                .field("plugins", &self.plugins)
                .field("output", &"<dyn Write>")
                .field("footnote_ix", &self.footnote_ix)
                .field("last_char", &self.last_char)
                .finish()
        }
    }

    impl<'o, 'c: 'o> Context<'o, 'c> {
        /// Create a new rendering context.
        pub fn new(
            output: &'o mut dyn Write,
            options: &'o Options<'c>,
            plugins: &'o Plugins<'o>,
        ) -> Self {
            Self {
                options,
                plugins,
                output,
                footnote_ix: 0,
                last_char: None,
            }
        }

        /// Write a string to the output and track the last character.
        pub fn write_str(&mut self, s: &str) -> fmt::Result {
            self.output.write_str(s)?;
            if let Some(c) = s.chars().last() {
                self.last_char = Some(c);
            }
            Ok(())
        }

        /// Write a newline if the last output wasn't already a newline.
        /// For the beginning of the document, don't add a newline.
        pub fn cr(&mut self) -> fmt::Result {
            if self.last_char.is_none() {
                return Ok(());
            }
            if self.last_char != Some('\n') {
                self.output.write_str("\n")?;
                self.last_char = Some('\n');
            }
            Ok(())
        }

        /// Write a line feed (newline).
        pub fn lf(&mut self) -> fmt::Result {
            self.output.write_str("\n")?;
            self.last_char = Some('\n');
            Ok(())
        }

        /// Escape HTML special characters.
        pub fn escape(&mut self, text: &str) -> fmt::Result {
            escape_html(self.output, text)
        }

        /// Escape a URL for use in an href attribute.
        pub fn escape_href(&mut self, url: &str) -> fmt::Result {
            escape_href(self.output, url)
        }

        /// Finish rendering and return the result.
        pub fn finish(self) -> fmt::Result {
            Ok(())
        }
    }

    impl Write for Context<'_, '_> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.output.write_str(s)
        }
    }
}

pub use context::Context;

/// Child rendering mode for formatters.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum ChildRendering {
    /// Render children in full HTML mode.
    HTML,
    /// Render children in plain text mode (for alt text, etc.).
    Plain,
    /// Skip rendering children entirely.
    Skip,
}

/// Formats an AST as HTML.
pub fn format_document(
    root: &AstNode<'_>,
    options: &Options,
    output: &mut dyn Write,
) -> fmt::Result {
    format_document_with_plugins(root, options, output, &Plugins::default())
}

/// Formats an AST as HTML with plugins.
pub fn format_document_with_plugins(
    root: &AstNode<'_>,
    options: &Options,
    output: &mut dyn Write,
    plugins: &Plugins<'_>,
) -> fmt::Result {
    let mut context = Context::new(output, options, plugins);

    enum Phase {
        Pre,
        Post,
    }

    let mut stack = vec![(root, ChildRendering::HTML, Phase::Pre)];

    while let Some((node, child_rendering, phase)) = stack.pop() {
        match phase {
            Phase::Pre => {
                let new_cr = match child_rendering {
                    ChildRendering::Plain => {
                        render_plain(&mut context, node)?;
                        ChildRendering::Plain
                    }
                    ChildRendering::HTML => {
                        stack.push((node, ChildRendering::HTML, Phase::Post));
                        format_node_default(&mut context, node, true)?
                    }
                    ChildRendering::Skip => unreachable!(),
                };

                if !matches!(new_cr, ChildRendering::Skip) {
                    let mut child_opt = node.last_child();
                    while let Some(child) = child_opt {
                        stack.push((child, new_cr, Phase::Pre));
                        child_opt = child.previous_sibling();
                    }
                }
            }
            Phase::Post => {
                format_node_default(&mut context, node, false)?;
            }
        }
    }

    context.finish()
}

/// Render a node in plain text mode (for alt text, etc.).
fn render_plain(context: &mut Context, node: &AstNode<'_>) -> fmt::Result {
    let ast = node.data.borrow();
    match &ast.value {
        NodeValue::Text(text) => {
            context.escape(text)?;
        }
        NodeValue::Code(code) => {
            context.escape(&code.literal)?;
        }
        NodeValue::SoftBreak | NodeValue::HardBreak => {
            context.write_str(" ")?;
        }
        _ => {}
    }
    Ok(())
}

/// Default node formatting function.
fn format_node_default(
    context: &mut Context,
    node: &AstNode<'_>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let ast = node.data.borrow();

    match &ast.value {
        NodeValue::Document => Ok(ChildRendering::HTML),
        NodeValue::BlockQuote => render_block_quote(context, node, entering),
        NodeValue::List(list) => render_list(context, node, entering, list),
        NodeValue::Item(_) => render_item(context, node, entering),
        NodeValue::CodeBlock(code_block) => {
            render_code_block(context, node, entering, code_block)
        }
        NodeValue::HtmlBlock(html_block) => {
            render_html_block(context, entering, html_block)
        }
        NodeValue::Paragraph => render_paragraph(context, node, entering),
        NodeValue::Heading(heading) => render_heading(context, node, entering, heading),
        NodeValue::ThematicBreak => render_thematic_break(context, node, entering),
        NodeValue::Text(text) => render_text(context, entering, text),
        NodeValue::SoftBreak => render_soft_break(context, node, entering),
        NodeValue::HardBreak => render_hard_break(context, node, entering),
        NodeValue::Code(code) => render_code(context, entering, code),
        NodeValue::HtmlInline(html) => render_html_inline(context, entering, html),
        NodeValue::Emph => render_emph(context, node, entering),
        NodeValue::Strong => render_strong(context, node, entering),
        NodeValue::Link(link) => render_link(context, node, entering, link),
        NodeValue::Image(link) => render_image(context, node, entering, link),
        NodeValue::Strikethrough => render_strikethrough(context, node, entering),
        NodeValue::TaskItem(task_item) => {
            render_task_item(context, node, entering, task_item)
        }
        NodeValue::Table(table) => render_table(context, node, entering, table),
        NodeValue::TableRow(is_header) => {
            render_table_row(context, node, entering, *is_header)
        }
        NodeValue::TableCell => render_table_cell(context, node, entering),
        NodeValue::FootnoteDefinition(def) => {
            render_footnote_definition(context, node, entering, def)
        }
        NodeValue::FootnoteReference(reference) => {
            render_footnote_reference(context, node, entering, reference)
        }
        NodeValue::Math(math) => render_math(context, node, entering, math),
        _ => Ok(ChildRendering::HTML),
    }
}

// Block element renderers
fn render_block_quote(
    context: &mut Context,
    node: &AstNode<'_>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        context.write_str("<blockquote")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
        context.lf()?;
    } else {
        context.cr()?;
        context.write_str("</blockquote>")?;
        context.lf()?;
    }
    Ok(ChildRendering::HTML)
}

fn render_list(
    context: &mut Context,
    node: &AstNode<'_>,
    entering: bool,
    list: &NodeList,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        match list.list_type {
            ListType::Bullet => {
                context.write_str("<ul")?;
                render_sourcepos(context, node)?;
                context.write_str(">")?;
            }
            ListType::Ordered => {
                context.write_str("<ol")?;
                render_sourcepos(context, node)?;
                if list.start != 1 {
                    write!(context, " start=\"{}\"", list.start)?;
                }
                context.write_str(">")?;
            }
        }
        context.lf()?;
    } else {
        match list.list_type {
            ListType::Bullet => {
                context.write_str("</ul>")?;
            }
            ListType::Ordered => {
                context.write_str("</ol>")?;
            }
        }
        context.lf()?;
    }
    Ok(ChildRendering::HTML)
}

fn render_item(
    context: &mut Context,
    node: &AstNode<'_>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        context.write_str("<li")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
    } else {
        context.write_str("</li>")?;
        context.lf()?;
    }
    Ok(ChildRendering::HTML)
}

fn render_code_block(
    context: &mut Context,
    _node: &AstNode<'_>,
    entering: bool,
    code_block: &NodeCodeBlock,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;

        if let Some(highlighter) = &context.plugins.render.codefence_syntax_highlighter {
            let mut attrs = std::collections::HashMap::new();
            if !code_block.info.is_empty() {
                let lang = code_block.info.split_whitespace().next().unwrap_or("");
                if !lang.is_empty() {
                    attrs.insert(
                        "class",
                        std::borrow::Cow::Owned(format!("language-{}", lang)),
                    );
                }
            }

            highlighter.write_pre_tag(context, attrs.clone())?;
            highlighter.write_code_tag(context, attrs)?;
            highlighter.write_highlighted(
                context,
                Some(&code_block.info),
                &code_block.literal,
            )?;
            context.write_str("</code></pre>")?;
        } else {
            context.write_str("<pre><code")?;
            if !code_block.info.is_empty() {
                let lang = code_block.info.split_whitespace().next().unwrap_or("");
                if !lang.is_empty() {
                    write!(context, " class=\"language-{}\"", lang)?;
                }
            }
            context.write_str(">")?;
            escape_html(context, &code_block.literal)?;
            context.write_str("</code></pre>")?;
        }
        context.lf()?;
    }
    Ok(ChildRendering::HTML)
}

fn render_html_block(
    context: &mut Context,
    entering: bool,
    html_block: &NodeHtmlBlock,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        if context.options.render.escape {
            escape_html(context, &html_block.literal)?;
        } else {
            context.write_str(&html_block.literal)?;
        }
        context.cr()?;
    }
    Ok(ChildRendering::HTML)
}

fn render_paragraph(
    context: &mut Context,
    node: &AstNode<'_>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let tight = node
        .parent()
        .and_then(|n| n.parent())
        .map(|n| {
            let ast = n.data.borrow();
            match &ast.value {
                NodeValue::List(list) => list.tight,
                _ => false,
            }
        })
        .unwrap_or(false);

    if !tight {
        if entering {
            context.cr()?;
            context.write_str("<p")?;
            render_sourcepos(context, node)?;
            context.write_str(">")?;
        } else {
            context.write_str("</p>")?;
            context.lf()?;
        }
    }
    Ok(ChildRendering::HTML)
}

fn render_heading(
    context: &mut Context,
    node: &AstNode<'_>,
    entering: bool,
    heading: &NodeHeading,
) -> Result<ChildRendering, fmt::Error> {
    match &context.plugins.render.heading_adapter {
        None => {
            if entering {
                context.cr()?;
                write!(context, "<h{}", heading.level)?;
                render_sourcepos(context, node)?;
                context.write_str(">")?;
            } else {
                write!(context, "</h{}>", heading.level)?;
                context.lf()?;
            }
        }
        Some(adapter) => {
            use crate::adapters::HeadingMeta;

            let text_content = collect_text(node);
            let meta = HeadingMeta {
                level: heading.level,
                content: text_content,
            };

            if entering {
                context.cr()?;
                let sp = if context.options.render.sourcepos {
                    let ast = node.data.borrow();
                    Some(ast.sourcepos)
                } else {
                    None
                };
                adapter.enter(context, &meta, sp)?;
            } else {
                adapter.exit(context, &meta)?;
            }
        }
    }
    Ok(ChildRendering::HTML)
}

fn render_thematic_break(
    context: &mut Context,
    node: &AstNode<'_>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        context.write_str("<hr")?;
        render_sourcepos(context, node)?;
        context.write_str(" />")?;
        context.lf()?;
    }
    Ok(ChildRendering::HTML)
}

// Inline element renderers
fn render_text(
    context: &mut Context,
    entering: bool,
    text: &str,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        escape_html(context, text)?;
    }
    Ok(ChildRendering::HTML)
}

fn render_soft_break(
    context: &mut Context,
    _node: &AstNode<'_>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        if context.options.render.hardbreaks {
            context.write_str("<br />\n")?;
        } else {
            context.write_str("\n")?;
        }
    }
    Ok(ChildRendering::HTML)
}

fn render_hard_break(
    context: &mut Context,
    node: &AstNode<'_>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<br")?;
        render_sourcepos(context, node)?;
        context.write_str(" />\n")?;
    }
    Ok(ChildRendering::HTML)
}

fn render_code(
    context: &mut Context,
    entering: bool,
    code: &NodeCode,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<code>")?;
        escape_html(context, &code.literal)?;
        context.write_str("</code>")?;
    }
    Ok(ChildRendering::HTML)
}

fn render_html_inline(
    context: &mut Context,
    entering: bool,
    html: &str,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        if context.options.render.escape {
            escape_html(context, html)?;
        } else {
            context.write_str(html)?;
        }
    }
    Ok(ChildRendering::HTML)
}

fn render_emph(
    context: &mut Context,
    _node: &AstNode<'_>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<em>")?;
    } else {
        context.write_str("</em>")?;
    }
    Ok(ChildRendering::HTML)
}

fn render_strong(
    context: &mut Context,
    _node: &AstNode<'_>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<strong>")?;
    } else {
        context.write_str("</strong>")?;
    }
    Ok(ChildRendering::HTML)
}

fn render_link(
    context: &mut Context,
    _node: &AstNode<'_>,
    entering: bool,
    link: &NodeLink,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<a href=\"")?;
        escape_href(context, &link.url)?;
        context.write_str("\"")?;
        if !link.title.is_empty() {
            context.write_str(" title=\"")?;
            escape_html(context, &link.title)?;
            context.write_str("\"")?;
        }
        context.write_str(">")?;
    } else {
        context.write_str("</a>")?;
    }
    Ok(ChildRendering::HTML)
}

fn render_image(
    context: &mut Context,
    _node: &AstNode<'_>,
    entering: bool,
    link: &NodeLink,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<img src=\"")?;
        escape_href(context, &link.url)?;
        context.write_str("\" alt=\"")?;
        return Ok(ChildRendering::Plain);
    } else {
        if !link.title.is_empty() {
            context.write_str("\" title=\"")?;
            escape_html(context, &link.title)?;
        }
        context.write_str("\" />")?;
    }
    Ok(ChildRendering::HTML)
}

fn render_strikethrough(
    context: &mut Context,
    _node: &AstNode<'_>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<del>")?;
    } else {
        context.write_str("</del>")?;
    }
    Ok(ChildRendering::HTML)
}

// GFM extension renderers
fn render_task_item(
    context: &mut Context,
    node: &AstNode<'_>,
    entering: bool,
    task_item: &NodeTaskItem,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        context.write_str("<li")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;

        let checked = task_item.symbol.is_some();
        context.write_str("<input type=\"checkbox\" disabled=\"disabled\"")?;
        if checked {
            context.write_str(" checked=\"checked\"")?;
        }
        context.write_str(" /> ")?;
    } else {
        context.write_str("</li>")?;
        context.lf()?;
    }
    Ok(ChildRendering::HTML)
}

fn render_table(
    context: &mut Context,
    node: &AstNode<'_>,
    entering: bool,
    _table: &NodeTable,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        context.write_str("<table")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
        context.lf()?;
    } else {
        context.cr()?;
        context.write_str("</table>")?;
        context.lf()?;
    }
    Ok(ChildRendering::HTML)
}

fn render_table_row(
    context: &mut Context,
    _node: &AstNode<'_>,
    entering: bool,
    is_header: bool,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;
        if is_header {
            context.write_str("<thead>")?;
            context.lf()?;
        }
        context.write_str("<tr>")?;
        context.lf()?;
    } else {
        context.cr()?;
        context.write_str("</tr>")?;
        if is_header {
            context.lf()?;
            context.cr()?;
            context.write_str("</thead>")?;
            context.lf()?;
            context.write_str("<tbody>")?;
        }
        context.lf()?;
    }
    Ok(ChildRendering::HTML)
}

fn render_table_cell(
    context: &mut Context,
    node: &AstNode<'_>,
    entering: bool,
) -> Result<ChildRendering, fmt::Error> {
    let is_header = node
        .parent()
        .map(|parent| {
            let ast = parent.data.borrow();
            matches!(ast.value, NodeValue::TableRow(true))
        })
        .unwrap_or(false);

    if entering {
        context.cr()?;
        if is_header {
            context.write_str("<th>")?;
        } else {
            context.write_str("<td>")?;
        }
    } else if is_header {
        context.write_str("</th>")?;
    } else {
        context.write_str("</td>")?;
    }
    Ok(ChildRendering::HTML)
}

fn render_footnote_definition(
    context: &mut Context,
    _node: &AstNode<'_>,
    entering: bool,
    def: &NodeFootnoteDefinition,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        if context.footnote_ix == 0 {
            context.cr()?;
            context.write_str("<section class=\"footnotes\">")?;
            context.lf()?;
            context.write_str("<ol>")?;
            context.lf()?;
        }
        context.footnote_ix += 1;
        context.cr()?;
        context.write_str("<li id=\"fn-")?;
        escape_html(context, &def.name)?;
        context.write_str("\">")?;
    } else {
        context.write_str("</li>")?;
        context.lf()?;
    }
    Ok(ChildRendering::HTML)
}

fn render_footnote_reference(
    context: &mut Context,
    _node: &AstNode<'_>,
    entering: bool,
    reference: &NodeFootnoteReference,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<sup class=\"footnote-ref\"><a href=\"#fn-")?;
        escape_html(context, &reference.name)?;
        context.write_str("\" id=\"fnref-")?;
        escape_html(context, &reference.name)?;
        context.write_str("\">[")?;
        escape_html(context, &reference.name)?;
        context.write_str("]</a></sup>")?;
    }
    Ok(ChildRendering::HTML)
}

fn render_math(
    context: &mut Context,
    node: &AstNode<'_>,
    entering: bool,
    math: &NodeMath,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        let style = if math.display_math {
            "display"
        } else {
            "inline"
        };
        context.write_str("<span data-math-style=\"")?;
        context.write_str(style)?;
        context.write_str("\"")?;
        render_sourcepos(context, node)?;
        context.write_str(">")?;
        escape_html(context, &math.literal)?;
        context.write_str("</span>")?;
    }
    Ok(ChildRendering::HTML)
}

// Helper functions
fn render_sourcepos(context: &mut Context, node: &AstNode<'_>) -> fmt::Result {
    if context.options.render.sourcepos {
        let ast = node.data.borrow();
        if ast.sourcepos.start.line > 0 {
            write!(context, " data-sourcepos=\"{}\"", ast.sourcepos)
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

/// Collect text content from a node and its children.
pub fn collect_text(node: &AstNode<'_>) -> String {
    let mut text = String::new();
    collect_text_append(node, &mut text);
    text
}

/// Append text content from a node and its children to a buffer.
pub fn collect_text_append(node: &AstNode<'_>, output: &mut String) {
    let ast = node.data.borrow();
    match &ast.value {
        NodeValue::Text(text) => output.push_str(text),
        NodeValue::Code(code) => output.push_str(&code.literal),
        NodeValue::SoftBreak | NodeValue::HardBreak => output.push(' '),
        _ => {
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                collect_text_append(child, output);
                child_opt = child.next_sibling();
            }
        }
    }
}

/// Escape HTML special characters.
pub fn escape_html(output: &mut dyn Write, text: &str) -> fmt::Result {
    let bytes = text.as_bytes();
    let mut last = 0;

    for (i, &b) in bytes.iter().enumerate() {
        let escaped = match b {
            b'&' => Some("&amp;"),
            b'<' => Some("&lt;"),
            b'>' => Some("&gt;"),
            b'"' => Some("&quot;"),
            _ => None,
        };

        if let Some(esc) = escaped {
            if i > last {
                output.write_str(&text[last..i])?;
            }
            output.write_str(esc)?;
            last = i + 1;
        }
    }

    if last < text.len() {
        output.write_str(&text[last..])?;
    }

    Ok(())
}

/// Escape a string for use in HTML attribute context.
pub fn escape_href(output: &mut dyn Write, url: &str) -> fmt::Result {
    if !is_safe_url(url) {
        return output.write_str("#");
    }

    let bytes = url.as_bytes();
    let mut last = 0;

    for (i, &b) in bytes.iter().enumerate() {
        let escaped = match b {
            b'&' => Some("&amp;"),
            b'"' => Some("&quot;"),
            b'<' => Some("&lt;"),
            b'>' => Some("&gt;"),
            b'\'' => Some("&#x27;"),
            b'`' => Some("&#x60;"),
            _ => None,
        };

        if let Some(esc) = escaped {
            if i > last {
                output.write_str(&url[last..i])?;
            }
            output.write_str(esc)?;
            last = i + 1;
        }
    }

    if last < url.len() {
        output.write_str(&url[last..])?;
    }

    Ok(())
}

/// Check if a URL is safe to use in an href attribute.
pub fn is_safe_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();

    let dangerous_protocols = [
        "javascript:",
        "vbscript:",
        "file:",
        "data:text/html",
        "data:text/javascript",
    ];

    for protocol in &dangerous_protocols {
        if url_lower.starts_with(protocol) {
            return false;
        }
    }

    true
}

// Legacy compatibility
/// Render an arena_tree node to HTML (legacy compatibility).
pub fn render_from_node<'a>(root: crate::nodes::Node<'a>, _options: u32) -> String {
    use crate::parser::options::Options;

    let options = Options::default();
    let mut output = String::new();
    let _ = format_document_legacy(&options, &mut output, root);
    output
}

/// Render a node tree as HTML (legacy API).
///
/// This function provides compatibility with the old arena-based API.
/// New code should use [`format_document`] instead.
pub fn render(arena: &crate::arena::NodeArena, root: crate::arena::NodeId, options: u32) -> String {
    let mut renderer = HtmlRenderer::new(arena, options);
    renderer.render(root)
}

/// HTML renderer for arena-based AST (based on 100% passing implementation).
struct HtmlRenderer<'a> {
    arena: &'a crate::arena::NodeArena,
    output: String,
    /// Stack for tracking whether we need to close a tag
    tag_stack: Vec<&'static str>,
    /// Track if we're in a tight list
    tight_list_stack: Vec<bool>,
    /// Track if we're in a code block
    in_code_block: bool,
    /// Track the last output character for cr() logic
    last_out: char,
}

impl<'a> HtmlRenderer<'a> {
    fn new(arena: &'a crate::arena::NodeArena, _options: u32) -> Self {
        HtmlRenderer {
            arena,
            output: String::new(),
            tag_stack: Vec::new(),
            tight_list_stack: Vec::new(),
            in_code_block: false,
            last_out: '\n',
        }
    }

    fn render(&mut self, root: crate::arena::NodeId) -> String {
        self.render_node(root, true);
        // Remove trailing newlines
        while self.output.ends_with('\n') {
            self.output.pop();
        }
        self.output.clone()
    }

    /// Output a newline if the last output wasn't already a newline
    fn cr(&mut self) {
        if self.last_out != '\n' {
            self.output.push('\n');
            self.last_out = '\n';
        }
    }

    /// Output a literal string and track last character
    fn lit(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        self.output.push_str(s);
        self.last_out = s.chars().last().unwrap_or('\n');
    }

    /// Check if we're currently inside a tight list
    fn in_tight_list(&self) -> bool {
        self.tight_list_stack.last().copied().unwrap_or(false)
    }

    fn render_node(&mut self, node_id: crate::arena::NodeId, is_root: bool) {
        let node = self.arena.get(node_id);
        match &node.value {
            NodeValue::Document => {
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id, false);
                    child_opt = self.arena.get(child_id).next;
                }
            }
            NodeValue::Paragraph => {
                // In tight lists, paragraphs are not wrapped in <p> tags
                if !self.in_tight_list() {
                    self.cr();
                    self.lit("<p>");
                    self.tag_stack.push("p");
                    let mut child_opt = node.first_child;
                    while let Some(child_id) = child_opt {
                        self.render_inline(child_id);
                        child_opt = self.arena.get(child_id).next;
                    }
                    self.lit("</p>");
                    self.tag_stack.pop();
                } else {
                    // In tight lists, just render children without <p> tags
                    let mut child_opt = node.first_child;
                    while let Some(child_id) = child_opt {
                        self.render_inline(child_id);
                        child_opt = self.arena.get(child_id).next;
                    }
                }
            }
            NodeValue::Text(text) => {
                self.output.push_str(&escape_html_str(text));
            }
            NodeValue::Emph => {
                self.output.push_str("<em>");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push_str("</em>");
            }
            NodeValue::Strong => {
                self.output.push_str("<strong>");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push_str("</strong>");
            }
            NodeValue::Code(code) => {
                self.output.push_str("<code>");
                self.output.push_str(&escape_html_str(&code.literal));
                self.output.push_str("</code>");
            }
            NodeValue::Heading(heading) => {
                self.cr();
                self.lit(&format!("<h{}>", heading.level));
                self.tag_stack.push("h");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.lit(&format!("</h{}>", heading.level));
                self.lit("\n");
                self.tag_stack.pop();
            }
            NodeValue::Link(link) => {
                if link.title.is_empty() {
                    self.output.push_str(&format!("<a href=\"{}\">", escape_href_str(&link.url)));
                } else {
                    self.output.push_str(&format!(
                        "<a href=\"{}\" title=\"{}\">",
                        escape_href_str(&link.url),
                        escape_html_str(&link.title)
                    ));
                }
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push_str("</a>");
            }
            NodeValue::BlockQuote => {
                self.cr();
                self.lit("<blockquote>");
                self.lit("\n");
                self.tag_stack.push("blockquote");
                self.tight_list_stack.push(false);
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id, false);
                    child_opt = self.arena.get(child_id).next;
                }
                self.lit("</blockquote>");
                self.lit("\n");
                self.tag_stack.pop();
                self.tight_list_stack.pop();
            }
            NodeValue::CodeBlock(code_block) => {
                self.cr();
                self.in_code_block = true;
                self.lit("<pre><code");
                if !code_block.info.is_empty() {
                    let lang = code_block.info.split_whitespace().next().unwrap_or("");
                    if !lang.is_empty() {
                        self.lit(&format!(" class=\"language-{}\"", escape_html_str(lang)));
                    }
                }
                self.lit(">");
                self.lit(&escape_html_str(&code_block.literal));
                self.lit("</code></pre>");
                self.lit("\n");
                self.in_code_block = false;
            }
            NodeValue::List(list) => {
                self.tight_list_stack.push(list.tight);
                self.cr();
                match list.list_type {
                    ListType::Bullet => {
                        self.lit("<ul>");
                        self.tag_stack.push("ul");
                    }
                    ListType::Ordered => {
                        self.lit("<ol>");
                        self.tag_stack.push("ol");
                    }
                }
                self.lit("\n");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id, false);
                    child_opt = self.arena.get(child_id).next;
                }
                if let Some(tag) = self.tag_stack.pop() {
                    self.lit(&format!("</{}>", tag));
                    self.lit("\n");
                }
                self.tight_list_stack.pop();
            }
            NodeValue::Item(..) => {
                self.lit("<li>");
                self.tag_stack.push("li");
                let has_children = node.first_child.is_some();
                if !self.in_tight_list() && has_children {
                    self.lit("\n");
                }
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_node(child_id, false);
                    child_opt = self.arena.get(child_id).next;
                }
                if let Some(tag) = self.tag_stack.pop() {
                    self.lit(&format!("</{}>", tag));
                    self.lit("\n");
                }
            }
            NodeValue::ThematicBreak => {
                self.cr();
                self.lit("<hr />");
                self.lit("\n");
            }
            NodeValue::SoftBreak => {
                if self.in_code_block {
                    self.lit("\n");
                } else if self.in_tight_list() {
                    self.lit(" ");
                } else {
                    self.lit("\n");
                }
            }
            NodeValue::HardBreak => {
                self.lit("<br />");
                self.lit("\n");
            }
            NodeValue::HtmlBlock(block) => {
                self.cr();
                self.lit(&block.literal);
                self.lit("\n");
            }
            NodeValue::HtmlInline(html) => {
                self.output.push_str(html);
            }
            _ => {}
        }
    }

    /// Render inline content without adding block-level newlines
    fn render_inline(&mut self, node_id: crate::arena::NodeId) {
        let node = self.arena.get(node_id);
        match &node.value {
            NodeValue::Text(text) => {
                self.output.push_str(&escape_html_str(text));
            }
            NodeValue::Emph => {
                self.output.push_str("<em>");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push_str("</em>");
            }
            NodeValue::Strong => {
                self.output.push_str("<strong>");
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push_str("</strong>");
            }
            NodeValue::Code(code) => {
                self.output.push_str("<code>");
                self.output.push_str(&escape_html_str(&code.literal));
                self.output.push_str("</code>");
            }
            NodeValue::Link(link) => {
                if link.title.is_empty() {
                    self.output.push_str(&format!("<a href=\"{}\">", escape_href_str(&link.url)));
                } else {
                    self.output.push_str(&format!(
                        "<a href=\"{}\" title=\"{}\">",
                        escape_href_str(&link.url),
                        escape_html_str(&link.title)
                    ));
                }
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
                self.output.push_str("</a>");
            }
            NodeValue::Image(image) => {
                // Get alt text from child nodes
                let mut alt_text = String::new();
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.collect_alt_text(child_id, &mut alt_text);
                    child_opt = self.arena.get(child_id).next;
                }
                
                if image.title.is_empty() {
                    self.output.push_str(&format!(
                        "<img src=\"{}\" alt=\"{}\" />",
                        escape_href_str(&image.url),
                        escape_html_str(&alt_text)
                    ));
                } else {
                    self.output.push_str(&format!(
                        "<img src=\"{}\" alt=\"{}\" title=\"{}\" />",
                        escape_href_str(&image.url),
                        escape_html_str(&alt_text),
                        escape_html_str(&image.title)
                    ));
                }
            }
            NodeValue::SoftBreak => {
                self.output.push('\n');
            }
            NodeValue::HardBreak => {
                self.output.push_str("<br />\n");
            }
            NodeValue::HtmlInline(html) => {
                self.output.push_str(html);
            }
            _ => {
                // For other node types, just render children
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.render_inline(child_id);
                    child_opt = self.arena.get(child_id).next;
                }
            }
        }
    }

    /// Collect alt text from a node (for image rendering)
    fn collect_alt_text(&self, node_id: crate::arena::NodeId, output: &mut String) {
        let node = self.arena.get(node_id);
        match &node.value {
            NodeValue::Text(text) => {
                output.push_str(text);
            }
            NodeValue::Emph | NodeValue::Strong | NodeValue::Code(_) => {
                // For inline formatting, just collect the text content
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.collect_alt_text_inner(child_id, output);
                    child_opt = self.arena.get(child_id).next;
                }
            }
            _ => {
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.collect_alt_text(child_id, output);
                    child_opt = self.arena.get(child_id).next;
                }
            }
        }
    }

    fn collect_alt_text_inner(&self, node_id: crate::arena::NodeId, output: &mut String) {
        let node = self.arena.get(node_id);
        match &node.value {
            NodeValue::Text(text) => {
                output.push_str(text);
            }
            _ => {
                let mut child_opt = node.first_child;
                while let Some(child_id) = child_opt {
                    self.collect_alt_text_inner(child_id, output);
                    child_opt = self.arena.get(child_id).next;
                }
            }
        }
    }
}

fn escape_html_str(text: &str) -> String {
    let mut output = String::new();
    escape_html(&mut output, text).unwrap();
    output
}

fn escape_href_str(url: &str) -> String {
    let mut output = String::new();
    escape_href(&mut output, url).unwrap();
    output
}

fn format_document_legacy(
    options: &Options,
    output: &mut dyn Write,
    root: crate::nodes::Node<'_>,
) -> fmt::Result {
    render_node_legacy(options, output, root)
}

fn render_node_legacy(
    options: &Options,
    output: &mut dyn Write,
    node: crate::nodes::Node<'_>,
) -> fmt::Result {
    let ast = node.data.borrow();

    match &ast.value {
        NodeValue::Document => {
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                render_node_legacy(options, output, child)?;
                child_opt = child.next_sibling();
            }
        }
        NodeValue::Paragraph => {
            write!(output, "<p>")?;
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                render_node_legacy(options, output, child)?;
                child_opt = child.next_sibling();
            }
            write!(output, "</p>\n")?;
        }
        NodeValue::Text(text) => {
            escape_html(output, text)?;
        }
        NodeValue::Heading(heading) => {
            write!(output, "<h{}>", heading.level)?;
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                render_node_legacy(options, output, child)?;
                child_opt = child.next_sibling();
            }
            write!(output, "</h{}>\n", heading.level)?;
        }
        _ => {
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                render_node_legacy(options, output, child)?;
                child_opt = child.next_sibling();
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{Node, NodeArena, TreeOps};
    use crate::nodes::{NodeCode, NodeCodeBlock, NodeLink};

    #[test]
    fn test_escape_html() {
        let mut output = String::new();
        escape_html(&mut output, "<div>").unwrap();
        assert_eq!(output, "&lt;div&gt;");

        output.clear();
        escape_html(&mut output, "&").unwrap();
        assert_eq!(output, "&amp;");

        output.clear();
        escape_html(&mut output, "\"test\"").unwrap();
        assert_eq!(output, "&quot;test&quot;");
    }

    #[test]
    fn test_is_safe_url() {
        assert!(!is_safe_url("javascript:alert('xss')"));
        assert!(!is_safe_url("JAVASCRIPT:alert('xss')"));
        assert!(!is_safe_url("vbscript:msgbox('xss')"));
        assert!(!is_safe_url("file:///etc/passwd"));
        assert!(is_safe_url("https://example.com"));
        assert!(is_safe_url("http://example.com"));
        assert!(is_safe_url("/path/to/page"));
        assert!(is_safe_url("#anchor"));
    }

    #[test]
    fn test_escape_href() {
        let mut output = String::new();
        escape_href(&mut output, "https://example.com?a=1&b=2").unwrap();
        assert_eq!(output, "https://example.com?a=1&amp;b=2");

        output.clear();
        escape_href(&mut output, "javascript:alert('xss')").unwrap();
        assert_eq!(output, "#");
    }

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Hello world")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let html = render(&arena, root, 0);
        assert!(html.contains("<p>Hello world</p>"), "Expected <p>Hello world</p> in {}", html);
    }

    #[test]
    fn test_render_emph() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(NodeValue::Emph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("emphasized")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, emph, text);

        let html = render(&arena, root, 0);
        assert!(html.contains("<em>emphasized</em>"));
    }

    #[test]
    fn test_render_strong() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let strong = arena.alloc(Node::with_value(NodeValue::Strong));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("strong")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, strong, text);

        let html = render(&arena, root, 0);
        assert!(html.contains("<strong>strong</strong>"));
    }

    #[test]
    fn test_render_code() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let code = arena.alloc(Node::with_value(NodeValue::Code(Box::new(NodeCode {
            num_backticks: 1,
            literal: "code".to_string(),
        }))));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let html = render(&arena, root, 0);
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn test_render_heading() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
            level: 2,
            setext: false,
            closed: false,
        })));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Title")));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let html = render(&arena, root, 0);
        assert!(html.contains("<h2>Title</h2>"));
    }

    #[test]
    fn test_render_link() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url: "https://example.com".to_string(),
            title: "".to_string(),
        }))));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("link")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, link, text);

        let html = render(&arena, root, 0);
        assert!(html.contains("<a href=\"https://example.com\">link</a>"));
    }

    #[test]
    fn test_render_blockquote() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let blockquote = arena.alloc(Node::with_value(NodeValue::BlockQuote));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Quote")));

        TreeOps::append_child(&mut arena, root, blockquote);
        TreeOps::append_child(&mut arena, blockquote, para);
        TreeOps::append_child(&mut arena, para, text);

        let html = render(&arena, root, 0);
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("<p>Quote</p>"));
        assert!(html.contains("</blockquote>"));
    }

    #[test]
    fn test_render_code_block() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let code_block = arena.alloc(Node::with_value(NodeValue::CodeBlock(Box::new(
            NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
                closed: true,
            },
        ))));

        TreeOps::append_child(&mut arena, root, code_block);

        let html = render(&arena, root, 0);
        assert!(html.contains("<pre><code class=\"language-rust\">"));
        assert!(html.contains("fn main() {}"));
        assert!(html.contains("</code></pre>"));
    }

    #[test]
    fn test_render_bullet_list() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let list = arena.alloc(Node::with_value(NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            delimiter: crate::nodes::ListDelimType::Period,
            start: 1,
            tight: true,
            bullet_char: b'-',
            marker_offset: 0,
            padding: 2,
            is_task_list: false,
        })));
        let item = arena.alloc(Node::with_value(NodeValue::Item(NodeList::default())));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("Item")));

        TreeOps::append_child(&mut arena, root, list);
        TreeOps::append_child(&mut arena, list, item);
        TreeOps::append_child(&mut arena, item, para);
        TreeOps::append_child(&mut arena, para, text);

        let html = render(&arena, root, 0);
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>"));
        assert!(html.contains("Item"));
        assert!(html.contains("</li>"));
        assert!(html.contains("</ul>"));
    }

    #[test]
    fn test_render_thematic_break() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let hr = arena.alloc(Node::with_value(NodeValue::ThematicBreak));

        TreeOps::append_child(&mut arena, root, hr);

        let html = render(&arena, root, 0);
        assert!(html.contains("<hr />"));
    }

    #[test]
    fn test_escape_href_blocks_javascript() {
        let result = escape_href_str("javascript:alert('xss')");
        assert_eq!(result, "#");

        let result = escape_href_str("JAVASCRIPT:alert('xss')");
        assert_eq!(result, "#");

        let result = escape_href_str("JavaScript:alert('xss')");
        assert_eq!(result, "#");
    }

    #[test]
    fn test_escape_href_blocks_vbscript() {
        let result = escape_href_str("vbscript:msgbox('xss')");
        assert_eq!(result, "#");
    }

    #[test]
    fn test_escape_href_blocks_file_protocol() {
        let result = escape_href_str("file:///etc/passwd");
        assert_eq!(result, "#");
    }

    #[test]
    fn test_escape_href_allows_safe_urls() {
        let result = escape_href_str("https://example.com");
        assert_eq!(result, "https://example.com");

        let result = escape_href_str("http://example.com/path?query=value");
        assert_eq!(result, "http://example.com/path?query=value");
    }

    #[test]
    fn test_escape_href_escapes_special_chars() {
        let result = escape_href_str("https://example.com?a=1&b=2");
        assert_eq!(result, "https://example.com?a=1&amp;b=2");

        let result = escape_href_str("https://example.com/<script>");
        assert_eq!(result, "https://example.com/&lt;script&gt;");

        let result = escape_href_str("https://example.com/\"quoted\"");
        assert_eq!(result, "https://example.com/&quot;quoted&quot;");

        let result = escape_href_str("https://example.com/path'");
        assert_eq!(result, "https://example.com/path&#x27;");

        let result = escape_href_str("https://example.com/`backtick`");
        assert_eq!(result, "https://example.com/&#x60;backtick&#x60;");
    }

    #[test]
    fn test_render_link_with_unsafe_url() {
        let mut arena = NodeArena::new();
        let root = arena.alloc(Node::with_value(NodeValue::Document));
        let para = arena.alloc(Node::with_value(NodeValue::Paragraph));
        let link = arena.alloc(Node::with_value(NodeValue::Link(Box::new(NodeLink {
            url: "javascript:alert('xss')".to_string(),
            title: "".to_string(),
        }))));
        let text = arena.alloc(Node::with_value(NodeValue::make_text("click me")));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, link, text);

        let html = render(&arena, root, 0);
        assert!(
            html.contains("href=\"#\""),
            "Unsafe URL should be replaced with #"
        );
        assert!(
            !html.contains("javascript:"),
            "javascript: should not appear in output"
        );
    }

    // Helper function for tests
    fn escape_href_str(url: &str) -> String {
        let mut output = String::new();
        escape_href(&mut output, url).unwrap();
        output
    }

    fn escape_html_str(text: &str) -> String {
        let mut output = String::new();
        escape_html(&mut output, text).unwrap();
        output
    }
}
