//! Translation handler implementation
//!
//! This module provides translation support for the Markdown formatter,
//! inspired by flexmark-java's TranslationHandler. It handles multi-phase
//! rendering for translation workflows.

use super::context::TranslationPlaceholderGenerator;
use super::purpose::{RenderPurpose, TranslationSpanCollection};
use std::collections::HashMap;

/// Translation handler for managing translation workflows
///
/// This trait defines the interface for handling translation-related
/// operations during the rendering process.
pub trait TranslationHandler {
    /// Begin rendering a document
    fn begin_rendering(&mut self);

    /// Get the current render purpose
    fn get_render_purpose(&self) -> RenderPurpose;

    /// Set the render purpose
    fn set_render_purpose(&mut self, purpose: RenderPurpose);

    /// Check if text transformation is active
    fn is_transforming_text(&self) -> bool;

    /// Transform non-translating text
    ///
    /// This is used for text that should not be translated but needs
    /// placeholder substitution during translation phases.
    fn transform_non_translating(&mut self, text: &str) -> String;

    /// Transform translating text
    ///
    /// This is used for text that should be translated. During
    /// TranslationSpans phase, it extracts the text and returns a placeholder.
    fn transform_translating(&mut self, text: &str) -> String;

    /// Transform anchor reference
    ///
    /// Handles anchor references during translation, converting them
    /// to/from placeholders as needed.
    fn transform_anchor_ref(&mut self, page_ref: &str, anchor_ref: &str) -> String;

    /// Get all translating texts
    fn get_translating_texts(&self) -> Vec<String>;

    /// Set translated texts
    fn set_translated_texts(&mut self, texts: Vec<String>) -> Result<(), String>;

    /// Get the translation span collection
    fn get_span_collection(&self) -> &TranslationSpanCollection;

    /// Get a mutable reference to the translation span collection
    fn get_span_collection_mut(&mut self) -> &mut TranslationSpanCollection;

    /// Create a translating span
    fn translating_span<F>(&mut self, render: F) -> String
    where
        F: FnOnce() -> String;

    /// Create a non-translating span
    fn non_translating_span<F>(&mut self, render: F) -> String
    where
        F: FnOnce() -> String;

    /// Get the translation store for custom data
    fn get_translation_store(&self) -> &HashMap<String, String>;

    /// Get a mutable reference to the translation store
    fn get_translation_store_mut(&mut self) -> &mut HashMap<String, String>;
}

/// Default implementation of TranslationHandler
pub struct TranslationHandlerImpl {
    /// Current render purpose
    render_purpose: RenderPurpose,
    /// Translation span collection
    span_collection: TranslationSpanCollection,
    /// Translation store for custom data
    translation_store: HashMap<String, String>,
    /// Placeholder generator
    placeholder_generator: Box<dyn TranslationPlaceholderGenerator>,
    /// Whether we're currently in a post-processing scope
    post_processing_scope: bool,
}

impl std::fmt::Debug for TranslationHandlerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TranslationHandlerImpl")
            .field("render_purpose", &self.render_purpose)
            .field("span_collection", &self.span_collection)
            .field("translation_store", &self.translation_store)
            .field("post_processing_scope", &self.post_processing_scope)
            .finish_non_exhaustive()
    }
}

impl Clone for TranslationHandlerImpl {
    fn clone(&self) -> Self {
        Self {
            render_purpose: self.render_purpose,
            span_collection: self.span_collection.clone(),
            translation_store: self.translation_store.clone(),
            placeholder_generator: Box::new(
                super::context::DefaultPlaceholderGenerator::new(),
            ),
            post_processing_scope: self.post_processing_scope,
        }
    }
}

impl TranslationHandlerImpl {
    /// Create a new translation handler
    pub fn new() -> Self {
        Self {
            render_purpose: RenderPurpose::Format,
            span_collection: TranslationSpanCollection::new(),
            translation_store: HashMap::new(),
            placeholder_generator: Box::new(
                super::context::DefaultPlaceholderGenerator::new(),
            ),
            post_processing_scope: false,
        }
    }

    /// Create a new translation handler with a custom placeholder generator
    pub fn with_placeholder_generator(
        generator: Box<dyn TranslationPlaceholderGenerator>,
    ) -> Self {
        Self {
            render_purpose: RenderPurpose::Format,
            span_collection: TranslationSpanCollection::new(),
            translation_store: HashMap::new(),
            placeholder_generator: generator,
            post_processing_scope: false,
        }
    }

    /// Set the placeholder generator
    pub fn set_placeholder_generator(
        &mut self,
        generator: Box<dyn TranslationPlaceholderGenerator>,
    ) {
        self.placeholder_generator = generator;
    }

    /// Check if post-processing is active
    pub fn is_post_processing(&self) -> bool {
        self.post_processing_scope
    }

    /// Find a span by its original text
    fn find_span_by_original(
        &self,
        text: &str,
    ) -> Option<&super::purpose::TranslationSpan> {
        self.span_collection
            .get_spans()
            .iter()
            .find(|s| s.original_text == text)
    }

    /// Get the placeholder for a span, or return the original text if not found
    fn get_placeholder_or_text(&self, text: &str) -> String {
        self.find_span_by_original(text)
            .map(|span| span.placeholder.clone())
            .unwrap_or_else(|| text.to_string())
    }
}

impl Default for TranslationHandlerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationHandler for TranslationHandlerImpl {
    fn begin_rendering(&mut self) {
        self.span_collection.clear();
        self.translation_store.clear();
        self.post_processing_scope = false;
    }

    fn get_render_purpose(&self) -> RenderPurpose {
        self.render_purpose
    }

    fn set_render_purpose(&mut self, purpose: RenderPurpose) {
        self.render_purpose = purpose;
    }

    fn is_transforming_text(&self) -> bool {
        self.render_purpose.is_transforming_text()
    }

    fn transform_non_translating(&mut self, text: &str) -> String {
        match self.render_purpose {
            RenderPurpose::Format => text.to_string(),
            RenderPurpose::TranslationSpans => {
                // Create a placeholder for non-translating text
                let span = self.span_collection.add_isolated_span(text);
                span.placeholder.clone()
            }
            RenderPurpose::TranslatedSpans => {
                // Return the placeholder for the matching span
                self.get_placeholder_or_text(text)
            }
            RenderPurpose::Translated => {
                // Return the original text (non-translating)
                text.to_string()
            }
        }
    }

    fn transform_translating(&mut self, text: &str) -> String {
        match self.render_purpose {
            RenderPurpose::Format => text.to_string(),
            RenderPurpose::TranslationSpans => {
                // Create a placeholder for translating text
                let span = self.span_collection.add_span(text);
                span.placeholder.clone()
            }
            RenderPurpose::TranslatedSpans => {
                // Return the placeholder for the matching span
                self.get_placeholder_or_text(text)
            }
            RenderPurpose::Translated => {
                // Return the translated text if available, otherwise original
                self.find_span_by_original(text)
                    .and_then(|span| span.translated_text.clone())
                    .unwrap_or_else(|| text.to_string())
            }
        }
    }

    fn transform_anchor_ref(&mut self, page_ref: &str, anchor_ref: &str) -> String {
        match self.render_purpose {
            RenderPurpose::Format => anchor_ref.to_string(),
            RenderPurpose::TranslationSpans => {
                // Convert anchor to placeholder
                let full_ref = if page_ref.is_empty() {
                    anchor_ref.to_string()
                } else {
                    format!("{}# {}", page_ref, anchor_ref)
                };
                let span = self.span_collection.add_isolated_span(&full_ref);
                span.placeholder.clone()
            }
            RenderPurpose::TranslatedSpans => {
                // Return placeholder
                let full_ref = if page_ref.is_empty() {
                    anchor_ref.to_string()
                } else {
                    format!("{}# {}", page_ref, anchor_ref)
                };
                if let Some(span) = self
                    .span_collection
                    .get_spans()
                    .iter()
                    .find(|s| s.original_text == full_ref)
                {
                    span.placeholder.clone()
                } else {
                    anchor_ref.to_string()
                }
            }
            RenderPurpose::Translated => {
                // Return potentially modified anchor
                // In a full implementation, this would handle ID uniquification
                anchor_ref.to_string()
            }
        }
    }

    fn get_translating_texts(&self) -> Vec<String> {
        self.span_collection
            .get_spans()
            .iter()
            .filter(|s| !s.is_isolated)
            .map(|s| s.original_text.clone())
            .collect()
    }

    fn set_translated_texts(&mut self, texts: Vec<String>) -> Result<(), String> {
        self.span_collection.set_translated_texts(texts)
    }

    fn get_span_collection(&self) -> &TranslationSpanCollection {
        &self.span_collection
    }

    fn get_span_collection_mut(&mut self) -> &mut TranslationSpanCollection {
        &mut self.span_collection
    }

    fn translating_span<F>(&mut self, render: F) -> String
    where
        F: FnOnce() -> String,
    {
        match self.render_purpose {
            RenderPurpose::Format => render(),
            RenderPurpose::TranslationSpans => {
                let content = render();
                let span = self.span_collection.add_span(&content);
                span.placeholder.clone()
            }
            RenderPurpose::TranslatedSpans => {
                // Suppress output, return placeholder
                let content = render();
                if let Some(span) = self
                    .span_collection
                    .get_spans()
                    .iter()
                    .find(|s| s.original_text == content)
                {
                    span.placeholder.clone()
                } else {
                    String::new()
                }
            }
            RenderPurpose::Translated => {
                // Return translated content
                let content = render();
                if let Some(span) = self
                    .span_collection
                    .get_spans()
                    .iter()
                    .find(|s| s.original_text == content)
                {
                    if let Some(ref translated) = span.translated_text {
                        translated.clone()
                    } else {
                        content
                    }
                } else {
                    content
                }
            }
        }
    }

    fn non_translating_span<F>(&mut self, render: F) -> String
    where
        F: FnOnce() -> String,
    {
        match self.render_purpose {
            RenderPurpose::Format => render(),
            RenderPurpose::TranslationSpans => {
                let content = render();
                let span = self.span_collection.add_isolated_span(&content);
                span.placeholder.clone()
            }
            RenderPurpose::TranslatedSpans => {
                // Suppress output, return placeholder
                let content = render();
                if let Some(span) = self
                    .span_collection
                    .get_spans()
                    .iter()
                    .find(|s| s.original_text == content)
                {
                    span.placeholder.clone()
                } else {
                    String::new()
                }
            }
            RenderPurpose::Translated => render(),
        }
    }

    fn get_translation_store(&self) -> &HashMap<String, String> {
        &self.translation_store
    }

    fn get_translation_store_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.translation_store
    }
}

/// Translation context extension trait
///
/// This trait provides methods for working with translation
/// in the context of node formatting.
pub trait TranslationContext {
    /// Get the render purpose
    fn get_render_purpose(&self) -> RenderPurpose;

    /// Check if text transformation is active
    fn is_transforming_text(&self) -> bool;

    /// Transform non-translating text
    fn transform_non_translating(&self, text: &str) -> String;

    /// Transform translating text
    fn transform_translating(&self, text: &str) -> String;

    /// Transform anchor reference
    fn transform_anchor_ref(&self, page_ref: &str, anchor_ref: &str) -> String;

    /// Check if post-processing is active
    fn is_post_processing_non_translating(&self) -> bool;

    /// Get the translation store
    fn get_translation_store(&self) -> &HashMap<String, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_handler_new() {
        let handler = TranslationHandlerImpl::new();
        assert!(matches!(
            handler.get_render_purpose(),
            RenderPurpose::Format
        ));
        assert!(!handler.is_transforming_text());
    }

    #[test]
    fn test_translation_handler_purpose() {
        let mut handler = TranslationHandlerImpl::new();

        handler.set_render_purpose(RenderPurpose::TranslationSpans);
        assert!(matches!(
            handler.get_render_purpose(),
            RenderPurpose::TranslationSpans
        ));
        assert!(handler.is_transforming_text());

        handler.set_render_purpose(RenderPurpose::Translated);
        assert!(matches!(
            handler.get_render_purpose(),
            RenderPurpose::Translated
        ));
        assert!(handler.is_transforming_text());
    }

    #[test]
    fn test_transform_non_translating_format() {
        let mut handler = TranslationHandlerImpl::new();
        handler.set_render_purpose(RenderPurpose::Format);

        let result = handler.transform_non_translating("Hello World");
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_transform_non_translating_translation_spans() {
        let mut handler = TranslationHandlerImpl::new();
        handler.set_render_purpose(RenderPurpose::TranslationSpans);

        let result = handler.transform_non_translating("Hello World");
        assert!(result.starts_with("_"));
        assert!(result.ends_with("_"));
        assert_ne!(result, "Hello World");
    }

    #[test]
    fn test_transform_translating_translation_spans() {
        let mut handler = TranslationHandlerImpl::new();
        handler.set_render_purpose(RenderPurpose::TranslationSpans);

        let result = handler.transform_translating("Hello");
        assert!(result.starts_with("_"));
        assert!(result.ends_with("_"));

        // Check that the span was recorded
        assert_eq!(handler.get_span_collection().len(), 1);
    }

    #[test]
    fn test_get_translating_texts() {
        let mut handler = TranslationHandlerImpl::new();
        handler.set_render_purpose(RenderPurpose::TranslationSpans);

        handler.transform_translating("Hello");
        handler.transform_translating("World");
        handler.transform_non_translating("Code");

        let texts = handler.get_translating_texts();
        assert_eq!(texts.len(), 2);
        assert!(texts.contains(&"Hello".to_string()));
        assert!(texts.contains(&"World".to_string()));
    }

    #[test]
    fn test_set_translated_texts() {
        let mut handler = TranslationHandlerImpl::new();
        handler.set_render_purpose(RenderPurpose::TranslationSpans);

        handler.transform_translating("Hello");
        handler.transform_translating("World");

        handler
            .set_translated_texts(vec!["Bonjour".to_string(), "Monde".to_string()])
            .unwrap();

        handler.set_render_purpose(RenderPurpose::Translated);
        let result1 = handler.transform_translating("Hello");
        let result2 = handler.transform_translating("World");

        assert_eq!(result1, "Bonjour");
        assert_eq!(result2, "Monde");
    }

    #[test]
    fn test_transform_anchor_ref() {
        let mut handler = TranslationHandlerImpl::new();

        // Format mode - should return as-is
        handler.set_render_purpose(RenderPurpose::Format);
        let result = handler.transform_anchor_ref("page", "section");
        assert_eq!(result, "section");
    }

    #[test]
    fn test_translation_store() {
        let mut handler = TranslationHandlerImpl::new();

        handler
            .get_translation_store_mut()
            .insert("key".to_string(), "value".to_string());

        assert_eq!(
            handler.get_translation_store().get("key"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_begin_rendering() {
        let mut handler = TranslationHandlerImpl::new();
        handler.set_render_purpose(RenderPurpose::TranslationSpans);
        handler.transform_translating("Hello");

        assert_eq!(handler.get_span_collection().len(), 1);

        handler.begin_rendering();

        assert_eq!(handler.get_span_collection().len(), 0);
    }

    #[test]
    fn test_translating_span() {
        let mut handler = TranslationHandlerImpl::new();

        // Format mode
        handler.set_render_purpose(RenderPurpose::Format);
        let result = handler.translating_span(|| "Hello".to_string());
        assert_eq!(result, "Hello");

        // TranslationSpans mode
        handler.set_render_purpose(RenderPurpose::TranslationSpans);
        let result = handler.translating_span(|| "World".to_string());
        assert!(result.starts_with("_"));
    }

    #[test]
    fn test_non_translating_span() {
        let mut handler = TranslationHandlerImpl::new();

        // Format mode
        handler.set_render_purpose(RenderPurpose::Format);
        let result = handler.non_translating_span(|| "Code".to_string());
        assert_eq!(result, "Code");
    }
}
