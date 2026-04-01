//! Autolink matching for inline parsing

/// Email autolink pattern
/// Based on commonmark.js reEmailAutolink
pub fn match_email_autolink(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('<') {
        return None;
    }

    // Email pattern from commonmark.js:
    // /^<([a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*>/
    let rest = &input[1..];

    // Check for valid email characters in local part (before @)
    let mut chars = rest.chars().peekable();
    let mut i = 0;
    let mut found_at = false;

    // Local part: [a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+
    while let Some(&c) = chars.peek() {
        if c == '@' {
            found_at = true;
            chars.next();
            i += 1;
            break;
        } else if c == '>' || c == '<' || c == '\n' || c == ' ' || c == '\t' {
            return None;
        } else if c == '\\' {
            // Backslash escape is not allowed in email autolinks
            return None;
        } else if is_valid_email_local_char(c) {
            chars.next();
            i += 1;
        } else {
            return None;
        }
    }

    if !found_at || i <= 1 {
        return None;
    }

    // Domain part: [a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*
    let domain_start = i;
    let mut label_start = i;

    while let Some(&c) = chars.peek() {
        if c == '>' {
            // End of email
            if i > domain_start && i > label_start {
                let email = &rest[..i];
                return Some((email.to_string(), i + 2)); // +2 for < and >
            }
            return None;
        } else if c == '<' || c == '\n' || c == ' ' || c == '\t' {
            return None;
        } else if c == '.' {
            chars.next();
            i += 1;
            label_start = i;
        } else if c.is_ascii_alphanumeric() || c == '-' {
            chars.next();
            i += 1;
        } else {
            return None;
        }
    }

    None
}

fn is_valid_email_local_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            '.' | '!'
                | '#'
                | '$'
                | '%'
                | '&'
                | '\''
                | '*'
                | '+'
                | '/'
                | '='
                | '?'
                | '^'
                | '_'
                | '`'
                | '{'
                | '|'
                | '}'
                | '~'
                | '-'
        )
}

/// URL autolink pattern
/// Based on commonmark.js reAutolink: /^<[A-Za-z][A-Za-z0-9.+-]{1,31}:[^<>\x00-\x20]*>/i
pub fn match_url_autolink(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('<') {
        return None;
    }

    // URL pattern: <scheme:...>
    let rest = &input[1..];

    // Must start with a letter, then letters/digits/+/-/.
    let mut i = 0;
    let mut has_colon = false;

    for c in rest.chars() {
        if c == ':' {
            has_colon = true;
            i += 1;
            break;
        } else if c.is_ascii_alphabetic()
            || c.is_ascii_digit()
            || c == '+'
            || c == '-'
            || c == '.'
        {
            if i == 0 && !c.is_ascii_alphabetic() {
                return None;
            }
            i += 1;
            if i > 32 {
                return None; // Scheme too long
            }
        } else {
            return None;
        }
    }

    if !has_colon || i < 3 {
        // Scheme must be at least 2 characters (i includes the colon, so i >= 3 means scheme >= 2)
        return None;
    }

    // Now parse the rest of the URL
    // [^<>\x00-\x20]* means: no <, >, or ASCII control characters/space
    let url_start = i;
    let mut end_pos = 0;

    for (j, c) in rest[url_start..].chars().enumerate() {
        if c == '>' {
            end_pos = url_start + j;
            break;
        } else if c == '\n' || c == '<' || c == ' ' || c == '\t' || c.is_ascii_control()
        {
            // Space or control character - invalid URL
            return None;
        }
    }

    if end_pos > url_start {
        let url = &rest[..end_pos];
        return Some((url.to_string(), end_pos + 2)); // +2 for < and >
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_email_autolink_valid() {
        // Valid email autolinks
        let result = match_email_autolink("<test@example.com>");
        assert!(result.is_some());
        let (email, len) = result.unwrap();
        assert_eq!(email, "test@example.com");
        assert_eq!(len, 18); // <test@example.com> = 18 chars

        let result = match_email_autolink("<user.name@example.co.uk>");
        assert!(result.is_some());
        let (email, _) = result.unwrap();
        assert_eq!(email, "user.name@example.co.uk");

        let result = match_email_autolink("<user+tag@example.com>");
        assert!(result.is_some());
        let (email, _) = result.unwrap();
        assert_eq!(email, "user+tag@example.com");
    }

    #[test]
    fn test_match_email_autolink_invalid() {
        // Invalid email autolinks
        assert!(match_email_autolink("not_an_email").is_none());
        assert!(match_email_autolink("<@example.com>").is_none()); // No local part
        assert!(match_email_autolink("<test@>").is_none()); // No domain
        assert!(match_email_autolink("<test@example").is_none()); // No closing >
        assert!(match_email_autolink("test@example.com>").is_none()); // No opening <
        assert!(match_email_autolink("<test@@example.com>").is_none()); // Double @
        assert!(match_email_autolink("<test@exam ple.com>").is_none()); // Space in domain
    }

    #[test]
    fn test_match_email_autolink_special_chars() {
        // Email with special characters in local part
        let result = match_email_autolink("<user!#$%&'*+/=?^_`{|}~-@example.com>");
        assert!(result.is_some());

        // Backslash is not allowed
        assert!(match_email_autolink("<user\\name@example.com>").is_none());
    }

    #[test]
    fn test_match_url_autolink_valid() {
        // Valid URL autolinks
        let result = match_url_autolink("<https://example.com>");
        assert!(result.is_some());
        let (url, len) = result.unwrap();
        assert_eq!(url, "https://example.com");
        assert_eq!(len, 21); // <https://example.com> = 21 chars

        let result = match_url_autolink("<http://localhost:8080/path>");
        assert!(result.is_some());
        let (url, _) = result.unwrap();
        assert_eq!(url, "http://localhost:8080/path");

        let result = match_url_autolink("<ftp://files.example.com>");
        assert!(result.is_some());
        let (url, _) = result.unwrap();
        assert_eq!(url, "ftp://files.example.com");
    }

    #[test]
    fn test_match_url_autolink_invalid() {
        // Invalid URL autolinks
        assert!(match_url_autolink("not_a_url").is_none());
        assert!(match_url_autolink("<123://example.com>").is_none()); // Scheme must start with letter
        assert!(match_url_autolink("<a:short>").is_none()); // Scheme too short
        assert!(match_url_autolink("<http://example.com").is_none()); // No closing >
        assert!(match_url_autolink("http://example.com>").is_none()); // No opening <
    }

    #[test]
    fn test_match_url_autolink_scheme_length() {
        // Scheme with max length (32 chars)
        let long_scheme = format!("<{}:test>", "a".repeat(31));
        assert!(match_url_autolink(&long_scheme).is_some());

        // Scheme too long (>32 chars)
        let too_long_scheme = format!("<{}:test>", "a".repeat(33));
        assert!(match_url_autolink(&too_long_scheme).is_none());
    }

    #[test]
    fn test_is_valid_email_local_char() {
        assert!(is_valid_email_local_char('a'));
        assert!(is_valid_email_local_char('Z'));
        assert!(is_valid_email_local_char('0'));
        assert!(is_valid_email_local_char('!'));
        assert!(is_valid_email_local_char('#'));
        assert!(is_valid_email_local_char('$'));
        assert!(is_valid_email_local_char('&'));
        assert!(is_valid_email_local_char('*'));
        assert!(is_valid_email_local_char('+'));
        assert!(is_valid_email_local_char('-'));
        assert!(is_valid_email_local_char('/'));
        assert!(is_valid_email_local_char('='));
        assert!(is_valid_email_local_char('?'));
        assert!(is_valid_email_local_char('^'));
        assert!(is_valid_email_local_char('_'));
        assert!(is_valid_email_local_char('`'));
        assert!(is_valid_email_local_char('{'));
        assert!(is_valid_email_local_char('|'));
        assert!(is_valid_email_local_char('}'));
        assert!(is_valid_email_local_char('~'));
        assert!(is_valid_email_local_char('.'));
        assert!(is_valid_email_local_char('\''));
        assert!(is_valid_email_local_char('%'));

        // Invalid characters
        assert!(!is_valid_email_local_char('@'));
        assert!(!is_valid_email_local_char(' '));
        assert!(!is_valid_email_local_char('<'));
        assert!(!is_valid_email_local_char('>'));
        assert!(!is_valid_email_local_char('\\'));
    }
}
