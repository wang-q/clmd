//! PDF writer (not yet implemented)
//!
//! This module is reserved for future PDF output generation.
//!
//! # Status: Not Implemented
//!
//! PDF output is not yet implemented. To add PDF support, one of these
//! approaches could be used:
//!
//! 1. **Direct PDF generation**: Add `printpdf` or `genpdf` crate dependency
//!    - Implement proper PDF structure generation
//!    - Add font embedding
//!    - Handle images and complex layouts
//!
//! 2. **External tool integration**: Use external tools like:
//!    - `wkhtmltopdf` - HTML to PDF via wkhtmltopdf
//!    - `pandoc` - Convert to PDF via pandoc
//!    - `weasyprint` - HTML/CSS to PDF
//!
//! 3. **LaTeX intermediate**: Convert to LaTeX first, then use LaTeX to PDF

use crate::core::arena::{NodeArena, NodeId};
use crate::core::error::{ClmdError, ClmdResult};
use crate::options::WriterOptions;

/// Write a document as PDF.
///
/// # Errors
///
/// Always returns an error as PDF output is not yet implemented.
pub fn write_pdf(
    _arena: &NodeArena,
    _root: NodeId,
    _options: &WriterOptions,
) -> ClmdResult<String> {
    Err(ClmdError::not_implemented(
        "PDF output is not yet implemented. \
         Consider using LaTeX output and converting with pdflatex, \
         or use an external tool like wkhtmltopdf or pandoc.",
    ))
}
