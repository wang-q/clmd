//! Emoji support for clmd.
//!
//! This module provides emoji handling capabilities, inspired by Pandoc's
//! emoji support. It allows conversion between emoji shortcodes (like `:smile:`)
//! and Unicode emoji characters.
//!
//! # Example
//!
//! ```ignore
//! use clmd::text::emoji::{EmojiMapper, has_emoji_shortcode};
//!
//! // Convert shortcode to emoji
//! let mapper = EmojiMapper::new();
//! let emoji = mapper.shortcode_to_emoji(":smile:");
//! assert_eq!(emoji, Some("😄"));
//!
//! // Check if text contains emoji shortcodes
//! assert!(has_emoji_shortcode("Hello :smile: world"));
//! ```

use std::collections::HashMap;

/// A mapper for emoji shortcodes to Unicode characters.
#[derive(Debug, Clone)]
pub struct EmojiMapper {
    mappings: HashMap<String, String>,
}

impl Default for EmojiMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl EmojiMapper {
    /// Create a new emoji mapper with built-in emoji mappings.
    pub fn new() -> Self {
        let mut mappings = HashMap::new();
        Self::populate_common_emojis(&mut mappings);
        Self { mappings }
    }

    /// Create an empty emoji mapper.
    pub fn empty() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Add a custom emoji mapping.
    pub fn add_mapping(
        &mut self,
        shortcode: impl Into<String>,
        emoji: impl Into<String>,
    ) {
        let shortcode = shortcode.into();
        let shortcode = if shortcode.starts_with(':') && shortcode.ends_with(':') {
            shortcode
        } else {
            format!(":{shortcode}:")
        };
        self.mappings.insert(shortcode, emoji.into());
    }

    /// Convert a shortcode to an emoji.
    ///
    /// Returns `None` if the shortcode is not recognized.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::text::emoji::EmojiMapper;
    ///
    /// let mapper = EmojiMapper::new();
    /// assert_eq!(mapper.shortcode_to_emoji(":smile:"), Some("😄"));
    /// assert_eq!(mapper.shortcode_to_emoji(":unknown:"), None);
    /// ```
    pub fn shortcode_to_emoji(&self, shortcode: &str) -> Option<&str> {
        self.mappings.get(shortcode).map(|s| s.as_str())
    }

    /// Convert emoji characters in text to shortcodes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::text::emoji::EmojiMapper;
    ///
    /// let mapper = EmojiMapper::new();
    /// let text = mapper.emoji_to_shortcode("😄");
    /// assert_eq!(text, Some(":smile:".to_string()));
    /// ```
    pub fn emoji_to_shortcode(&self, emoji: &str) -> Option<String> {
        self.mappings
            .iter()
            .find(|(_, v)| v.as_str() == emoji)
            .map(|(k, _)| k.clone())
    }

    /// Replace all shortcodes in text with emoji characters.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use clmd::text::emoji::EmojiMapper;
    ///
    /// let mapper = EmojiMapper::new();
    /// let text = mapper.replace_shortcodes("Hello :smile: :wave:");
    /// assert_eq!(text, "Hello 😄 👋");
    /// ```
    pub fn replace_shortcodes(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Find all potential shortcodes (text between colons)
        let mut start = 0;
        while let Some(colon_pos) = result[start..].find(':') {
            let absolute_pos = start + colon_pos;

            // Look for closing colon
            if let Some(end_colon) = result[absolute_pos + 1..].find(':') {
                let end_absolute = absolute_pos + 1 + end_colon;
                let shortcode = &result[absolute_pos..=end_absolute];

                if let Some(emoji) = self.shortcode_to_emoji(shortcode) {
                    result.replace_range(absolute_pos..=end_absolute, emoji);
                    start = absolute_pos + emoji.len();
                } else {
                    start = absolute_pos + 1;
                }
            } else {
                break;
            }
        }

        result
    }

    /// Get all available shortcodes.
    pub fn shortcodes(&self) -> impl Iterator<Item = &String> {
        self.mappings.keys()
    }

    /// Get the number of emoji mappings.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Check if the mapper is empty.
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Populate common emoji mappings.
    fn populate_common_emojis(mappings: &mut HashMap<String, String>) {
        // Smileys and emotions
        mappings.insert(":smile:".to_string(), "😄".to_string());
        mappings.insert(":grinning:".to_string(), "😀".to_string());
        mappings.insert(":joy:".to_string(), "😂".to_string());
        mappings.insert(":wink:".to_string(), "😉".to_string());
        mappings.insert(":blush:".to_string(), "😊".to_string());
        mappings.insert(":heart_eyes:".to_string(), "😍".to_string());
        mappings.insert(":kissing_heart:".to_string(), "😘".to_string());
        mappings.insert(":thinking:".to_string(), "🤔".to_string());
        mappings.insert(":neutral_face:".to_string(), "😐".to_string());
        mappings.insert(":expressionless:".to_string(), "😑".to_string());
        mappings.insert(":confused:".to_string(), "😕".to_string());
        mappings.insert(":slightly_frowning_face:".to_string(), "🙁".to_string());
        mappings.insert(":frowning_face:".to_string(), "☹️".to_string());
        mappings.insert(":cry:".to_string(), "😢".to_string());
        mappings.insert(":sob:".to_string(), "😭".to_string());
        mappings.insert(":angry:".to_string(), "😠".to_string());
        mappings.insert(":rage:".to_string(), "😡".to_string());
        mappings.insert(":star_struck:".to_string(), "🤩".to_string());
        mappings.insert(":sunglasses:".to_string(), "😎".to_string());
        mappings.insert(":nerd_face:".to_string(), "🤓".to_string());
        mappings.insert(":face_with_monocle:".to_string(), "🧐".to_string());

        // Gestures and people
        mappings.insert(":wave:".to_string(), "👋".to_string());
        mappings.insert(":thumbsup:".to_string(), "👍".to_string());
        mappings.insert(":thumbsdown:".to_string(), "👎".to_string());
        mappings.insert(":clap:".to_string(), "👏".to_string());
        mappings.insert(":raised_hands:".to_string(), "🙌".to_string());
        mappings.insert(":pray:".to_string(), "🙏".to_string());
        mappings.insert(":ok_hand:".to_string(), "👌".to_string());
        mappings.insert(":v:".to_string(), "✌️".to_string());
        mappings.insert(":point_up:".to_string(), "☝️".to_string());
        mappings.insert(":point_down:".to_string(), "👇".to_string());
        mappings.insert(":point_left:".to_string(), "👈".to_string());
        mappings.insert(":point_right:".to_string(), "👉".to_string());

        // Hearts and symbols
        mappings.insert(":heart:".to_string(), "❤️".to_string());
        mappings.insert(":orange_heart:".to_string(), "🧡".to_string());
        mappings.insert(":yellow_heart:".to_string(), "💛".to_string());
        mappings.insert(":green_heart:".to_string(), "💚".to_string());
        mappings.insert(":blue_heart:".to_string(), "💙".to_string());
        mappings.insert(":purple_heart:".to_string(), "💜".to_string());
        mappings.insert(":black_heart:".to_string(), "🖤".to_string());
        mappings.insert(":white_heart:".to_string(), "🤍".to_string());
        mappings.insert(":brown_heart:".to_string(), "🤎".to_string());
        mappings.insert(":broken_heart:".to_string(), "💔".to_string());
        mappings.insert(":sparkling_heart:".to_string(), "💖".to_string());
        mappings.insert(":star:".to_string(), "⭐".to_string());
        mappings.insert(":sparkles:".to_string(), "✨".to_string());

        // Objects
        mappings.insert(":fire:".to_string(), "🔥".to_string());
        mappings.insert(":rocket:".to_string(), "🚀".to_string());
        mappings.insert(":tada:".to_string(), "🎉".to_string());
        mappings.insert(":gift:".to_string(), "🎁".to_string());
        mappings.insert(":book:".to_string(), "📖".to_string());
        mappings.insert(":computer:".to_string(), "💻".to_string());
        mappings.insert(":phone:".to_string(), "📱".to_string());
        mappings.insert(":bulb:".to_string(), "💡".to_string());
        mappings.insert(":lock:".to_string(), "🔒".to_string());
        mappings.insert(":key:".to_string(), "🔑".to_string());
        mappings.insert(":bell:".to_string(), "🔔".to_string());
        mappings.insert(":warning:".to_string(), "⚠️".to_string());
        mappings.insert(":x:".to_string(), "❌".to_string());
        mappings.insert(":o:".to_string(), "⭕".to_string());
        mappings.insert(":white_check_mark:".to_string(), "✅".to_string());
        mappings.insert(":question:".to_string(), "❓".to_string());
        mappings.insert(":exclamation:".to_string(), "❗".to_string());

        // Nature
        mappings.insert(":sun:".to_string(), "☀️".to_string());
        mappings.insert(":cloud:".to_string(), "☁️".to_string());
        mappings.insert(":rainbow:".to_string(), "🌈".to_string());
        mappings.insert(":umbrella:".to_string(), "☂️".to_string());
        mappings.insert(":snowflake:".to_string(), "❄️".to_string());
        mappings.insert(":zap:".to_string(), "⚡".to_string());
        mappings.insert(":droplet:".to_string(), "💧".to_string());

        // Food
        mappings.insert(":coffee:".to_string(), "☕".to_string());
        mappings.insert(":tea:".to_string(), "🍵".to_string());
        mappings.insert(":pizza:".to_string(), "🍕".to_string());
        mappings.insert(":burger:".to_string(), "🍔".to_string());
        mappings.insert(":cake:".to_string(), "🍰".to_string());
        mappings.insert(":ice_cream:".to_string(), "🍦".to_string());
        mappings.insert(":apple:".to_string(), "🍎".to_string());
        mappings.insert(":banana:".to_string(), "🍌".to_string());
    }
}

/// Check if text contains emoji shortcodes.
///
/// # Example
///
/// ```ignore
/// use clmd::text::emoji::has_emoji_shortcode;
///
/// assert!(has_emoji_shortcode("Hello :smile:"));
/// assert!(!has_emoji_shortcode("Hello world"));
/// ```ignore
pub fn has_emoji_shortcode(text: &str) -> bool {
    // Simple check for pattern :word:
    let mut in_shortcode = false;
    let mut shortcode_start = 0;

    for (i, ch) in text.char_indices() {
        if ch == ':' {
            if !in_shortcode {
                in_shortcode = true;
                shortcode_start = i;
            } else {
                // Check if we have a valid shortcode (at least one character between colons)
                if i > shortcode_start + 1 {
                    let content = &text[shortcode_start + 1..i];
                    // Valid shortcodes contain only alphanumeric characters and underscores
                    if content.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return true;
                    }
                }
                in_shortcode = false;
            }
        }
    }

    false
}

/// Replace emoji shortcodes in text with Unicode characters using the default mapper.
///
/// This is a convenience function for simple use cases.
///
/// # Example
///
/// ```ignore
/// use clmd::text::emoji::replace_emoji_shortcodes;
///
/// let text = replace_emoji_shortcodes("Hello :smile: :wave:");
/// assert_eq!(text, "Hello 😄 👋");
/// ```ignore
pub fn replace_emoji_shortcodes(text: &str) -> String {
    let mapper = EmojiMapper::new();
    mapper.replace_shortcodes(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcode_to_emoji() {
        let mapper = EmojiMapper::new();
        assert_eq!(mapper.shortcode_to_emoji(":smile:"), Some("😄"));
        assert_eq!(mapper.shortcode_to_emoji(":heart:"), Some("❤️"));
        assert_eq!(mapper.shortcode_to_emoji(":unknown:"), None);
    }

    #[test]
    fn test_emoji_to_shortcode() {
        let mapper = EmojiMapper::new();
        assert_eq!(mapper.emoji_to_shortcode("😄"), Some(":smile:".to_string()));
        assert_eq!(mapper.emoji_to_shortcode("❤️"), Some(":heart:".to_string()));
    }

    #[test]
    fn test_replace_shortcodes() {
        let mapper = EmojiMapper::new();
        let text = mapper.replace_shortcodes("Hello :smile: world");
        assert_eq!(text, "Hello 😄 world");
    }

    #[test]
    fn test_replace_multiple_shortcodes() {
        let mapper = EmojiMapper::new();
        let text = mapper.replace_shortcodes(":wave: :smile: :heart:");
        assert_eq!(text, "👋 😄 ❤️");
    }

    #[test]
    fn test_no_shortcodes() {
        let mapper = EmojiMapper::new();
        let text = mapper.replace_shortcodes("Hello world");
        assert_eq!(text, "Hello world");
    }

    #[test]
    fn test_invalid_shortcode() {
        let mapper = EmojiMapper::new();
        let text = mapper.replace_shortcodes("Hello :unknown: world");
        assert_eq!(text, "Hello :unknown: world");
    }

    #[test]
    fn test_has_emoji_shortcode() {
        assert!(has_emoji_shortcode("Hello :smile:"));
        assert!(has_emoji_shortcode(":wave: hello"));
        assert!(!has_emoji_shortcode("Hello world"));
        assert!(!has_emoji_shortcode("Hello : world"));
    }

    #[test]
    fn test_add_custom_mapping() {
        let mut mapper = EmojiMapper::empty();
        mapper.add_mapping("custom", "🎨");
        assert_eq!(mapper.shortcode_to_emoji(":custom:"), Some("🎨"));
    }

    #[test]
    fn test_empty_mapper() {
        let mapper = EmojiMapper::empty();
        assert!(mapper.is_empty());
        assert_eq!(mapper.len(), 0);
    }

    #[test]
    fn test_default_mapper_not_empty() {
        let mapper = EmojiMapper::new();
        assert!(!mapper.is_empty());
        assert!(mapper.len() > 0);
    }

    #[test]
    fn test_replace_emoji_shortcodes_convenience() {
        let text = replace_emoji_shortcodes(":fire: :rocket:");
        assert_eq!(text, "🔥 🚀");
    }
}
