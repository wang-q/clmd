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

mod context;

pub use context::{escape_href, escape_html, is_safe_url, Context};

use std::fmt::{self, Write};

use crate::nodes::{
    AstNode, LineColumn, ListType, NodeCode, NodeCodeBlock, NodeDescriptionItem,
    NodeFootnoteDefinition, NodeFootnoteReference, NodeHeading, NodeHtmlBlock, NodeLink,
    NodeList, NodeMath, NodeTable, NodeTaskItem, NodeValue, SourcePos, TableAlignment,
};
use crate::parser::options::{Options, Plugins};

/// Child rendering mode for formatters.
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
///
/// # Arguments
///
/// * `root` - The root node of the AST
/// * `options` - Rendering options
/// * `output` - The output buffer to write to
///
/// # Returns
///
/// A `fmt::Result` indicating success or failure
///
/// # Example
///
/// ```
/// use clmd::{Arena, parse_document, Options, format_html};
/// use std::fmt::Write;
///
/// let arena = Arena::new();
/// let options = Options::default();
/// let root = parse_document(&arena, "# Hello\n\nWorld", &options);
///
/// let mut html = String::new();
/// format_html(root, &options, &mut html).unwrap();
///
/// assert!(html.contains("<h1>"));
/// ```
pub fn format_document(
    root: &AstNode<'_>,
    options: &Options,
    output: &mut dyn Write,
) -> fmt::Result {
    format_document_with_plugins(root, options, output, &Plugins::default())
}

/// Formats an AST as HTML with plugins.
///
/// # Arguments
///
/// * `root` - The root node of the AST
/// * `options` - Rendering options
/// * `output` - The output buffer to write to
/// * `plugins` - Plugins for customizing rendering
///
/// # Returns
///
/// A `fmt::Result` indicating success or failure
pub fn format_document_with_plugins(
    root: &AstNode<'_>,
    options: &Options,
    output: &mut dyn Write,
    plugins: &Plugins<'_>,
) -> fmt::Result {
    let mut context = Context::new(output, options, plugins);

    // Use an iterative approach with a work stack
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
                    ChildRendering::Skip => {
                        // Should never be pushed with Skip
                        unreachable!()
                    }
                };

                if !matches!(new_cr, ChildRendering::Skip) {
                    // Push children in reverse order for depth-first traversal
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
        // Document
        NodeValue::Document => Ok(ChildRendering::HTML),

        // Block elements
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

        // Inline elements
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

        // GFM extensions
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

        // Other extensions
        NodeValue::Math(math) => render_math(context, node, entering, math),

        _ => Ok(ChildRendering::HTML),
    }
}

// ============================================================================
// Block element renderers
// ============================================================================

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
    node: &AstNode<'_>,
    entering: bool,
    code_block: &NodeCodeBlock,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.cr()?;

        // Check for syntax highlighter plugin
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
        } else if context.options.render.r#unsafe {
            context.write_str(&html_block.literal)?;
        } else {
            context.write_str("<!-- raw HTML omitted -->")?;
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
    // Check if we're in a tight list
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

// ============================================================================
// Inline element renderers
// ============================================================================

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
        } else if context.options.render.r#unsafe {
            context.write_str(html)?;
        } else {
            context.write_str("<!-- raw HTML omitted -->")?;
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
    node: &AstNode<'_>,
    entering: bool,
    link: &NodeLink,
) -> Result<ChildRendering, fmt::Error> {
    if entering {
        context.write_str("<img src=\"")?;
        escape_href(context, &link.url)?;
        context.write_str("\" alt=\"")?;
        // Return Plain to collect alt text
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

// ============================================================================
// GFM extension renderers
// ============================================================================

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
    // Determine if we're in a header row
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
    node: &AstNode<'_>,
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

// ============================================================================
// Other extension renderers
// ============================================================================

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

// ============================================================================
// Helper functions
// ============================================================================

/// Render sourcepos attribute if enabled.
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
            // Recursively collect from children
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                collect_text_append(child, output);
                child_opt = child.next_sibling();
            }
        }
    }
}

// ============================================================================
// Legacy compatibility
// ============================================================================

/// Render an arena_tree node to HTML (legacy compatibility).
///
/// This function is kept for backward compatibility.
/// New code should use [`format_document`] instead.
pub fn render_from_node<'a>(root: crate::nodes::Node<'a>, _options: u32) -> String {
    use crate::parser::options::Options;

    let options = Options::default();
    let mut output = String::new();

    // Create a wrapper to adapt between the two node types
    // This is a temporary bridge
    format_document_legacy(&options, &mut output, root);
    output
}

fn format_document_legacy(
    options: &Options,
    output: &mut dyn Write,
    root: crate::nodes::Node<'_>,
) -> fmt::Result {
    // Simple recursive rendering for legacy compatibility
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
            // Render children
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
            // For other nodes, just render children
            let mut child_opt = node.first_child();
            while let Some(child) = child_opt {
                render_node_legacy(options, output, child)?;
                child_opt = child.next_sibling();
            }
        }
    }

    Ok(())
}
