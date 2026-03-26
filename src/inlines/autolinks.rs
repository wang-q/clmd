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
