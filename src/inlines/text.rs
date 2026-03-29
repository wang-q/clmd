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
