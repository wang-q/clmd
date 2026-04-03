//! Footnote rendering for HTML renderer

use crate::core::arena::NodeId;
use crate::core::nodes::NodeValue;
use crate::text::html_utils::escape_html;

use crate::render::html::renderer::HtmlRenderer;

impl<'a> HtmlRenderer<'a> {
    pub(crate) fn render_footnotes(&mut self) {
        self.lit("<section class=\"footnotes\">");
        self.lit("\n");
        self.lit("<ol>");
        self.lit("\n");

        // Collect footnotes to avoid borrow issues
        let footnotes: Vec<(String, NodeId)> = self.footnotes.clone();

        for (name, def_id) in footnotes {
            self.lit(&format!("<li id=\"fn-{}\">", escape_html(&name)));
            // Render footnote content
            self.render_node(def_id, true);
            self.lit(&format!(
                " <a href=\"#fnref-{}\" class=\"footnote-backref\">↩</a>",
                escape_html(&name)
            ));
            self.lit("</li>");
            self.lit("\n");
        }

        self.lit("</ol>");
        self.lit("\n");
        self.lit("</section>");
        self.lit("\n");
    }

    pub(crate) fn find_footnote_def(&self, name: &str) -> Option<NodeId> {
        // Search the arena for the footnote definition with matching name
        for (id, node) in self.arena.iter() {
            if let NodeValue::FootnoteDefinition(def) = &node.value {
                if def.name == name {
                    return Some(id);
                }
            }
        }
        None
    }
}
