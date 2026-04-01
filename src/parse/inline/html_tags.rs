//! HTML tag matching for inline parsing

/// Match HTML tag and return the tag content and length
pub fn match_html_tag(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('<') {
        return None;
    }

    // Try different HTML tag types in order

    // 1. HTML Comment: <!-- ... -->
    if let Some(result) = match_html_comment(input) {
        return Some(result);
    }

    // 2. Processing Instruction: <? ... ?>
    if let Some(result) = match_processing_instruction(input) {
        return Some(result);
    }

    // 3. Declaration: <! ... >
    if let Some(result) = match_declaration(input) {
        return Some(result);
    }

    // 4. CDATA: <![CDATA[ ... ]]>
    if let Some(result) = match_cdata(input) {
        return Some(result);
    }

    // 5. Regular HTML tag (open, close, or self-closing)
    if let Some(result) = match_regular_html_tag(input) {
        return Some(result);
    }

    None
}

/// Match HTML comment: <!-- ... -->
fn match_html_comment(input: &str) -> Option<(String, usize)> {
    if !input.starts_with("<!--") {
        return None;
    }

    // Find -->
    if let Some(end) = input.find("-->") {
        return Some((input[..end + 3].to_string(), end + 3));
    }

    None
}

/// Match processing instruction: <? ... ?>
fn match_processing_instruction(input: &str) -> Option<(String, usize)> {
    if !input.starts_with("<?") {
        return None;
    }

    // Find ?>
    if let Some(end) = input.find("?>") {
        return Some((input[..end + 2].to_string(), end + 2));
    }

    None
}

/// Match declaration: <! ... >
/// According to commonmark.js: /^<![A-Za-z]/ - must start with a letter after <!
fn match_declaration(input: &str) -> Option<(String, usize)> {
    if !input.starts_with("<!") || input.starts_with("<![") {
        return None;
    }

    // Declaration must have at least one character after <!
    if input.len() <= 2 {
        return None;
    }

    // Check that the character after <! is an ASCII letter (A-Z or a-z)
    // Per commonmark.js: /^<![A-Za-z]/
    let third_char = input.chars().nth(2)?;
    if !third_char.is_ascii_alphabetic() {
        return None;
    }

    // Find >
    if let Some(end) = input.find('>') {
        // Must not contain < or > inside
        let content = &input[2..end];
        if content.contains('<') || content.contains('>') {
            return None;
        }
        return Some((input[..end + 1].to_string(), end + 1));
    }

    None
}

/// Match CDATA: <![CDATA[ ... ]]>
fn match_cdata(input: &str) -> Option<(String, usize)> {
    if !input.starts_with("<![CDATA[") {
        return None;
    }

    // Find ]]>
    if let Some(end) = input.find("]]>") {
        return Some((input[..end + 3].to_string(), end + 3));
    }

    None
}

/// Match regular HTML tag: open, close, or self-closing
/// Based on commonmark.js regex patterns:
/// TAGNAME = "[A-Za-z][A-Za-z0-9-]*"
/// ATTRIBUTENAME = "[a-zA-Z_:][a-zA-Z0-9:._-]*"
/// ATTRIBUTE = "(?:\\s+" + ATTRIBUTENAME + ATTRIBUTEVALUESPEC + "?)"
/// OPENTAG = "<" + TAGNAME + ATTRIBUTE + "*" + "\\s*/?>"
fn match_regular_html_tag(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('<') {
        return None;
    }

    let rest = &input[1..];

    // Check for close tag: </tag>
    if rest.starts_with('/') {
        return match_close_tag(input);
    }

    // Must start with a letter for tag name (not whitespace)
    let first_char = rest.chars().next()?;
    if !first_char.is_ascii_alphabetic() {
        return None;
    }

    // Parse tag name: [A-Za-z][A-Za-z0-9-]*
    let mut i = 1; // Skip the '<'
    for c in rest.chars() {
        if c.is_ascii_alphanumeric() || c == '-' {
            i += 1;
        } else {
            break;
        }
    }

    // Parse attributes: (whitespace+ attribute_name value?)*
    // ATTRIBUTENAME = "[a-zA-Z_:][a-zA-Z0-9:._-]*"
    // First skip whitespace after tag name
    while i < input.len() {
        let c = input.chars().nth(i)?;
        if c.is_ascii_whitespace() {
            i += 1;
        } else {
            break;
        }
    }

    // Track if the previous attribute was a boolean attribute
    // If so, we already skipped the whitespace after it
    let mut after_boolean_attr = false;

    loop {
        if i >= input.len() {
            break;
        }

        let c = input.chars().nth(i)?;

        // Check for end of tag
        if c == '>' {
            return Some((input[..i + 1].to_string(), i + 1));
        }

        // Check for self-closing tag />
        if c == '/' {
            if i + 1 < input.len() && input.chars().nth(i + 1)? == '>' {
                return Some((input[..i + 2].to_string(), i + 2));
            }
            // '/' not followed by '>' is invalid
            return None;
        }

        // Parse attribute name: [a-zA-Z_:][a-zA-Z0-9:._-]*
        let first_attr_char = input.chars().nth(i)?;
        if !first_attr_char.is_ascii_alphabetic()
            && first_attr_char != '_'
            && first_attr_char != ':'
        {
            return None;
        }
        i += 1;

        while i < input.len() {
            let c = input.chars().nth(i)?;
            if c.is_ascii_alphanumeric() || c == ':' || c == '_' || c == '.' || c == '-'
            {
                i += 1;
            } else {
                break;
            }
        }

        // Check for attribute value
        // Skip whitespace after attribute name (before =)
        // But only if we didn't already skip it (i.e., not after a boolean attribute)
        if !after_boolean_attr {
            while i < input.len() {
                let ws_char = input.chars().nth(i)?;
                if ws_char.is_ascii_whitespace() {
                    i += 1;
                } else {
                    break;
                }
            }
        }
        after_boolean_attr = false;

        if i < input.len() {
            let c = input.chars().nth(i)?;
            if c == '=' {
                i += 1;
                // Skip whitespace after =
                while i < input.len() {
                    let ws_char = input.chars().nth(i)?;
                    if ws_char.is_ascii_whitespace() {
                        i += 1;
                    } else {
                        break;
                    }
                }
                // Parse attribute value
                if i >= input.len() {
                    return None;
                }
                let val_char = input.chars().nth(i)?;
                if val_char == '"' || val_char == '\'' {
                    // Quoted value
                    let quote = val_char;
                    i += 1;
                    while i < input.len() {
                        let c = input.chars().nth(i)?;
                        if c == quote {
                            i += 1;
                            break;
                        }
                        i += 1;
                    }
                } else {
                    // Unquoted value: [^"'=<>`\x00-\x20]+
                    // Note: CommonMark allows = in unquoted values (test #616)
                    while i < input.len() {
                        let c = input.chars().nth(i)?;
                        if c == '"'
                            || c == '\''
                            || c == '<'
                            || c == '>'
                            || c == '`'
                            || c.is_ascii_whitespace()
                        {
                            break;
                        }
                        i += 1;
                    }
                }
            } else {
                // If no '=', this is a boolean attribute
                // The whitespace after the attribute name has already been skipped above
                // Set flag so we don't skip it again on the next iteration
                after_boolean_attr = true;
            }
        }

        // Skip whitespace before next attribute (or end of tag)
        // But only if we didn't just parse a boolean attribute
        if !after_boolean_attr {
            let mut ws_count = 0;
            while i < input.len() {
                let c = input.chars().nth(i)?;
                if c.is_ascii_whitespace() {
                    i += 1;
                    ws_count += 1;
                } else {
                    break;
                }
            }

            // If we didn't see whitespace and we're not at the end of the tag, invalid
            // (attributes must be separated by whitespace)
            if ws_count == 0 && i < input.len() {
                let c = input.chars().nth(i)?;
                if c != '>' && c != '/' {
                    return None;
                }
            }
        }
    }

    None
}

/// Match close tag: </tag>
/// Allows whitespace between tag name and >
fn match_close_tag(input: &str) -> Option<(String, usize)> {
    if !input.starts_with("</") {
        return None;
    }

    let rest = &input[2..];

    // Must start with a letter
    let first_char = rest.chars().next()?;
    if !first_char.is_ascii_alphabetic() {
        return None;
    }

    // Parse tag name
    let mut i = 2; // Skip the '</'
    for c in rest.chars() {
        if c.is_ascii_alphanumeric() || c == '-' {
            i += 1;
        } else {
            break;
        }
    }

    // Skip whitespace
    while i < input.len() {
        let c = input.chars().nth(i)?;
        if c.is_ascii_whitespace() {
            i += 1;
        } else {
            break;
        }
    }

    // Expect >
    if i < input.len() && input.chars().nth(i)? == '>' {
        return Some((input[..i + 1].to_string(), i + 1));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_html_comment() {
        // Valid HTML comments
        let result = match_html_tag("<!-- comment -->");
        assert!(result.is_some());
        let (tag, len) = result.unwrap();
        assert_eq!(tag, "<!-- comment -->");
        assert_eq!(len, 16);

        let result = match_html_tag("<!--simple-->");
        assert!(result.is_some());
        let (tag, _) = result.unwrap();
        assert_eq!(tag, "<!--simple-->");

        // Invalid - no closing -->
        let result = match_html_tag("<!-- unclosed");
        assert!(result.is_none());

        // Not a comment
        let result = match_html_tag("<not-a-comment>");
        assert!(result.is_some()); // Matches as regular tag
    }

    #[test]
    fn test_match_processing_instruction() {
        // Valid processing instruction
        let result = match_html_tag("<?xml version=\"1.0\"?>");
        assert!(result.is_some());
        let (tag, len) = result.unwrap();
        assert_eq!(tag, "<?xml version=\"1.0\"?>");
        assert_eq!(len, 21);

        // Invalid - no closing ?>
        let result = match_html_tag("<?php echo");
        assert!(result.is_none());
    }

    #[test]
    fn test_match_declaration() {
        // Valid declaration
        let result = match_html_tag("<!DOCTYPE html>");
        assert!(result.is_some());
        let (tag, len) = result.unwrap();
        assert_eq!(tag, "<!DOCTYPE html>");
        assert_eq!(len, 15);

        // Invalid - must start with letter after <!
        let result = match_html_tag("<!123>");
        assert!(result.is_none());

        // CDATA should not match as declaration
        let result = match_html_tag("<![CDATA[ test ]]>");
        assert!(result.is_some());
        let (tag, _) = result.unwrap();
        assert_eq!(tag, "<![CDATA[ test ]]>");
    }

    #[test]
    fn test_match_cdata() {
        // Valid CDATA
        let result = match_html_tag("<![CDATA[ some <b>content</b> ]]>");
        assert!(result.is_some());
        let (tag, _len) = result.unwrap();
        assert_eq!(tag, "<![CDATA[ some <b>content</b> ]]>");

        // Invalid - no closing ]]>
        let result = match_html_tag("<![CDATA[ unclosed");
        assert!(result.is_none());
    }

    #[test]
    fn test_match_regular_html_tag_open() {
        // Simple open tag
        let result = match_html_tag("<div>");
        assert!(result.is_some());
        let (tag, len) = result.unwrap();
        assert_eq!(tag, "<div>");
        assert_eq!(len, 5);

        // Tag with attributes
        let result = match_html_tag("<div class=\"container\" id=\"main\">");
        assert!(result.is_some());
        let (tag, _) = result.unwrap();
        assert_eq!(tag, "<div class=\"container\" id=\"main\">");

        // Self-closing tag
        let result = match_html_tag("<br/>");
        assert!(result.is_some());
        let (tag, _) = result.unwrap();
        assert_eq!(tag, "<br/>");

        let result = match_html_tag("<img src=\"test.jpg\" />");
        assert!(result.is_some());
        let (tag, _) = result.unwrap();
        assert_eq!(tag, "<img src=\"test.jpg\" />");
    }

    #[test]
    fn test_match_regular_html_tag_close() {
        // Simple close tag
        let result = match_html_tag("</div>");
        assert!(result.is_some());
        let (tag, len) = result.unwrap();
        assert_eq!(tag, "</div>");
        assert_eq!(len, 6);

        // Close tag with whitespace
        let result = match_html_tag("</div  >");
        assert!(result.is_some());
        let (tag, _) = result.unwrap();
        assert_eq!(tag, "</div  >");
    }

    #[test]
    fn test_match_html_tag_invalid() {
        // Invalid - no <
        let result = match_html_tag("not-a-tag");
        assert!(result.is_none());

        // Invalid - starts with number
        let result = match_html_tag("<123>");
        assert!(result.is_none());

        // Invalid - just <
        let result = match_html_tag("<");
        assert!(result.is_none());
    }

    #[test]
    fn test_match_html_comment_function() {
        // Direct test of match_html_comment
        let result = match_html_comment("<!-- test -->");
        assert!(result.is_some());
        let (tag, len) = result.unwrap();
        assert_eq!(tag, "<!-- test -->");
        assert_eq!(len, 13);

        // Not a comment
        let result = match_html_comment("<div>");
        assert!(result.is_none());
    }

    #[test]
    fn test_match_processing_instruction_function() {
        // Direct test
        let result = match_processing_instruction("<?php ?>");
        assert!(result.is_some());

        // Not a PI
        let result = match_processing_instruction("<div>");
        assert!(result.is_none());
    }

    #[test]
    fn test_match_declaration_function() {
        // Valid
        let result = match_declaration("<!DOCTYPE>");
        assert!(result.is_some());

        // Invalid - starts with [
        let result = match_declaration("<![CDATA[]]>");
        assert!(result.is_none());

        // Invalid - starts with number
        let result = match_declaration("<!123>");
        assert!(result.is_none());
    }

    #[test]
    fn test_match_cdata_function() {
        // Valid
        let result = match_cdata("<![CDATA[test]]>");
        assert!(result.is_some());

        // Not CDATA
        let result = match_cdata("<div>");
        assert!(result.is_none());
    }

    #[test]
    fn test_match_close_tag_function() {
        // Valid
        let result = match_close_tag("</div>");
        assert!(result.is_some());
        let (tag, len) = result.unwrap();
        assert_eq!(tag, "</div>");
        assert_eq!(len, 6);

        // With whitespace
        let result = match_close_tag("</div  >");
        assert!(result.is_some());

        // Invalid - starts with number
        let result = match_close_tag("</123>");
        assert!(result.is_none());

        // Not a close tag
        let result = match_close_tag("<div>");
        assert!(result.is_none());
    }
}
