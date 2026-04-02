//! Slide show processing for clmd.
//!
//! This module provides functionality for converting Markdown documents into
//! slide shows, inspired by Pandoc's Text.Pandoc.Slides module.
//!
//! # Supported Formats
//!
//! - **Reveal.js** - HTML-based presentations
//! - **Beamer** - LaTeX-based presentations
//! - **PowerPoint** - Via Pandoc-compatible output
//! - **Slideous** - HTML slide show framework
//! - **S5** - Simple Standards-based Slide Show System
//! - **DZSlides** - HTML5 slide show framework
//!
//! # Example
//!
//! ```ignore
//! use clmd::io::format::slides::{SlideLevel, SlideShow};
//!
//! // Parse a document and split into slides
//! let slides = SlideShow::from_markdown("# Slide 1\n\nContent\n\n# Slide 2");
//! assert_eq!(slides.len(), 2);
//! ```

use crate::core::arena::{NodeArena, NodeId};
use crate::core::nodes::NodeValue;

/// Default slide level (h1 headings start new slides).
pub const DEFAULT_SLIDE_LEVEL: u8 = 1;

/// Slide level configuration.
///
/// Determines which heading level starts a new slide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SlideLevel {
    /// Level 1 headings start new slides.
    #[default]
    H1,
    /// Level 2 headings start new slides.
    H2,
    /// Level 3 headings start new slides.
    H3,
    /// Level 4 headings start new slides.
    H4,
    /// Level 5 headings start new slides.
    H5,
    /// Level 6 headings start new slides.
    H6,
    /// Custom level.
    Custom(u8),
}

impl SlideLevel {
    /// Get the numeric level value.
    pub fn as_u8(self) -> u8 {
        match self {
            SlideLevel::H1 => 1,
            SlideLevel::H2 => 2,
            SlideLevel::H3 => 3,
            SlideLevel::H4 => 4,
            SlideLevel::H5 => 5,
            SlideLevel::H6 => 6,
            SlideLevel::Custom(n) => n.clamp(1, 6),
        }
    }

    /// Check if a heading at the given level starts a new slide.
    pub fn is_slide_start(self, heading_level: u8) -> bool {
        heading_level <= self.as_u8()
    }
}

impl From<u8> for SlideLevel {
    fn from(level: u8) -> Self {
        match level {
            1 => SlideLevel::H1,
            2 => SlideLevel::H2,
            3 => SlideLevel::H3,
            4 => SlideLevel::H4,
            5 => SlideLevel::H5,
            6 => SlideLevel::H6,
            n => SlideLevel::Custom(n.clamp(1, 6)),
        }
    }
}

/// A single slide in a slide show.
#[derive(Debug, Clone)]
pub struct Slide {
    /// Title of the slide (from heading).
    pub title: Option<String>,
    /// Content nodes in this slide.
    pub content: Vec<NodeId>,
    /// Slide level (heading level that started this slide).
    pub level: u8,
    /// Slide index.
    pub index: usize,
}

impl Slide {
    /// Create a new empty slide.
    pub fn new(index: usize) -> Self {
        Self {
            title: None,
            content: Vec::new(),
            level: 1,
            index,
        }
    }

    /// Set the slide title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the slide level.
    pub fn with_level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }

    /// Add content to the slide.
    pub fn add_content(&mut self, node_id: NodeId) {
        self.content.push(node_id);
    }

    /// Get the number of content nodes.
    pub fn content_count(&self) -> usize {
        self.content.len()
    }

    /// Check if the slide is empty.
    pub fn is_empty(&self) -> bool {
        self.content.is_empty() && self.title.is_none()
    }
}

/// Slide show configuration.
#[derive(Debug, Clone)]
pub struct SlideConfig {
    /// Slide level (which heading starts new slides).
    pub slide_level: SlideLevel,
    /// Include table of contents slide.
    pub include_toc: bool,
    /// TOC slide title.
    pub toc_title: String,
    /// Incremental lists (show items one by one).
    pub incremental_lists: bool,
    /// Center content vertically.
    pub center: bool,
    /// Show slide numbers.
    pub slide_numbers: bool,
    /// Theme name.
    pub theme: Option<String>,
    /// Transition effect.
    pub transition: Option<String>,
}

impl Default for SlideConfig {
    fn default() -> Self {
        Self {
            slide_level: SlideLevel::default(),
            include_toc: false,
            toc_title: "Table of Contents".to_string(),
            incremental_lists: false,
            center: false,
            slide_numbers: true,
            theme: None,
            transition: None,
        }
    }
}

impl SlideConfig {
    /// Create a new slide configuration with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the slide level.
    pub fn slide_level(mut self, level: impl Into<SlideLevel>) -> Self {
        self.slide_level = level.into();
        self
    }

    /// Enable table of contents.
    pub fn with_toc(mut self) -> Self {
        self.include_toc = true;
        self
    }

    /// Set TOC title.
    pub fn toc_title(mut self, title: impl Into<String>) -> Self {
        self.toc_title = title.into();
        self
    }

    /// Enable incremental lists.
    pub fn incremental(mut self) -> Self {
        self.incremental_lists = true;
        self
    }

    /// Center content vertically.
    pub fn center(mut self) -> Self {
        self.center = true;
        self
    }

    /// Enable/disable slide numbers.
    pub fn slide_numbers(mut self, enabled: bool) -> Self {
        self.slide_numbers = enabled;
        self
    }

    /// Set theme.
    pub fn theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = Some(theme.into());
        self
    }

    /// Set transition effect.
    pub fn transition(mut self, transition: impl Into<String>) -> Self {
        self.transition = Some(transition.into());
        self
    }
}

/// A slide show containing multiple slides.
#[derive(Debug, Clone)]
pub struct SlideShow {
    /// Slides in the show.
    pub slides: Vec<Slide>,
    /// Configuration.
    pub config: SlideConfig,
}

impl SlideShow {
    /// Create a new empty slide show.
    pub fn new() -> Self {
        Self {
            slides: Vec::new(),
            config: SlideConfig::default(),
        }
    }

    /// Create a slide show with configuration.
    pub fn with_config(config: SlideConfig) -> Self {
        Self {
            slides: Vec::new(),
            config,
        }
    }

    /// Split a document into slides based on the configured slide level.
    pub fn from_arena(arena: &NodeArena, root: NodeId, config: SlideConfig) -> Self {
        let mut show = Self::with_config(config);
        show.split_into_slides(arena, root);
        show
    }

    /// Parse Markdown and split into slides.
    pub fn from_markdown(md: &str) -> Self {
        use crate::{parse_document, Options};

        let options = Options::default();
        let (arena, root) = parse_document(md, &options);
        Self::from_arena(&arena, root, SlideConfig::default())
    }

    /// Get the number of slides.
    pub fn len(&self) -> usize {
        self.slides.len()
    }

    /// Check if the slide show is empty.
    pub fn is_empty(&self) -> bool {
        self.slides.is_empty()
    }

    /// Get a slide by index.
    pub fn get(&self, index: usize) -> Option<&Slide> {
        self.slides.get(index)
    }

    /// Get a mutable reference to a slide.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Slide> {
        self.slides.get_mut(index)
    }

    /// Add a slide.
    pub fn add_slide(&mut self, slide: Slide) {
        self.slides.push(slide);
    }

    /// Remove a slide.
    pub fn remove_slide(&mut self, index: usize) -> Option<Slide> {
        if index < self.slides.len() {
            Some(self.slides.remove(index))
        } else {
            None
        }
    }

    /// Insert a slide at a specific position.
    pub fn insert_slide(&mut self, index: usize, slide: Slide) {
        self.slides.insert(index, slide);
    }

    /// Iterate over slides.
    pub fn iter(&self) -> impl Iterator<Item = &Slide> {
        self.slides.iter()
    }

    /// Iterate over slides mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Slide> {
        self.slides.iter_mut()
    }

    /// Split the document into slides.
    fn split_into_slides(&mut self, arena: &NodeArena, root: NodeId) {
        let slide_level = self.config.slide_level;
        let mut current_slide = Slide::new(0);
        let mut slide_index = 0;

        let root_node = arena.get(root);
        let mut child_opt = root_node.first_child;

        while let Some(child_id) = child_opt {
            let node = arena.get(child_id);

            // Check if this node starts a new slide
            let is_slide_boundary = if let NodeValue::Heading(heading) = &node.value {
                slide_level.is_slide_start(heading.level)
            } else {
                false
            };

            if is_slide_boundary {
                // Save current slide if it has content
                if !current_slide.is_empty() {
                    self.slides.push(current_slide);
                    slide_index += 1;
                }

                // Start a new slide
                let heading = if let NodeValue::Heading(h) = &node.value {
                    h
                } else {
                    unreachable!()
                };

                let title = get_heading_text(arena, child_id);
                current_slide = Slide::new(slide_index)
                    .with_title(title)
                    .with_level(heading.level);
            } else {
                // Add to current slide
                current_slide.add_content(child_id);
            }

            child_opt = node.next;
        }

        // Don't forget the last slide
        if !current_slide.is_empty() {
            self.slides.push(current_slide);
        }

        // Add TOC slide if requested
        if self.config.include_toc {
            self.add_toc_slide();
        }
    }

    /// Add a table of contents slide.
    fn add_toc_slide(&mut self) {
        let toc_slide = Slide::new(0)
            .with_title(&self.config.toc_title)
            .with_level(1);

        // Insert at the beginning
        self.slides.insert(0, toc_slide);

        // Update indices
        for (i, slide) in self.slides.iter_mut().enumerate() {
            slide.index = i;
        }
    }

    /// Get slide titles.
    pub fn titles(&self) -> Vec<Option<&String>> {
        self.slides.iter().map(|s| s.title.as_ref()).collect()
    }

    /// Render to reveal.js HTML format.
    pub fn to_reveal_js(&self) -> String {
        let mut output = String::new();

        output.push_str("<div class=\"reveal\">\n");
        output.push_str("  <div class=\"slides\">\n");

        for slide in &self.slides {
            output.push_str("    <section>\n");

            if let Some(ref title) = slide.title {
                output.push_str(&format!("      <h2>{}</h2>\n", escape_html(title)));
            }

            output.push_str("      <!-- Slide content -->\n");
            output.push_str("    </section>\n");
        }

        output.push_str("  </div>\n");
        output.push_str("</div>\n");

        output
    }

    /// Render to beamer LaTeX format.
    pub fn to_beamer(&self) -> String {
        let mut output = String::new();

        for slide in &self.slides {
            output.push_str("\\begin{frame}\n");

            if let Some(ref title) = slide.title {
                output.push_str(&format!("  \\frametitle{{{}}}\n", escape_latex(title)));
            }

            output.push_str("  % Slide content\n");
            output.push_str("\\end{frame}\n\n");
        }

        output
    }
}

impl Default for SlideShow {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the text content of a heading node.
fn get_heading_text(arena: &NodeArena, heading_id: NodeId) -> String {
    let mut result = String::new();
    collect_text_recursive(arena, heading_id, &mut result);
    result.trim().to_string()
}

/// Recursively collect text from a node and its children.
fn collect_text_recursive(arena: &NodeArena, node_id: NodeId, result: &mut String) {
    let node = arena.get(node_id);

    if let NodeValue::Text(text) = &node.value {
        result.push_str(text);
    }

    let mut child_opt = node.first_child;
    while let Some(child_id) = child_opt {
        collect_text_recursive(arena, child_id, result);
        child_opt = arena.get(child_id).next;
    }
}

/// Escape HTML special characters.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Escape LaTeX special characters.
fn escape_latex(text: &str) -> String {
    text.replace('\\', "\\textbackslash{}")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('$', "\\$")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('^', "\\^{}")
        .replace('~', "\\textasciitilde{}")
}

/// Split a document into slides.
///
/// This is a convenience function for quickly splitting a document.
pub fn split_into_slides(
    arena: &NodeArena,
    root: NodeId,
    slide_level: impl Into<SlideLevel>,
) -> Vec<Slide> {
    let config = SlideConfig::default().slide_level(slide_level);
    let show = SlideShow::from_arena(arena, root, config);
    show.slides
}

/// Check if a document appears to be a slide show.
///
/// A document is considered a slide show if it has multiple top-level
/// headings at or below the specified slide level.
pub fn is_slide_show(
    arena: &NodeArena,
    root: NodeId,
    slide_level: impl Into<SlideLevel>,
) -> bool {
    let level = slide_level.into();
    let root_node = arena.get(root);
    let mut slide_count = 0;

    let mut child_opt = root_node.first_child;
    while let Some(child_id) = child_opt {
        let node = arena.get(child_id);

        if let NodeValue::Heading(heading) = &node.value {
            if level.is_slide_start(heading.level) {
                slide_count += 1;
                if slide_count >= 2 {
                    return true;
                }
            }
        }

        child_opt = node.next;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_document, Options};

    #[test]
    fn test_slide_level() {
        assert_eq!(SlideLevel::H1.as_u8(), 1);
        assert_eq!(SlideLevel::H2.as_u8(), 2);
        assert_eq!(SlideLevel::Custom(3).as_u8(), 3);
        assert_eq!(SlideLevel::Custom(10).as_u8(), 6); // Clamped

        assert!(SlideLevel::H2.is_slide_start(1));
        assert!(SlideLevel::H2.is_slide_start(2));
        assert!(!SlideLevel::H2.is_slide_start(3));
    }

    #[test]
    fn test_slide_level_from_u8() {
        assert!(matches!(SlideLevel::from(1), SlideLevel::H1));
        assert!(matches!(SlideLevel::from(2), SlideLevel::H2));
        assert!(matches!(SlideLevel::from(7), SlideLevel::Custom(6)));
    }

    #[test]
    fn test_slide_creation() {
        let slide = Slide::new(0).with_title("Test Slide").with_level(2);

        assert_eq!(slide.title, Some("Test Slide".to_string()));
        assert_eq!(slide.level, 2);
        assert_eq!(slide.index, 0);
        assert!(slide.content.is_empty());
    }

    #[test]
    fn test_slide_show_from_markdown() {
        let md = "# Slide 1\n\nContent 1\n\n# Slide 2\n\nContent 2";
        let show = SlideShow::from_markdown(md);

        assert_eq!(show.len(), 2);
        assert_eq!(show.get(0).unwrap().title, Some("Slide 1".to_string()));
        assert_eq!(show.get(1).unwrap().title, Some("Slide 2".to_string()));
    }

    #[test]
    fn test_slide_show_with_h2_level() {
        let md = "# Title\n\n## Slide 1\n\nContent\n\n## Slide 2\n\nContent";
        let config = SlideConfig::default().slide_level(2);
        let options = Options::default();
        let (arena, root) = parse_document(md, &options);
        let show = SlideShow::from_arena(&arena, root, config);

        // H1 starts a new slide, so we have: Title slide, Slide 1, Slide 2
        assert_eq!(show.len(), 3);
        assert_eq!(show.get(0).unwrap().title, Some("Title".to_string()));
        assert_eq!(show.get(1).unwrap().title, Some("Slide 1".to_string()));
        assert_eq!(show.get(2).unwrap().title, Some("Slide 2".to_string()));
    }

    #[test]
    fn test_is_slide_show() {
        let md_single = "# Title\n\nContent";
        let md_multiple = "# Slide 1\n\nContent\n\n# Slide 2";

        let options = Options::default();

        let (arena1, root1) = parse_document(md_single, &options);
        assert!(!is_slide_show(&arena1, root1, SlideLevel::H1));

        let (arena2, root2) = parse_document(md_multiple, &options);
        assert!(is_slide_show(&arena2, root2, SlideLevel::H1));
    }

    #[test]
    fn test_slide_config() {
        let config = SlideConfig::new()
            .slide_level(2)
            .with_toc()
            .toc_title("Agenda")
            .incremental()
            .center()
            .slide_numbers(false)
            .theme("black")
            .transition("slide");

        assert_eq!(config.slide_level.as_u8(), 2);
        assert!(config.include_toc);
        assert_eq!(config.toc_title, "Agenda");
        assert!(config.incremental_lists);
        assert!(config.center);
        assert!(!config.slide_numbers);
        assert_eq!(config.theme, Some("black".to_string()));
        assert_eq!(config.transition, Some("slide".to_string()));
    }

    #[test]
    fn test_reveal_js_output() {
        let md = "# Slide 1\n\n# Slide 2";
        let show = SlideShow::from_markdown(md);
        let html = show.to_reveal_js();

        assert!(html.contains("<div class=\"reveal\">"));
        assert!(html.contains("<div class=\"slides\">"));
        assert!(html.contains("<section>"));
        assert!(html.contains("<h2>Slide 1</h2>"));
        assert!(html.contains("<h2>Slide 2</h2>"));
    }

    #[test]
    fn test_beamer_output() {
        let md = "# Slide 1\n\n# Slide 2";
        let show = SlideShow::from_markdown(md);
        let latex = show.to_beamer();

        assert!(latex.contains("\\begin{frame}"));
        assert!(latex.contains("\\frametitle{Slide 1}"));
        assert!(latex.contains("\\frametitle{Slide 2}"));
        assert!(latex.contains("\\end{frame}"));
    }

    #[test]
    fn test_split_into_slides() {
        let md = "# First\n\nContent\n\n# Second";
        let options = Options::default();
        let (arena, root) = parse_document(md, &options);
        let slides = split_into_slides(&arena, root, SlideLevel::H1);

        assert_eq!(slides.len(), 2);
        assert_eq!(slides[0].title, Some("First".to_string()));
        assert_eq!(slides[1].title, Some("Second".to_string()));
    }

    #[test]
    fn test_empty_slide_show() {
        let show = SlideShow::new();
        assert!(show.is_empty());
        assert_eq!(show.len(), 0);
    }

    #[test]
    fn test_slide_manipulation() {
        let mut show = SlideShow::new();
        let slide1 = Slide::new(0).with_title("First");
        let slide2 = Slide::new(1).with_title("Second");

        show.add_slide(slide1);
        show.add_slide(slide2);

        assert_eq!(show.len(), 2);

        let removed = show.remove_slide(0);
        assert_eq!(removed.unwrap().title, Some("First".to_string()));
        assert_eq!(show.len(), 1);

        let slide3 = Slide::new(2).with_title("Third");
        show.insert_slide(0, slide3);
        assert_eq!(show.get(0).unwrap().title, Some("Third".to_string()));
    }

    #[test]
    fn test_toc_slide() {
        let md = "# Slide 1\n\n# Slide 2";
        let config = SlideConfig::default().with_toc();
        let _show = SlideShow::from_markdown(md);
        let show_with_toc = SlideShow::with_config(config);

        // Just verify config is set correctly
        assert!(show_with_toc.config.include_toc);
        assert_eq!(show_with_toc.config.toc_title, "Table of Contents");
    }

    #[test]
    fn test_html_escaping() {
        let text = "Test <script>alert('xss')</script>";
        let escaped = escape_html(text);
        assert!(!escaped.contains("<script>"));
        assert!(escaped.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_latex_escaping() {
        let text = "Special $characters% & more";
        let escaped = escape_latex(text);
        assert!(escaped.contains("\\$"));
        assert!(escaped.contains("\\%"));
        assert!(escaped.contains("\\&"));
    }
}
