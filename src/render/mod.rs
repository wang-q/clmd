//! Rendering modules for Arena-based AST
//!
//! This module provides output generation for documents parsed using the Arena-based parser.
//! Supported formats: HTML, XML, CommonMark, LaTeX, and Man page.
//!
//! # Overview
//!
//! Each renderer traverses the AST and generates output in its respective format:
//!
//! - **HTML**: Web-ready markup
//! - **XML**: Structured data representation
//! - **CommonMark**: Round-trip Markdown format
//! - **LaTeX**: Typesetting for academic documents
//! - **Man**: Unix manual page format
//!
//! # Example
//!
//! ```ignore
//! use clmd::{markdown_to_html, parse::options::Options};
//!
//! let options = Options::default();
//! let html = markdown_to_html("# Hello\n\nWorld", &options);
//! assert!(html.contains("<h1>Hello</h1>"));
//! assert!(html.contains("<p>World</p>"));
//! ```

pub mod commonmark;
pub mod format;
pub mod renderer;

/// Deprecated: Use `format::html` instead.
#[deprecated(since = "0.2.0", note = "Use `format::html` instead")]
pub mod html {
    pub use crate::render::format::html::*;
}

/// Deprecated: Use `format::xml` instead.
#[deprecated(since = "0.2.0", note = "Use `format::xml` instead")]
pub mod xml {
    pub use crate::render::format::xml::*;
}

/// Deprecated: Use `format::latex` instead.
#[deprecated(since = "0.2.0", note = "Use `format::latex` instead")]
pub mod latex {
    pub use crate::render::format::latex::*;
}

/// Deprecated: Use `format::man` instead.
#[deprecated(since = "0.2.0", note = "Use `format::man` instead")]
pub mod man {
    pub use crate::render::format::man::*;
}

/// Deprecated: Use `format::typst` instead.
#[deprecated(since = "0.2.0", note = "Use `format::typst` instead")]
pub mod typst {
    pub use crate::render::format::typst::*;
}

/// Deprecated: Use `format::docx` instead.
#[deprecated(since = "0.2.0", note = "Use `format::docx` instead")]
pub mod docx {
    pub use crate::render::format::docx::*;
}

/// Deprecated: Use `format::pdf` instead.
#[deprecated(since = "0.2.0", note = "Use `format::pdf` instead")]
pub mod pdf {
    pub use crate::render::format::pdf::*;
}

// Re-export renderer types
pub use renderer::{
    render, render_to_commonmark, render_to_html, render_to_latex, render_to_man,
    render_to_xml, OutputFormat, Renderer,
};

// Re-export escape_html from html_utils for backward compatibility
pub use crate::text::html_utils::escape_html;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\"test\""), "&quot;test&quot;");
    }
}
