//! Trait definitions for options callbacks.
//!
//! This module defines traits for custom callbacks used in parsing and rendering,
//! such as URL rewriting and broken link resolution.

use std::fmt::{self, Debug, Formatter};

/// Trait for link and image URL rewrite extensions.
pub trait URLRewriter: Send + Sync {
    /// Converts the given URL from Markdown to its representation when output as HTML.
    fn rewrite(&self, url: &str) -> String;
}

impl Debug for dyn URLRewriter + '_ {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        formatter.write_str("<dyn URLRewriter>")
    }
}

impl<F> URLRewriter for F
where
    F: Fn(&str) -> String + Send + Sync,
{
    fn rewrite(&self, url: &str) -> String {
        self(url)
    }
}

/// The type of the callback used when a reference link is encountered with no
/// matching reference.
///
/// The details of the broken reference are passed in the
/// [`BrokenLinkReference`] argument. If a [`ResolvedReference`] is returned, it
/// is used as the link; otherwise, no link is made and the reference text is
/// preserved in its entirety.
pub trait BrokenLinkCallback: Send + Sync {
    /// Potentially resolve a single broken link reference.
    fn resolve(
        &self,
        broken_link_reference: BrokenLinkReference,
    ) -> Option<ResolvedReference>;
}

impl Debug for dyn BrokenLinkCallback + '_ {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        formatter.write_str("<dyn BrokenLinkCallback>")
    }
}

impl<F> BrokenLinkCallback for F
where
    F: Fn(BrokenLinkReference) -> Option<ResolvedReference> + Send + Sync,
{
    fn resolve(
        &self,
        broken_link_reference: BrokenLinkReference,
    ) -> Option<ResolvedReference> {
        self(broken_link_reference)
    }
}

/// Struct to the broken link callback, containing details on the link reference
/// which failed to find a match.
#[derive(Debug)]
pub struct BrokenLinkReference<'l> {
    /// The normalized reference link label. Unicode case folding is applied.
    pub normalized: &'l str,
    /// The original text in the link label.
    pub original: &'l str,
}

/// A reference link's resolved details.
#[derive(Clone, Debug)]
pub struct ResolvedReference {
    /// The destination URL of the reference link.
    pub url: String,
    /// The text of the link.
    pub title: String,
}
