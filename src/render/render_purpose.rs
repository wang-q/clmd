//! Render purpose definitions
//!
//! This module defines the different purposes for rendering,
//! inspired by flexmark-java's RenderPurpose enum. This is primarily
//! used for translation workflows where content needs to be extracted,
/// translated, and then re-rendered.

/// Render purpose for controlling rendering behavior
///
/// The render purpose determines how content should be rendered,
/// especially in translation workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderPurpose {
    /// Normal formatting
    ///
    /// Standard formatting without any special translation handling.
    Format,

    /// Translation spans - extract translatable text
    ///
    /// In this mode, translatable text is extracted and replaced with
    /// placeholders. The extracted text is collected for translation.
    TranslationSpans,

    /// Translated spans - use translated placeholders
    ///
    /// In this mode, the placeholders are rendered, but the actual
    /// translated content is not yet inserted.
    TranslatedSpans,

    /// Translated - final rendering with translated content
    ///
    /// In this mode, the final translated content is rendered,
    /// replacing the placeholders.
    Translated,
}

impl RenderPurpose {
    /// Check if this is a formatting purpose (normal rendering)
    pub fn is_format(&self) -> bool {
        matches!(self, RenderPurpose::Format)
    }

    /// Check if this is a translation-related purpose
    pub fn is_translation(&self) -> bool {
        matches!(
            self,
            RenderPurpose::TranslationSpans
                | RenderPurpose::TranslatedSpans
                | RenderPurpose::Translated
        )
    }

    /// Check if text transformation is active
    ///
    /// Returns true for all translation-related purposes.
    pub fn is_transforming_text(&self) -> bool {
        matches!(
            self,
            RenderPurpose::TranslationSpans
                | RenderPurpose::TranslatedSpans
                | RenderPurpose::Translated
        )
    }

    /// Check if this is the translation extraction phase
    pub fn is_translation_spans(&self) -> bool {
        matches!(self, RenderPurpose::TranslationSpans)
    }

    /// Check if this is the translated spans phase
    pub fn is_translated_spans(&self) -> bool {
        matches!(self, RenderPurpose::TranslatedSpans)
    }

    /// Check if this is the final translated phase
    pub fn is_translated(&self) -> bool {
        matches!(self, RenderPurpose::Translated)
    }

    /// Get the display name for this purpose
    pub fn name(&self) -> &'static str {
        match self {
            RenderPurpose::Format => "Format",
            RenderPurpose::TranslationSpans => "TranslationSpans",
            RenderPurpose::TranslatedSpans => "TranslatedSpans",
            RenderPurpose::Translated => "Translated",
        }
    }

    /// Get the next purpose in the translation workflow
    ///
    /// Returns the next purpose in the sequence:
    /// Format -> Format (no change)
    /// TranslationSpans -> TranslatedSpans
    /// TranslatedSpans -> Translated
    /// Translated -> Translated (no change)
    pub fn next(&self) -> Self {
        match self {
            RenderPurpose::Format => RenderPurpose::Format,
            RenderPurpose::TranslationSpans => RenderPurpose::TranslatedSpans,
            RenderPurpose::TranslatedSpans => RenderPurpose::Translated,
            RenderPurpose::Translated => RenderPurpose::Translated,
        }
    }
}

impl Default for RenderPurpose {
    fn default() -> Self {
        RenderPurpose::Format
    }
}

/// Translation span information
///
/// Used to track translatable content during the translation workflow.
#[derive(Debug, Clone)]
pub struct TranslationSpan {
    /// Unique identifier for this span
    pub id: usize,
    /// The original text
    pub original_text: String,
    /// The translated text (if available)
    pub translated_text: Option<String>,
    /// The placeholder used during rendering
    pub placeholder: String,
    /// Whether this span is contextually isolated (e.g., link reference)
    pub is_isolated: bool,
}

impl TranslationSpan {
    /// Create a new translation span
    pub fn new(id: usize, original_text: impl Into<String>, placeholder_format: &str) -> Self {
        let original_text = original_text.into();
        let placeholder = placeholder_format.replace("{}", &id.to_string());
        Self {
            id,
            original_text,
            translated_text: None,
            placeholder,
            is_isolated: false,
        }
    }

    /// Create a new isolated translation span
    pub fn new_isolated(
        id: usize,
        original_text: impl Into<String>,
        placeholder_format: &str,
    ) -> Self {
        let mut span = Self::new(id, original_text, placeholder_format);
        span.is_isolated = true;
        span
    }

    /// Set the translated text
    pub fn set_translated(&mut self, translated: impl Into<String>) {
        self.translated_text = Some(translated.into());
    }

    /// Get the text to render based on the render purpose
    pub fn get_text(&self, purpose: RenderPurpose) -> &str {
        match purpose {
            RenderPurpose::Format => &self.original_text,
            RenderPurpose::TranslationSpans => &self.placeholder,
            RenderPurpose::TranslatedSpans => &self.placeholder,
            RenderPurpose::Translated => self
                .translated_text
                .as_deref()
                .unwrap_or(&self.original_text),
        }
    }

    /// Check if this span has been translated
    pub fn is_translated(&self) -> bool {
        self.translated_text.is_some()
    }
}

/// Translation span collection
///
/// Manages all translation spans during the rendering process.
#[derive(Debug, Clone, Default)]
pub struct TranslationSpanCollection {
    spans: Vec<TranslationSpan>,
    next_id: usize,
    placeholder_format: String,
}

impl TranslationSpanCollection {
    /// Create a new translation span collection
    pub fn new() -> Self {
        Self {
            spans: Vec::new(),
            next_id: 1,
            placeholder_format: "_{}_".to_string(),
        }
    }

    /// Create a new collection with a custom placeholder format
    pub fn with_placeholder_format(format: impl Into<String>) -> Self {
        Self {
            spans: Vec::new(),
            next_id: 1,
            placeholder_format: format.into(),
        }
    }

    /// Add a new translation span
    pub fn add_span(&mut self, original_text: impl Into<String>) -> &TranslationSpan {
        let id = self.next_id;
        self.next_id += 1;
        let span = TranslationSpan::new(id, original_text, &self.placeholder_format);
        self.spans.push(span);
        self.spans.last().unwrap()
    }

    /// Add a new isolated translation span
    pub fn add_isolated_span(&mut self, original_text: impl Into<String>) -> &TranslationSpan {
        let id = self.next_id;
        self.next_id += 1;
        let span = TranslationSpan::new_isolated(id, original_text, &self.placeholder_format);
        self.spans.push(span);
        self.spans.last().unwrap()
    }

    /// Get a span by ID
    pub fn get_span(&self, id: usize) -> Option<&TranslationSpan> {
        self.spans.iter().find(|s| s.id == id)
    }

    /// Get a mutable span by ID
    pub fn get_span_mut(&mut self, id: usize) -> Option<&mut TranslationSpan> {
        self.spans.iter_mut().find(|s| s.id == id)
    }

    /// Get all spans
    pub fn get_spans(&self) -> &[TranslationSpan] {
        &self.spans
    }

    /// Get all original texts
    pub fn get_original_texts(&self) -> Vec<&str> {
        self.spans.iter().map(|s| s.original_text.as_str()).collect()
    }

    /// Set translated texts
    ///
    /// The number of texts must match the number of spans.
    pub fn set_translated_texts(&mut self, texts: Vec<String>) -> Result<(), String> {
        if texts.len() != self.spans.len() {
            return Err(format!(
                "Expected {} translated texts, got {}",
                self.spans.len(),
                texts.len()
            ));
        }
        for (span, text) in self.spans.iter_mut().zip(texts) {
            span.set_translated(text);
        }
        Ok(())
    }

    /// Clear all spans
    pub fn clear(&mut self) {
        self.spans.clear();
        self.next_id = 1;
    }

    /// Get the number of spans
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    /// Check if there are no spans
    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    /// Get the placeholder for a given ID
    pub fn get_placeholder(&self, id: usize) -> Option<&str> {
        self.get_span(id).map(|s| s.placeholder.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_purpose_defaults() {
        let purpose: RenderPurpose = Default::default();
        assert!(matches!(purpose, RenderPurpose::Format));
    }

    #[test]
    fn test_render_purpose_checks() {
        assert!(RenderPurpose::Format.is_format());
        assert!(!RenderPurpose::TranslationSpans.is_format());

        assert!(RenderPurpose::TranslationSpans.is_translation());
        assert!(RenderPurpose::TranslatedSpans.is_translation());
        assert!(RenderPurpose::Translated.is_translation());
        assert!(!RenderPurpose::Format.is_translation());

        assert!(RenderPurpose::TranslationSpans.is_transforming_text());
        assert!(RenderPurpose::Translated.is_transforming_text());
        assert!(!RenderPurpose::Format.is_transforming_text());
    }

    #[test]
    fn test_render_purpose_next() {
        assert!(matches!(
            RenderPurpose::Format.next(),
            RenderPurpose::Format
        ));
        assert!(matches!(
            RenderPurpose::TranslationSpans.next(),
            RenderPurpose::TranslatedSpans
        ));
        assert!(matches!(
            RenderPurpose::TranslatedSpans.next(),
            RenderPurpose::Translated
        ));
        assert!(matches!(
            RenderPurpose::Translated.next(),
            RenderPurpose::Translated
        ));
    }

    #[test]
    fn test_translation_span() {
        let mut span = TranslationSpan::new(1, "Hello", "_{}_");
        assert_eq!(span.id, 1);
        assert_eq!(span.original_text, "Hello");
        assert_eq!(span.placeholder, "_1_");
        assert!(!span.is_isolated);

        assert_eq!(span.get_text(RenderPurpose::Format), "Hello");
        assert_eq!(span.get_text(RenderPurpose::TranslationSpans), "_1_");
        assert_eq!(span.get_text(RenderPurpose::Translated), "Hello");

        span.set_translated("Bonjour");
        assert_eq!(span.get_text(RenderPurpose::Translated), "Bonjour");
    }

    #[test]
    fn test_translation_span_collection() {
        let mut collection = TranslationSpanCollection::new();
        assert!(collection.is_empty());

        collection.add_span("Hello");
        collection.add_span("World");

        assert_eq!(collection.len(), 2);
        assert_eq!(collection.get_original_texts(), vec!["Hello", "World"]);

        let placeholders: Vec<_> = (1..=2)
            .filter_map(|id| collection.get_placeholder(id))
            .collect();
        assert_eq!(placeholders, vec!["_1_", "_2_"]);

        collection
            .set_translated_texts(vec!["Bonjour".to_string(), "Monde".to_string()])
            .unwrap();

        assert_eq!(
            collection.get_span(1).unwrap().get_text(RenderPurpose::Translated),
            "Bonjour"
        );
    }

    #[test]
    fn test_translation_span_collection_error() {
        let mut collection = TranslationSpanCollection::new();
        collection.add_span("Hello");

        let result = collection.set_translated_texts(vec![]);
        assert!(result.is_err());
    }
}
