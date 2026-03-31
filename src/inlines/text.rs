//! Text processing and smart punctuation for inline parsing

/// Apply smart punctuation transformations
/// Based on commonmark.js: replace ellipses and dashes
pub fn apply_smart_punctuation(text: &str) -> String {
    // First handle ellipses: ... -> …
    let text = text.replace("...", "\u{2026}");

    // Then handle dashes
    // We need to be careful about the order: --- should be matched before --
    // Based on commonmark.js logic:
    // - --- -> — (em dash)
    // - -- -> – (en dash)
    // But for sequences like ----, we need to apply rules for multiple dashes

    let mut result = String::new();
    let mut dash_count = 0;

    for c in text.chars() {
        if c == '-' {
            dash_count += 1;
        } else {
            // Process any accumulated dashes
            if dash_count > 0 {
                result.push_str(&convert_dashes(dash_count));
                dash_count = 0;
            }
            result.push(c);
        }
    }

    // Process any trailing dashes
    if dash_count > 0 {
        result.push_str(&convert_dashes(dash_count));
    }

    result
}

/// Convert a sequence of dashes to em/en dashes
/// Based on commonmark.js logic from smart_punct.txt:
/// - A homogeneous sequence is preferred (all en or all em)
/// - 10 hyphens = 5 en dashes
/// - 9 hyphens = 3 em dashes
/// - 6 hyphens = 2 em dashes (3 is multiple of 3, preferred)
/// - 7 hyphens = 2 em + 1 en (when homogeneous is not possible)
/// - em dashes come first, then en dashes, with as few en dashes as possible
fn convert_dashes(count: usize) -> String {
    if count == 1 {
        return "-".to_string();
    }

    let mut result = String::new();

    // Try to use homogeneous sequence first
    // Prefer em dashes when divisible by 3 (3, 6, 9, ...)
    if count % 3 == 0 {
        // Divisible by 3: use all em dashes
        for _ in 0..(count / 3) {
            result.push('\u{2014}'); // em dash
        }
    } else if count % 2 == 0 {
        // Even number but not divisible by 3: use all en dashes
        for _ in 0..(count / 2) {
            result.push('\u{2013}'); // en dash
        }
    } else {
        // Not homogeneous: use em dashes first, then en dashes
        // Use as many em dashes as possible, then fill with en dashes
        let mut remaining = count;

        // Try to minimize en dashes
        // Start with as many em dashes as possible
        while remaining > 4 {
            result.push('\u{2014}'); // em dash
            remaining -= 3;
        }

        // Handle remaining (should be 2, 4, or 5 at this point)
        match remaining {
            2 => result.push('\u{2013}'),             // en dash
            4 => result.push_str("\u{2013}\u{2013}"), // 2 en dashes
            5 => result.push_str("\u{2014}\u{2013}"), // em + en
            _ => {}                                   // should not happen
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_smart_punctuation_ellipsis() {
        // Test ellipsis conversion
        let result = apply_smart_punctuation("Hello...world");
        assert_eq!(result, "Hello\u{2026}world");

        let result = apply_smart_punctuation("...");
        assert_eq!(result, "\u{2026}");

        let result = apply_smart_punctuation("a...b...c");
        assert_eq!(result, "a\u{2026}b\u{2026}c");
    }

    #[test]
    fn test_apply_smart_punctuation_dashes() {
        // Single dash stays as hyphen
        let result = apply_smart_punctuation("co-operate");
        assert_eq!(result, "co-operate");

        // Double dash -> en dash
        let result = apply_smart_punctuation("pages 1--10");
        assert_eq!(result, "pages 1\u{2013}10");

        // Triple dash -> em dash
        let result = apply_smart_punctuation("Hello---world");
        assert_eq!(result, "Hello\u{2014}world");
    }

    #[test]
    fn test_apply_smart_punctuation_mixed() {
        // Mixed ellipsis and dashes
        let result = apply_smart_punctuation("Well...---you know");
        assert_eq!(result, "Well\u{2026}\u{2014}you know");
    }

    #[test]
    fn test_convert_dashes() {
        // Single dash
        assert_eq!(convert_dashes(1), "-");

        // Two dashes -> en dash
        assert_eq!(convert_dashes(2), "\u{2013}");

        // Three dashes -> em dash
        assert_eq!(convert_dashes(3), "\u{2014}");

        // Four dashes -> two en dashes
        assert_eq!(convert_dashes(4), "\u{2013}\u{2013}");

        // Five dashes -> em + en
        assert_eq!(convert_dashes(5), "\u{2014}\u{2013}");

        // Six dashes -> two em dashes
        assert_eq!(convert_dashes(6), "\u{2014}\u{2014}");

        // Seven dashes -> em + two en (7 = 3 + 4, one em dash then two en dashes)
        assert_eq!(convert_dashes(7), "\u{2014}\u{2013}\u{2013}");

        // Eight dashes -> four en dashes
        assert_eq!(convert_dashes(8), "\u{2013}\u{2013}\u{2013}\u{2013}");

        // Nine dashes -> three em dashes
        assert_eq!(convert_dashes(9), "\u{2014}\u{2014}\u{2014}");

        // Ten dashes -> five en dashes
        assert_eq!(
            convert_dashes(10),
            "\u{2013}\u{2013}\u{2013}\u{2013}\u{2013}"
        );
    }

    #[test]
    fn test_apply_smart_punctuation_no_change() {
        // Text without special sequences
        let result = apply_smart_punctuation("Just plain text");
        assert_eq!(result, "Just plain text");

        // Single dashes should remain
        let result = apply_smart_punctuation("a-b-c");
        assert_eq!(result, "a-b-c");
    }
}
