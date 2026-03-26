#![no_main]

use libfuzzer_sys::fuzz_target;
use clmd::Document;

fuzz_target!(|data: &[u8]| {
    // Try to parse the input as a string
    if let Ok(input) = std::str::from_utf8(data) {
        // Try to parse the document
        if let Ok(doc) = Document::parse(input) {
            // Try to render to various formats
            // These should not panic even with malformed input
            let _html = doc.to_html();
            let _xml = doc.to_xml();
            let _commonmark = doc.to_commonmark();
        }
    }
});
