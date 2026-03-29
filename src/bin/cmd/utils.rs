use std::fs;
use std::io::{self, Read, Write};

/// Read input from file or stdin
pub fn read_input(input_path: Option<&str>) -> anyhow::Result<String> {
    if let Some(path) = input_path {
        fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", path, e))
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| anyhow::anyhow!("Failed to read stdin: {}", e))?;
        Ok(buffer)
    }
}

/// Write output to file or stdout
pub fn write_output(output_path: Option<&str>, content: &str) -> anyhow::Result<()> {
    if let Some(path) = output_path {
        fs::write(path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", path, e))
    } else {
        io::stdout()
            .write_all(content.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write stdout: {}", e))
    }
}

/// Count words in text
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Count characters (excluding whitespace)
pub fn count_chars(text: &str) -> usize {
    text.chars().filter(|c| !c.is_whitespace()).count()
}

/// Generate slug from heading text for anchor links
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .replace(' ', "-")
}
