//! Template system for clmd.
//!
//! This module provides template support for document rendering,
//! inspired by Pandoc's template system.
//!
//! # Example
//!
//! ```
//! use clmd::templates::{Template, TemplateEngine};
//!
//! let template = Template::new("<html><body>{{content}}</body></html>");
//! let engine = TemplateEngine::new();
//! let result = engine.render(&template, &[("content", "Hello World")]).unwrap();
//! assert_eq!(result, "<html><body>Hello World</body></html>");
//! ```

use crate::error::{ClmdError, ClmdResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A template for document rendering.
#[derive(Debug, Clone)]
pub struct Template {
    /// Template content.
    content: String,
    /// Template name (for error messages).
    name: String,
}

impl Template {
    /// Create a new template from a string.
    pub fn new<S: Into<String>>(content: S) -> Self {
        Self {
            content: content.into(),
            name: "inline".to_string(),
        }
    }

    /// Create a new template with a name.
    pub fn with_name<S: Into<String>>(content: S, name: S) -> Self {
        Self {
            content: content.into(),
            name: name.into(),
        }
    }

    /// Load a template from a file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> ClmdResult<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| ClmdError::template_error(format!("Failed to read template: {}", e)))?;
        Ok(Self {
            content,
            name: path.to_string_lossy().to_string(),
        })
    }

    /// Get the template content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get the template name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Template context for variable substitution.
#[derive(Debug, Default, Clone)]
pub struct TemplateContext {
    /// Variables for substitution.
    variables: HashMap<String, String>,
}

impl TemplateContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Add a variable to the context.
    pub fn set<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &mut Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Get a variable value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(|s| s.as_str())
    }

    /// Build context from iterator.
    pub fn from_iter<I, K, V>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut context = Self::new();
        for (k, v) in iter {
            context.set(k, v);
        }
        context
    }
}

/// Template engine for rendering templates.
#[derive(Debug, Default, Clone)]
pub struct TemplateEngine {
    /// Partial templates.
    partials: HashMap<String, Template>,
}

impl TemplateEngine {
    /// Create a new template engine.
    pub fn new() -> Self {
        Self {
            partials: HashMap::new(),
        }
    }

    /// Register a partial template.
    pub fn register_partial<S: Into<String>>(&mut self, name: S, template: Template) {
        self.partials.insert(name.into(), template);
    }

    /// Render a template with the given context.
    ///
    /// Simple variable substitution using `{{variable}}` syntax.
    pub fn render(&self, template: &Template, context: &TemplateContext) -> ClmdResult<String> {
        let mut result = template.content().to_string();

        // Simple variable substitution
        for (key, value) in &context.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Handle partials (simplified)
        for (name, partial) in &self.partials {
            let placeholder = format!("{{{{>{}}}}}", name);
            if result.contains(&placeholder) {
                let rendered = self.render(partial, context)?;
                result = result.replace(&placeholder, &rendered);
            }
        }

        Ok(result)
    }

    /// Render with a simple key-value iterator.
    pub fn render_iter<I, K, V>(&self, template: &Template, iter: I) -> ClmdResult<String>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let context = TemplateContext::from_iter(iter);
        self.render(template, &context)
    }
}

/// Templated writer that wraps another writer with a template.
pub struct TemplatedWriter<W> {
    /// Inner writer.
    inner: W,
    /// Template to apply.
    template: Template,
    /// Template engine.
    engine: TemplateEngine,
}

impl<W> TemplatedWriter<W> {
    /// Create a new templated writer.
    pub fn new(inner: W, template: Template) -> Self {
        Self {
            inner,
            template,
            engine: TemplateEngine::new(),
        }
    }

    /// Get the inner writer.
    pub fn into_inner(self) -> W {
        self.inner
    }
}

/// Template registry for managing multiple templates.
#[derive(Debug, Default, Clone)]
pub struct TemplateRegistry {
    /// Registered templates.
    templates: HashMap<String, Template>,
    /// Default template directory.
    template_dir: Option<PathBuf>,
}

impl TemplateRegistry {
    /// Create a new template registry.
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            template_dir: None,
        }
    }

    /// Set the template directory.
    pub fn with_template_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.template_dir = Some(dir.into());
        self
    }

    /// Register a template.
    pub fn register<S: Into<String>>(&mut self, name: S, template: Template) {
        self.templates.insert(name.into(), template);
    }

    /// Get a template by name.
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name).or_else(|| {
            // Try to load from template directory
            if let Some(dir) = &self.template_dir {
                let path = dir.join(format!("{}.html", name));
                if path.exists() {
                    // Load and cache template
                    return None; // Simplified for now
                }
            }
            None
        })
    }

    /// Load a template from the template directory.
    pub fn load<S: AsRef<str>>(&mut self, name: S) -> ClmdResult<&Template> {
        let name = name.as_ref();
        
        if let Some(template) = self.templates.get(name) {
            return Ok(template);
        }

        if let Some(dir) = &self.template_dir {
            let path = dir.join(format!("{}.html", name));
            if path.exists() {
                let template = Template::from_file(&path)?;
                self.templates.insert(name.to_string(), template);
                return self.templates.get(name).ok_or_else(|| {
                    ClmdError::template_error("Template not found after loading")
                });
            }
        }

        Err(ClmdError::template_error(format!("Template not found: {}", name)))
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Default HTML template for standalone documents.
pub const DEFAULT_HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{{title}}</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            color: #333;
        }
        h1, h2, h3, h4, h5, h6 {
            margin-top: 24px;
            margin-bottom: 16px;
            font-weight: 600;
            line-height: 1.25;
        }
        pre {
            background: #f6f8fa;
            padding: 16px;
            overflow: auto;
            border-radius: 6px;
        }
        code {
            background: rgba(175, 184, 193, 0.2);
            padding: 0.2em 0.4em;
            border-radius: 6px;
            font-size: 85%;
        }
        pre code {
            background: transparent;
            padding: 0;
        }
        blockquote {
            border-left: 4px solid #dfe2e5;
            padding-left: 16px;
            margin-left: 0;
            color: #6a737d;
        }
        table {
            border-collapse: collapse;
            width: 100%;
        }
        th, td {
            border: 1px solid #dfe2e5;
            padding: 6px 13px;
        }
        th {
            background: #f6f8fa;
        }
    </style>
</head>
<body>
{{content}}
</body>
</html>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_new() {
        let template = Template::new("Hello {{name}}!");
        assert_eq!(template.content(), "Hello {{name}}!");
        assert_eq!(template.name(), "inline");
    }

    #[test]
    fn test_template_with_name() {
        let template = Template::with_name("Hello", "greeting");
        assert_eq!(template.name(), "greeting");
    }

    #[test]
    fn test_template_context() {
        let mut context = TemplateContext::new();
        context.set("name", "World").set("greeting", "Hello");
        
        assert_eq!(context.get("name"), Some("World"));
        assert_eq!(context.get("greeting"), Some("Hello"));
        assert_eq!(context.get("missing"), None);
    }

    #[test]
    fn test_template_context_from_iter() {
        let context = TemplateContext::from_iter([("a", "1"), ("b", "2")]);
        assert_eq!(context.get("a"), Some("1"));
        assert_eq!(context.get("b"), Some("2"));
    }

    #[test]
    fn test_template_engine_render() {
        let engine = TemplateEngine::new();
        let template = Template::new("Hello {{name}}!");
        let mut context = TemplateContext::new();
        context.set("name", "World");
        
        let result = engine.render(&template, &context).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_template_engine_render_multiple() {
        let engine = TemplateEngine::new();
        let template = Template::new("{{greeting}} {{name}}!");
        let mut context = TemplateContext::new();
        context.set("greeting", "Hello").set("name", "World");
        
        let result = engine.render(&template, &context).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_template_engine_render_iter() {
        let engine = TemplateEngine::new();
        let template = Template::new("{{a}} + {{b}} = {{c}}");
        
        let result = engine.render_iter(&template, [("a", "1"), ("b", "2"), ("c", "3")]).unwrap();
        assert_eq!(result, "1 + 2 = 3");
    }

    #[test]
    fn test_template_registry() {
        let mut registry = TemplateRegistry::new();
        let template = Template::with_name("Hello", "greeting");
        
        registry.register("greeting", template);
        assert!(registry.get("greeting").is_some());
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn test_default_html_template() {
        assert!(DEFAULT_HTML_TEMPLATE.contains("{{content}}"));
        assert!(DEFAULT_HTML_TEMPLATE.contains("<!DOCTYPE html>"));
    }
}
