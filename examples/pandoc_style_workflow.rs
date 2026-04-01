//! Example: Pandoc-style workflow using clmd's new architecture
//!
//! This example demonstrates how to use the new pandoc-inspired features:
//! - ClmdMonad for testable IO
//! - Reader/Writer registry for format conversion
//! - Filter chain for document transformation
//! - Template system for rendering
//! - MediaBag for resource management

use clmd::arena::{Node, NodeArena, TreeOps};
use clmd::context::PureContext;
use clmd::error::ClmdResult;
use clmd::formats::mime::get_mime_type;
use clmd::mediabag::MediaBag;
use clmd::options::Options;
use clmd::pipeline::PipelineBuilder;
use clmd::readers::ReaderRegistry;
use clmd::template::{TemplateContext, TemplateEngine};
use clmd::transforms::{Filter, FilterChain};
use clmd::uri::is_uri;
use clmd::writers::WriterRegistry;

fn main() -> ClmdResult<()> {
    println!("=== Pandoc-style Workflow Example ===\n");

    // 1. Using ClmdContext for testable IO
    println!("1. ClmdContext Example:");
    let ctx = PureContext::new();
    ctx.info("Created PureContext for testing");
    println!("   Created PureContext for testing without IO\n");

    // 2. Reader/Writer Registry
    println!("2. Reader/Writer Registry:");
    let reader_registry = ReaderRegistry::default();
    let writer_registry = WriterRegistry::default();

    println!("   Available readers: {:?}", reader_registry.formats());
    println!("   Available writers: {:?}", writer_registry.formats());

    // Get reader by extension
    if let Some(reader) = reader_registry.get_by_extension("md") {
        println!("   Format for .md files: {:?}", reader.format());
    }

    // Get writer by extension
    if let Some(writer) = writer_registry.get_by_extension("html") {
        println!("   Format for .html files: {:?}\n", writer.format());
    }

    // 3. Document Conversion Pipeline
    println!("3. Document Conversion Pipeline:");
    let pipeline = PipelineBuilder::new().from("markdown").to("html").build()?;

    let markdown_input = "# Hello World\n\nThis is a **test** document.";
    let options = Options::default();
    let html_output = pipeline.convert(markdown_input, &options)?;
    println!(
        "   Input (Markdown): {}",
        markdown_input.lines().next().unwrap()
    );
    println!(
        "   Output (HTML): {}...\n",
        &html_output[..html_output.len().min(100)]
    );

    // 4. Filter Chain
    println!("4. Filter Chain:");
    let mut arena = NodeArena::new();
    let root = create_sample_document(&mut arena);

    let mut chain = FilterChain::new();
    chain.add(Filter::header_shift(1)); // Increase header levels

    println!("   Applying header shift filter...");
    chain
        .apply(&mut arena, root)
        .map_err(|e| clmd::error::ClmdError::Other(format!("Filter error: {}", e)))?;
    println!("   Filter applied successfully!\n");

    // 5. Template System
    println!("5. Template System:");
    let _engine = TemplateEngine::new();
    let template = TemplateEngine::default_html_template();

    let mut ctx = TemplateContext::new();
    ctx.set("title", "My Document");
    ctx.set("body", "<h1>Hello World</h1><p>Content here</p>");

    let rendered = template.render(&ctx);
    println!(
        "   Template rendered: {}...\n",
        &rendered[..rendered.len().min(80)]
    );

    // 6. MediaBag Resource Management
    println!("6. MediaBag Resource Management:");
    let mut bag = MediaBag::new();

    // Insert a sample image
    let sample_png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    bag.insert_auto("logo.png", sample_png);

    println!("   Resources in bag: {}", bag.len());
    println!("   Images: {}", bag.images().len());

    // Convert to data URI
    if let Some(data_uri) = bag.to_data_uri("logo.png") {
        println!("   Data URI: {}...\n", &data_uri[..data_uri.len().min(60)]);
    }

    // 7. MIME Type Detection
    println!("7. MIME Type Detection:");
    let files = vec!["image.png", "style.css", "script.js", "document.pdf"];
    for file in files {
        if let Some(mime) = get_mime_type(std::path::Path::new(file)) {
            println!("   {} -> {}", file, mime);
        }
    }
    println!();

    // 8. URI Utilities
    println!("8. URI Utilities:");
    let urls = vec![
        "https://example.com/path",
        "mailto:test@example.com",
        "/local/path",
        "not-a-uri",
    ];
    for url in urls {
        println!("   {} is URI: {}", url, is_uri(url));
    }
    println!();

    println!("=== All examples completed successfully! ===");
    Ok(())
}

/// Helper function to create a sample document for filter testing
fn create_sample_document(arena: &mut NodeArena) -> u32 {
    use clmd::nodes::{NodeHeading, NodeValue};

    let root = arena.alloc(Node::with_value(NodeValue::Document));
    let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
        level: 1,
        setext: false,
        closed: false,
    })));
    TreeOps::append_child(arena, root, heading);
    root
}
