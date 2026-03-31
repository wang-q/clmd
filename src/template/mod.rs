//! Template system for clmd.
//!
//! This module provides a flexible template system for document rendering,
//! inspired by Pandoc's template architecture. Templates use a simple
//! variable substitution syntax similar to Pandoc's templates.
//!
//! # Template Syntax
//!
//! - `${variable}` - Insert variable value
//! - `${variable:default}` - Insert variable with default value
//! - `$if(variable)$...$endif$` - Conditional block
//! - `$for(variable)$...$endfor$` - Loop block
//! - `$partial(name)$` - Include partial template
//!
//! # Example
//!
//! ```
//! use clmd::template::{Template, TemplateContext};
//!
//! let template = Template::compile("Hello, ${name}!").unwrap();
//! let mut context = TemplateContext::new();
//! context.set("name", "World");
//!
//! let result = template.render(&context);
//! assert_eq!(result, "Hello, World!");
//! ```

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// A compiled template.
#[derive(Debug, Clone)]
pub struct Template {
    /// The original template source.
    source: String,
    /// Parsed template parts.
    parts: Vec<TemplatePart>,
}

/// A part of a template.
#[derive(Debug, Clone, PartialEq)]
enum TemplatePart {
    /// Literal text.
    Text(String),
    /// Variable substitution.
    Variable {
        name: String,
        default: Option<String>,
    },
    /// Conditional block.
    If {
        condition: String,
        then_branch: Vec<TemplatePart>,
        else_branch: Vec<TemplatePart>,
    },
    /// Loop block.
    For {
        variable: String,
        body: Vec<TemplatePart>,
    },
    /// Partial template inclusion.
    Partial(String),
}

/// Context for template rendering.
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    /// Variable values.
    variables: HashMap<String, String>,
    /// List variables for loops.
    lists: HashMap<String, Vec<TemplateContext>>,
}

impl TemplateContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable value.
    pub fn set<K: Into<String>, V: Into<String>>(
        &mut self,
        key: K,
        value: V,
    ) -> &mut Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Get a variable value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(|s| s.as_str())
    }

    /// Check if a variable exists and is truthy.
    pub fn is_truthy(&self, key: &str) -> bool {
        self.variables
            .get(key)
            .map(|v| !v.is_empty() && v != "false" && v != "0")
            .unwrap_or(false)
    }

    /// Set a list variable.
    pub fn set_list<K: Into<String>>(
        &mut self,
        key: K,
        items: Vec<TemplateContext>,
    ) -> &mut Self {
        self.lists.insert(key.into(), items);
        self
    }

    /// Get a list variable.
    pub fn get_list(&self, key: &str) -> Option<&Vec<TemplateContext>> {
        self.lists.get(key)
    }

    /// Extend this context with another context.
    pub fn extend(&mut self, other: &TemplateContext) -> &mut Self {
        self.variables.extend(other.variables.clone());
        self.lists.extend(other.lists.clone());
        self
    }

    /// Create a context with document metadata.
    pub fn from_metadata(
        title: Option<&str>,
        author: Option<&str>,
        date: Option<&str>,
    ) -> Self {
        let mut ctx = Self::new();
        if let Some(title) = title {
            ctx.set("title", title);
        }
        if let Some(author) = author {
            ctx.set("author", author);
        }
        if let Some(date) = date {
            ctx.set("date", date);
        }
        ctx
    }
}

/// Error type for template operations.
#[derive(Debug, Clone)]
pub enum TemplateError {
    /// Syntax error in template.
    SyntaxError(String),
    /// Variable not found.
    VariableNotFound(String),
    /// Partial template not found.
    PartialNotFound(String),
    /// Recursive inclusion detected.
    RecursiveInclude(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SyntaxError(msg) => write!(f, "Template syntax error: {}", msg),
            Self::VariableNotFound(name) => write!(f, "Variable not found: {}", name),
            Self::PartialNotFound(name) => {
                write!(f, "Partial template not found: {}", name)
            }
            Self::RecursiveInclude(name) => {
                write!(f, "Recursive template inclusion: {}", name)
            }
        }
    }
}

impl std::error::Error for TemplateError {}

impl Template {
    /// Compile a template from a string.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::template::Template;
    ///
    /// let template = Template::compile("Hello, ${name}!").unwrap();
    /// ```
    pub fn compile(source: impl Into<String>) -> Result<Self, TemplateError> {
        let source = source.into();
        let parts = Self::parse(&source)?;
        Ok(Self { source, parts })
    }

    /// Load and compile a template from a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or the template has syntax errors.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, TemplateError> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| TemplateError::PartialNotFound(e.to_string()))?;
        Self::compile(source)
    }

    /// Get the template source.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Render the template with the given context.
    ///
    /// # Example
    ///
    /// ```
    /// use clmd::template::{Template, TemplateContext};
    ///
    /// let template = Template::compile("Hello, ${name}!").unwrap();
    /// let mut context = TemplateContext::new();
    /// context.set("name", "World");
    ///
    /// let result = template.render(&context);
    /// assert_eq!(result, "Hello, World!");
    /// ```
    pub fn render(&self, context: &TemplateContext) -> String {
        self.render_parts(&self.parts, context)
    }

    /// Parse template source into parts.
    fn parse(source: &str) -> Result<Vec<TemplatePart>, TemplateError> {
        let mut parts = Vec::new();
        let mut chars = source.chars().peekable();
        let mut current_text = String::new();

        while let Some(ch) = chars.next() {
            if ch == '$' {
                // Check for template syntax
                if chars.peek() == Some(&'{') {
                    // Variable: ${name} or ${name:default}
                    if !current_text.is_empty() {
                        parts.push(TemplatePart::Text(current_text.clone()));
                        current_text.clear();
                    }
                    chars.next(); // consume '{'
                    let var_parts = Self::parse_until(&mut chars, '}')?;
                    parts.push(Self::parse_variable(&var_parts)?);
                } else if chars.peek() == Some(&'i') {
                    // If statement: $if(var)$...$endif$
                    let keyword: String = chars.by_ref().take(2).collect();
                    if keyword == "if" && chars.peek() == Some(&'(') {
                        if !current_text.is_empty() {
                            parts.push(TemplatePart::Text(current_text.clone()));
                            current_text.clear();
                        }
                        chars.next(); // consume '('
                        let condition = Self::parse_until(&mut chars, ')')?;
                        chars.next(); // consume '$'
                        let then_branch = Self::parse_branch(&mut chars, "endif")?;
                        parts.push(TemplatePart::If {
                            condition,
                            then_branch,
                            else_branch: Vec::new(),
                        });
                    } else {
                        current_text.push(ch);
                        current_text.push_str(&keyword);
                    }
                } else if chars.peek() == Some(&'f') {
                    // For loop: $for(var)$...$endfor$
                    let keyword: String = chars.by_ref().take(3).collect();
                    if keyword == "for" && chars.peek() == Some(&'(') {
                        if !current_text.is_empty() {
                            parts.push(TemplatePart::Text(current_text.clone()));
                            current_text.clear();
                        }
                        chars.next(); // consume '('
                        let variable = Self::parse_until(&mut chars, ')')?;
                        chars.next(); // consume '$'
                        let body = Self::parse_branch(&mut chars, "endfor")?;
                        parts.push(TemplatePart::For { variable, body });
                    } else {
                        current_text.push(ch);
                        current_text.push_str(&keyword);
                    }
                } else {
                    current_text.push(ch);
                }
            } else {
                current_text.push(ch);
            }
        }

        if !current_text.is_empty() {
            parts.push(TemplatePart::Text(current_text));
        }

        Ok(parts)
    }

    /// Parse until a delimiter character.
    fn parse_until(
        chars: &mut std::iter::Peekable<std::str::Chars>,
        delim: char,
    ) -> Result<String, TemplateError> {
        let mut result = String::new();
        while let Some(ch) = chars.next() {
            if ch == delim {
                return Ok(result);
            }
            result.push(ch);
        }
        Err(TemplateError::SyntaxError(format!("Expected '{}'", delim)))
    }

    /// Parse a variable expression.
    fn parse_variable(expr: &str) -> Result<TemplatePart, TemplateError> {
        if let Some(pos) = expr.find(':') {
            let (name, default) = expr.split_at(pos);
            Ok(TemplatePart::Variable {
                name: name.trim().to_string(),
                default: Some(default[1..].trim().to_string()),
            })
        } else {
            Ok(TemplatePart::Variable {
                name: expr.trim().to_string(),
                default: None,
            })
        }
    }

    /// Parse a branch (content between $keyword$ and $endkeyword$).
    fn parse_branch(
        chars: &mut std::iter::Peekable<std::str::Chars>,
        end_keyword: &str,
    ) -> Result<Vec<TemplatePart>, TemplateError> {
        let mut content = String::new();
        let mut depth = 1;

        while let Some(ch) = chars.next() {
            if ch == '$' {
                // Check for nested or ending markers
                let mut keyword = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphabetic() || ch == '_' {
                        keyword.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                if keyword == end_keyword {
                    depth -= 1;
                    if depth == 0 {
                        // Consume the closing $
                        if chars.peek() == Some(&'$') {
                            chars.next();
                        }
                        return Self::parse(&content);
                    } else {
                        content.push('$');
                        content.push_str(&keyword);
                    }
                } else if keyword == "if" || keyword == "for" {
                    depth += 1;
                    content.push('$');
                    content.push_str(&keyword);
                } else {
                    content.push('$');
                    content.push_str(&keyword);
                }
            } else {
                content.push(ch);
            }
        }

        Err(TemplateError::SyntaxError(format!(
            "Unclosed block, expected $endif$ or $endfor$"
        )))
    }

    /// Render template parts.
    fn render_parts(&self, parts: &[TemplatePart], context: &TemplateContext) -> String {
        let mut result = String::new();

        for part in parts {
            match part {
                TemplatePart::Text(text) => result.push_str(text),
                TemplatePart::Variable { name, default } => {
                    let value = context
                        .get(name)
                        .map(|s| s.to_string())
                        .or_else(|| default.clone())
                        .unwrap_or_default();
                    result.push_str(&value);
                }
                TemplatePart::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    if context.is_truthy(condition) {
                        result.push_str(&self.render_parts(then_branch, context));
                    } else {
                        result.push_str(&self.render_parts(else_branch, context));
                    }
                }
                TemplatePart::For { variable, body } => {
                    if let Some(items) = context.get_list(variable) {
                        for item in items {
                            let mut merged = context.clone();
                            merged.extend(item);
                            result.push_str(&self.render_parts(body, &merged));
                        }
                    }
                }
                TemplatePart::Partial(name) => {
                    // Partials are resolved by the template engine
                    // For now, just output a placeholder
                    result.push_str(&format!("<!-- partial: {} -->", name));
                }
            }
        }

        result
    }
}

/// Template engine for managing multiple templates.
#[derive(Debug, Clone, Default)]
pub struct TemplateEngine {
    /// Compiled templates.
    templates: HashMap<String, Template>,
    /// Partial templates.
    partials: HashMap<String, Template>,
    /// Default template directory.
    template_dir: Option<PathBuf>,
}

impl TemplateEngine {
    /// Create a new template engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the template directory.
    pub fn with_template_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.template_dir = Some(dir.into());
        self
    }

    /// Register a template.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        template: Template,
    ) -> &mut Self {
        self.templates.insert(name.into(), template);
        self
    }

    /// Register a template from a string.
    pub fn register_string(
        &mut self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> Result<&mut Self, TemplateError> {
        let template = Template::compile(source)?;
        self.register(name, template);
        Ok(self)
    }

    /// Register a partial template.
    pub fn register_partial(
        &mut self,
        name: impl Into<String>,
        template: Template,
    ) -> &mut Self {
        self.partials.insert(name.into(), template);
        self
    }

    /// Get a template by name.
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// Render a template by name.
    pub fn render(&self, name: &str, context: &TemplateContext) -> Option<String> {
        self.get(name).map(|t| t.render(context))
    }

    /// Load a template from the template directory.
    pub fn load_template<P: AsRef<Path>>(
        &mut self,
        name: impl Into<String>,
        path: P,
    ) -> Result<&mut Self, TemplateError> {
        let template = Template::from_file(path)?;
        self.register(name, template);
        Ok(self)
    }

    /// Get a default HTML template.
    pub fn default_html_template() -> Template {
        Template::compile(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    $if(title)$<title>${title}</title>$endif$
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; line-height: 1.6; max-width: 800px; margin: 0 auto; padding: 20px; }
        h1, h2, h3, h4, h5, h6 { margin-top: 1.5em; margin-bottom: 0.5em; }
        code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; }
        pre { background: #f4f4f4; padding: 16px; overflow-x: auto; border-radius: 6px; }
        blockquote { border-left: 4px solid #ddd; margin: 0; padding-left: 16px; color: #666; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background: #f4f4f4; }
    </style>
</head>
<body>
$if(title)$<h1>${title}</h1>$endif$
$if(author)$<p class="author">${author}</p>$endif$
$if(date)$<p class="date">${date}</p>$endif$
${body}
</body>
</html>"#).unwrap()
    }

    /// Get a default standalone template for a format.
    pub fn default_template(format: &str) -> Option<Template> {
        match format {
            "html" | "html5" => Some(Self::default_html_template()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_variable() {
        let template = Template::compile("Hello, ${name}!").unwrap();
        let mut context = TemplateContext::new();
        context.set("name", "World");

        assert_eq!(template.render(&context), "Hello, World!");
    }

    #[test]
    fn test_variable_with_default() {
        let template = Template::compile("Hello, ${name:Guest}!").unwrap();
        let context = TemplateContext::new();

        assert_eq!(template.render(&context), "Hello, Guest!");
    }

    #[test]
    fn test_if_condition() {
        let template = Template::compile("$if(show)$Visible$endif$").unwrap();

        let mut ctx1 = TemplateContext::new();
        ctx1.set("show", "true");
        assert_eq!(template.render(&ctx1), "Visible");

        let ctx2 = TemplateContext::new();
        assert_eq!(template.render(&ctx2), "");
    }

    #[test]
    fn test_context_from_metadata() {
        let ctx = TemplateContext::from_metadata(
            Some("My Title"),
            Some("John Doe"),
            Some("2024-01-01"),
        );

        assert_eq!(ctx.get("title"), Some("My Title"));
        assert_eq!(ctx.get("author"), Some("John Doe"));
        assert_eq!(ctx.get("date"), Some("2024-01-01"));
    }

    #[test]
    fn test_template_engine() {
        let mut engine = TemplateEngine::new();
        engine
            .register_string("greeting", "Hello, ${name}!")
            .unwrap();

        let mut ctx = TemplateContext::new();
        ctx.set("name", "World");

        assert_eq!(
            engine.render("greeting", &ctx),
            Some("Hello, World!".to_string())
        );
    }

    #[test]
    fn test_default_html_template() {
        let template = TemplateEngine::default_html_template();
        let mut ctx = TemplateContext::new();
        ctx.set("title", "Test");
        ctx.set("body", "<p>Content</p>");

        let result = template.render(&ctx);
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<title>Test</title>"));
        assert!(result.contains("<p>Content</p>"));
    }

    #[test]
    fn test_truthy_values() {
        let mut ctx = TemplateContext::new();

        ctx.set("val", "true");
        assert!(ctx.is_truthy("val"));

        ctx.set("val", "1");
        assert!(ctx.is_truthy("val"));

        ctx.set("val", "hello");
        assert!(ctx.is_truthy("val"));

        ctx.set("val", "");
        assert!(!ctx.is_truthy("val"));

        ctx.set("val", "false");
        assert!(!ctx.is_truthy("val"));

        ctx.set("val", "0");
        assert!(!ctx.is_truthy("val"));
    }
}
