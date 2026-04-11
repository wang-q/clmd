//! Code block rendering for HTML renderer

use crate::core::arena::NodeId;
use crate::core::nodes::NodeValue;
use crate::text::html_utils::escape_html;
use std::fmt::Write;

use crate::render::html::renderer::HtmlRenderer;

impl<'a> HtmlRenderer<'a> {
    pub(crate) fn render_code_block(&mut self, node_id: NodeId) {
        let node = self.arena.get(node_id);
        if let NodeValue::CodeBlock(code_block) = &node.value {
            self.in_code_block = true;

            // Parse language from info string
            let lang = if !code_block.info.is_empty() {
                code_block.info.split_whitespace().next().unwrap_or("")
            } else {
                ""
            };

            // Default rendering without syntax highlighting
            self.lit("<pre><code");
            self.render_sourcepos(node_id);
            if !lang.is_empty() {
                write!(self.output, " class=\"language-{}\"", escape_html(lang))
                    .expect("write to String cannot fail");
            }
            self.lit(">");

            // Write code content (escaped)
            self.lit(&escape_html(&code_block.literal));

            self.lit("</code></pre>");

            self.lit("\n");
            self.in_code_block = false;
        }
    }
}
