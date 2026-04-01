//! Example: Pandoc-style workflow using clmd's new architecture
//!
//! This example demonstrates how to use the new pandoc-inspired features:
//! - ClmdMonad for testable IO
//! - Reader/Writer registry for format conversion
//! - Filter chain for document transformation
//! - Template system for rendering
//! - MediaBag for resource management

use clmd::context::mediabag::MediaBag;
use clmd::context::PureContext;
use clmd::core::arena::{Node, NodeArena, TreeOps};
use clmd::core::error::ClmdResult;
use clmd::options::Options;
use clmd::readers::ReaderRegistry;
use clmd::template::{TemplateContext, TemplateEngine};
use clmd::text::uri::is_uri;
use clmd::transforms::{Filter, FilterChain};
use clmd::writers::WriterRegistry;

fn main() -> ClmdResult<()> {
    println!("=== Pandoc-style Workflow Example ===\n");

    // 1. Using ClmdContext for testable IO
    println!("1. ClmdContext Example:");
    let ctx = PureContext::new();
    ctx.info("Created PureContext for testing");
    println!("   Created PureContext for testing without IO\n");

    // 2. Reader Registry
    println!("2. Reader Registry:");
    let readers = ReaderRegistry::new();
    println!("   Reader registry created successfully");

    // 3. Writer Registry
    println!("\n3. Writer Registry:");
    let writers = WriterRegistry::new();
    println!("   Writer registry created successfully");

    // 4. Options and Pipeline
    println!("\n4. Options:");
    let _options = Options::default();
    println!("   Options created successfully!");

    // 5. Filter Chain
    println!("\n5. Filter Chain:");
    let mut arena = NodeArena::new();
    let root = create_sample_document(&mut arena);

    let mut chain = FilterChain::new();
    chain.add(Filter::header_shift(1)); // Increase header levels

    println!("   Applying header shift filter...");
    chain.apply(&mut arena, root).map_err(|e| {
        clmd::core::error::ClmdError::Other(format!("Filter error: {}", e))
    })?;
    println!("   Filter applied successfully!\n");

    // 6. Template System
    println!("6. Template System:");
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

    // 7. MediaBag Resource Management
    println!("7. MediaBag Resource Management:");
    let mut bag = MediaBag::new();

    // Insert a sample image
    let sample_png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    bag.insert_auto("logo.png", sample_png);

    println!("   Resources in bag: {}", bag.len());
    println!("   Images: {}", bag.images().len());

    // Convert to data URI
    if let Some(data_uri) = bag.to_data_uri("logo.png") {
        let preview_len = data_uri.len().min(60);
        println!("   Data URI: {}...\n", &data_uri[..preview_len]);
    }

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
    use clmd::core::nodes::{NodeHeading, NodeValue};

    let root = arena.alloc(Node::with_value(NodeValue::Document));
    let heading = arena.alloc(Node::with_value(NodeValue::Heading(NodeHeading {
        level: 1,
        setext: false,
        closed: false,
    })));
    TreeOps::append_child(arena, root, heading);
    root
}
