#![no_main]

use libfuzzer_sys::fuzz_target;
use clmd::Document;

fuzz_target!(|data: &[u8]| {
    // Try to parse the input as a string
    if let Ok(input) = std::str::from_utf8(data) {
        // Parse with default options
        if let Ok(doc) = Document::parse(input) {
            // Test HTML rendering with various options
            let _html = doc.to_html();
            
            // Test XML rendering
            let _xml = doc.to_xml();
            
            // Test CommonMark rendering
            let _commonmark = doc.to_commonmark();
            
            // Test LaTeX rendering
            let _latex = doc.to_latex();
        }
    }
});
