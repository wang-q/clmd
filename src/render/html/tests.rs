//! Unit tests for HTML renderer

#[cfg(test)]
mod tests {
    use crate::core::arena::{Node, NodeArena, TreeOps};
    use crate::core::nodes::{
        ListType, NodeCode, NodeCodeBlock, NodeHeading, NodeLink, NodeList,
        NodeThematicBreak,
    };
    use crate::options::Options;
    use crate::render::html::render;
    use crate::text::html_utils::escape_html;

    #[test]
    fn test_escape_html_util() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\"test\""), "&quot;test&quot;");
    }

    #[test]
    fn test_render_paragraph() {
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let para =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::make_text("Hello world"),
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, text);

        let html = render(&arena, root, &Options::default());
        println!("HTML output: {:?}", html);
        assert!(
            html.contains("<p>Hello world</p>"),
            "Expected <p>Hello world</p> in {}",
            html
        );
    }

    #[test]
    fn test_render_emph() {
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let para =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Paragraph));
        let emph = arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Emph));
        let text = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::make_text("emphasized"),
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, emph);
        TreeOps::append_child(&mut arena, emph, text);

        let html = render(&arena, root, &Options::default());
        assert!(html.contains("<em>emphasized</em>"));
    }

    #[test]
    fn test_render_strong() {
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let para =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Paragraph));
        let strong =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Strong));
        let text = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::make_text("strong"),
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, strong);
        TreeOps::append_child(&mut arena, strong, text);

        let html = render(&arena, root, &Options::default());
        assert!(html.contains("<strong>strong</strong>"));
    }

    #[test]
    fn test_render_code() {
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let para =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Paragraph));
        let code = arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Code(
            Box::new(NodeCode {
                num_backticks: 1,
                literal: "code".to_string(),
            }),
        )));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, code);

        let html = render(&arena, root, &Options::default());
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn test_render_heading() {
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let heading = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::Heading(NodeHeading {
                level: 2,
                setext: false,
                closed: false,
            }),
        ));
        let text = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::make_text("Title"),
        ));

        TreeOps::append_child(&mut arena, root, heading);
        TreeOps::append_child(&mut arena, heading, text);

        let html = render(&arena, root, &Options::default());
        assert!(html.contains("<h2>Title</h2>"));
    }

    #[test]
    fn test_render_link() {
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let para =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Paragraph));
        let link = arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Link(
            Box::new(NodeLink {
                url: "https://example.com".to_string(),
                title: "".to_string(),
            }),
        )));
        let text = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::make_text("link"),
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, link, text);

        let html = render(&arena, root, &Options::default());
        assert!(html.contains("<a href=\"https://example.com\">link</a>"));
    }

    #[test]
    fn test_render_blockquote() {
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let blockquote =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::BlockQuote));
        let para =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::make_text("Quote"),
        ));

        TreeOps::append_child(&mut arena, root, blockquote);
        TreeOps::append_child(&mut arena, blockquote, para);
        TreeOps::append_child(&mut arena, para, text);

        let html = render(&arena, root, &Options::default());
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("<p>Quote</p>"));
        assert!(html.contains("</blockquote>"));
    }

    #[test]
    fn test_render_code_block() {
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let code_block = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::CodeBlock(Box::new(NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
                closed: true,
            })),
        ));

        TreeOps::append_child(&mut arena, root, code_block);

        let html = render(&arena, root, &Options::default());
        assert!(html.contains("<pre><code class=\"language-rust\">"));
        assert!(html.contains("fn main() {}"));
        assert!(html.contains("</code></pre>"));
    }

    #[test]
    fn test_render_bullet_list() {
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let list = arena.alloc(Node::with_value(crate::core::nodes::NodeValue::List(
            NodeList {
                list_type: ListType::Bullet,
                delimiter: crate::core::nodes::ListDelimType::Period,
                start: 1,
                tight: true,
                bullet_char: b'-',
                marker_offset: 0,
                padding: 2,
                is_task_list: false,
            },
        )));
        let item = arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Item(
            NodeList::default(),
        )));
        let para =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::make_text("Item"),
        ));

        TreeOps::append_child(&mut arena, root, list);
        TreeOps::append_child(&mut arena, list, item);
        TreeOps::append_child(&mut arena, item, para);
        TreeOps::append_child(&mut arena, para, text);

        let html = render(&arena, root, &Options::default());
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>"));
        assert!(html.contains("Item"));
        assert!(html.contains("</li>"));
        assert!(html.contains("</ul>"));
    }

    #[test]
    fn test_render_thematic_break() {
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let hr = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::ThematicBreak(NodeThematicBreak::default()),
        ));

        TreeOps::append_child(&mut arena, root, hr);

        let html = render(&arena, root, &Options::default());
        assert!(html.contains("<hr />"));
    }

    // Security tests for XSS prevention
    #[test]
    fn test_escape_href_blocks_javascript() {
        use crate::render::html::escaping::escape_href;

        // javascript: protocol should be blocked
        let result = escape_href("javascript:alert('xss')");
        assert_eq!(result, "#");

        // Case variations
        let result = escape_href("JAVASCRIPT:alert('xss')");
        assert_eq!(result, "#");

        let result = escape_href("JavaScript:alert('xss')");
        assert_eq!(result, "#");
    }

    #[test]
    fn test_escape_href_blocks_vbscript() {
        use crate::render::html::escaping::escape_href;

        let result = escape_href("vbscript:msgbox('xss')");
        assert_eq!(result, "#");
    }

    #[test]
    fn test_escape_href_blocks_file_protocol() {
        use crate::render::html::escaping::escape_href;

        let result = escape_href("file:///etc/passwd");
        assert_eq!(result, "#");
    }

    #[test]
    fn test_escape_href_allows_safe_urls() {
        use crate::render::html::escaping::escape_href;

        // HTTP/HTTPS should be allowed
        let result = escape_href("https://example.com");
        assert_eq!(result, "https://example.com");

        let result = escape_href("http://example.com/path?query=value");
        assert_eq!(result, "http://example.com/path?query=value");
    }

    #[test]
    fn test_escape_href_escapes_special_chars() {
        use crate::render::html::escaping::escape_href;

        // Special characters should be escaped
        let result = escape_href("https://example.com?a=1&b=2");
        assert_eq!(result, "https://example.com?a=1&amp;b=2");

        let result = escape_href("https://example.com/<script>");
        assert_eq!(result, "https://example.com/&lt;script&gt;");

        let result = escape_href("https://example.com/\"quoted\"");
        assert_eq!(result, "https://example.com/&quot;quoted&quot;");

        // Single quotes and backticks should be escaped for attribute context
        let result = escape_href("https://example.com/path'");
        assert_eq!(result, "https://example.com/path&#x27;");

        let result = escape_href("https://example.com/`backtick`");
        assert_eq!(result, "https://example.com/&#x60;backtick&#x60;");
    }

    #[test]
    fn test_render_link_with_unsafe_url() {
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let para =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Paragraph));
        let link = arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Link(
            Box::new(NodeLink {
                url: "javascript:alert('xss')".to_string(),
                title: "".to_string(),
            }),
        )));
        let text = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::make_text("click me"),
        ));

        TreeOps::append_child(&mut arena, root, para);
        TreeOps::append_child(&mut arena, para, link);
        TreeOps::append_child(&mut arena, link, text);

        let html = render(&arena, root, &Options::default());
        // Unsafe URL should be replaced with "#"
        assert!(
            html.contains("href=\"#\""),
            "Unsafe URL should be replaced with #"
        );
        assert!(
            !html.contains("javascript:"),
            "javascript: should not appear in output"
        );
    }

    // Sourcepos tests
    #[test]
    fn test_sourcepos_heading() {
        use crate::core::nodes::SourcePos;
        let mut options = Options::default();
        options.render.sourcepos = true;
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let mut heading =
            Node::with_value(crate::core::nodes::NodeValue::Heading(NodeHeading {
                level: 1,
                setext: false,
                closed: false,
            }));
        heading.source_pos = SourcePos::new(1, 1, 1, 7);
        let heading_id = arena.alloc(heading);
        let text = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::make_text("Hello"),
        ));

        TreeOps::append_child(&mut arena, root, heading_id);
        TreeOps::append_child(&mut arena, heading_id, text);

        let html = render(&arena, root, &options);
        assert!(html.contains("data-sourcepos=\"1:1-1:7\""));
        assert!(html.contains("<h1 data-sourcepos=\"1:1-1:7\">"));
    }

    #[test]
    fn test_sourcepos_paragraph() {
        use crate::core::nodes::SourcePos;
        let mut options = Options::default();
        options.render.sourcepos = true;
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let mut para = Node::with_value(crate::core::nodes::NodeValue::Paragraph);
        para.source_pos = SourcePos::new(2, 1, 2, 10);
        let para_id = arena.alloc(para);
        let text = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::make_text("Paragraph"),
        ));

        TreeOps::append_child(&mut arena, root, para_id);
        TreeOps::append_child(&mut arena, para_id, text);

        let html = render(&arena, root, &options);
        assert!(html.contains("<p data-sourcepos=\"2:1-2:10\">"));
    }

    #[test]
    fn test_sourcepos_blockquote() {
        use crate::core::nodes::SourcePos;
        let mut options = Options::default();
        options.render.sourcepos = true;
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let mut blockquote = Node::with_value(crate::core::nodes::NodeValue::BlockQuote);
        blockquote.source_pos = SourcePos::new(1, 1, 3, 5);
        let blockquote_id = arena.alloc(blockquote);
        let para =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::make_text("Quote"),
        ));

        TreeOps::append_child(&mut arena, root, blockquote_id);
        TreeOps::append_child(&mut arena, blockquote_id, para);
        TreeOps::append_child(&mut arena, para, text);

        let html = render(&arena, root, &options);
        assert!(html.contains("<blockquote data-sourcepos=\"1:1-3:5\">"));
    }

    #[test]
    fn test_sourcepos_list() {
        use crate::core::nodes::SourcePos;
        let mut options = Options::default();
        options.render.sourcepos = true;
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let mut list = Node::with_value(crate::core::nodes::NodeValue::List(NodeList {
            list_type: ListType::Bullet,
            delimiter: crate::core::nodes::ListDelimType::Period,
            start: 1,
            tight: true,
            bullet_char: b'-',
            marker_offset: 0,
            padding: 2,
            is_task_list: false,
        }));
        list.source_pos = SourcePos::new(1, 1, 3, 5);
        let list_id = arena.alloc(list);
        let mut item =
            Node::with_value(crate::core::nodes::NodeValue::Item(NodeList::default()));
        item.source_pos = SourcePos::new(1, 1, 1, 5);
        let item_id = arena.alloc(item);
        let para =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Paragraph));
        let text = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::make_text("Item"),
        ));

        TreeOps::append_child(&mut arena, root, list_id);
        TreeOps::append_child(&mut arena, list_id, item_id);
        TreeOps::append_child(&mut arena, item_id, para);
        TreeOps::append_child(&mut arena, para, text);

        let html = render(&arena, root, &options);
        assert!(html.contains("<ul data-sourcepos=\"1:1-3:5\">"));
        assert!(html.contains("<li data-sourcepos=\"1:1-1:5\">"));
    }

    #[test]
    fn test_sourcepos_code_block() {
        use crate::core::nodes::SourcePos;
        let mut options = Options::default();
        options.render.sourcepos = true;
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let mut code_block = Node::with_value(crate::core::nodes::NodeValue::CodeBlock(
            Box::new(NodeCodeBlock {
                fenced: true,
                fence_char: b'`',
                fence_length: 3,
                fence_offset: 0,
                info: "rust".to_string(),
                literal: "fn main() {}".to_string(),
                closed: true,
            }),
        ));
        code_block.source_pos = SourcePos::new(1, 1, 3, 3);
        let code_block_id = arena.alloc(code_block);

        TreeOps::append_child(&mut arena, root, code_block_id);

        let html = render(&arena, root, &options);
        assert!(html.contains("data-sourcepos=\"1:1-3:3\""));
        assert!(html.contains("<code data-sourcepos=\"1:1-3:3\""));
    }

    #[test]
    fn test_sourcepos_thematic_break() {
        use crate::core::nodes::SourcePos;
        let mut options = Options::default();
        options.render.sourcepos = true;
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let mut hr = Node::with_value(crate::core::nodes::NodeValue::ThematicBreak(
            NodeThematicBreak::default(),
        ));
        hr.source_pos = SourcePos::new(2, 1, 2, 3);
        let hr_id = arena.alloc(hr);

        TreeOps::append_child(&mut arena, root, hr_id);

        let html = render(&arena, root, &options);
        assert!(html.contains("<hr data-sourcepos=\"2:1-2:3\" />"));
    }

    #[test]
    fn test_sourcepos_disabled() {
        use crate::core::nodes::SourcePos;
        let mut options = Options::default();
        options.render.sourcepos = true;
        let mut arena = NodeArena::new();
        let root =
            arena.alloc(Node::with_value(crate::core::nodes::NodeValue::Document));
        let mut heading =
            Node::with_value(crate::core::nodes::NodeValue::Heading(NodeHeading {
                level: 1,
                setext: false,
                closed: false,
            }));
        heading.source_pos = SourcePos::new(1, 1, 1, 7);
        let heading_id = arena.alloc(heading);
        let text = arena.alloc(Node::with_value(
            crate::core::nodes::NodeValue::make_text("Hello"),
        ));

        TreeOps::append_child(&mut arena, root, heading_id);
        TreeOps::append_child(&mut arena, heading_id, text);

        // Render without sourcepos
        let html = render(&arena, root, &Options::default());
        assert!(!html.contains("data-sourcepos"));
        assert!(html.contains("<h1>"));
    }
}
