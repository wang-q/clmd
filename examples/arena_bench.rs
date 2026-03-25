//! Arena-based parser benchmark
//!
//! This example compares the performance of Rc<RefCell> vs Arena-based node allocation.

use std::time::Instant;

// Simplified node structure for benchmarking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeType {
    Document,
    Paragraph,
    Text,
}

// Rc<RefCell> version (current implementation style)
mod rc_version {
    use super::NodeType;
    use std::cell::RefCell;
    use std::rc::Rc;

    pub struct Node {
        pub node_type: NodeType,
        pub parent: RefCell<Option<std::rc::Weak<RefCell<Node>>>>,
        pub first_child: RefCell<Option<Rc<RefCell<Node>>>>,
        pub last_child: RefCell<Option<Rc<RefCell<Node>>>>,
        pub next: RefCell<Option<Rc<RefCell<Node>>>>,
        pub prev: RefCell<Option<std::rc::Weak<RefCell<Node>>>>,
    }

    impl Node {
        pub fn new(node_type: NodeType) -> Rc<RefCell<Self>> {
            Rc::new(RefCell::new(Self {
                node_type,
                parent: RefCell::new(None),
                first_child: RefCell::new(None),
                last_child: RefCell::new(None),
                next: RefCell::new(None),
                prev: RefCell::new(None),
            }))
        }
    }

    pub fn append_child(parent: &Rc<RefCell<Node>>, child: Rc<RefCell<Node>>) {
        let parent_ref = parent.borrow();

        if let Some(ref last_child) = *parent_ref.last_child.borrow() {
            // Link child to previous last child
            *child.borrow().prev.borrow_mut() = Some(Rc::downgrade(last_child));
            *last_child.borrow().next.borrow_mut() = Some(child.clone());
        } else {
            // No children yet, set as first child
            *parent_ref.first_child.borrow_mut() = Some(child.clone());
        }

        // Always update last_child
        *parent_ref.last_child.borrow_mut() = Some(child.clone());
        *child.borrow().parent.borrow_mut() = Some(Rc::downgrade(parent));
    }

    /// Build a simple document tree with n paragraphs
    pub fn build_tree(n: usize) -> Rc<RefCell<Node>> {
        let doc = Node::new(NodeType::Document);

        for _ in 0..n {
            let para = Node::new(NodeType::Paragraph);

            // Add some text children
            for _ in 0..5 {
                let text = Node::new(NodeType::Text);
                append_child(&para, text);
            }

            append_child(&doc, para);
        }

        doc
    }

    /// Traverse the tree
    pub fn traverse(node: &Rc<RefCell<Node>>) -> usize {
        let mut count = 1;

        if let Some(ref first_child) = *node.borrow().first_child.borrow() {
            count += traverse(first_child);
        }

        if let Some(ref next) = *node.borrow().next.borrow() {
            count += traverse(next);
        }

        count
    }
}

// Arena version (proposed implementation)
mod arena_version {
    use super::NodeType;

    pub type NodeId = u32;

    pub struct Node {
        pub node_type: NodeType,
        pub parent: Option<NodeId>,
        pub first_child: Option<NodeId>,
        pub last_child: Option<NodeId>,
        pub next: Option<NodeId>,
        pub prev: Option<NodeId>,
    }

    impl Node {
        pub fn new(node_type: NodeType) -> Self {
            Self {
                node_type,
                parent: None,
                first_child: None,
                last_child: None,
                next: None,
                prev: None,
            }
        }
    }

    pub struct Arena {
        nodes: Vec<Node>,
    }

    impl Arena {
        pub fn new() -> Self {
            Self { nodes: Vec::new() }
        }

        pub fn alloc(&mut self, node: Node) -> NodeId {
            let id = self.nodes.len() as NodeId;
            self.nodes.push(node);
            id
        }

        pub fn get(&self, id: NodeId) -> &Node {
            &self.nodes[id as usize]
        }

        pub fn get_mut(&mut self, id: NodeId) -> &mut Node {
            &mut self.nodes[id as usize]
        }
    }

    pub fn append_child(arena: &mut Arena, parent_id: NodeId, child_id: NodeId) {
        let parent = arena.get_mut(parent_id);

        if let Some(last_child_id) = parent.last_child {
            let last_child = arena.get_mut(last_child_id);
            last_child.next = Some(child_id);

            let child = arena.get_mut(child_id);
            child.prev = Some(last_child_id);
        } else {
            let parent = arena.get_mut(parent_id);
            parent.first_child = Some(child_id);
        }

        let parent = arena.get_mut(parent_id);
        parent.last_child = Some(child_id);

        let child = arena.get_mut(child_id);
        child.parent = Some(parent_id);
    }

    /// Build a simple document tree with n paragraphs
    pub fn build_tree(n: usize) -> (Arena, NodeId) {
        let mut arena = Arena::new();
        let doc = arena.alloc(Node::new(NodeType::Document));

        for _ in 0..n {
            let para = arena.alloc(Node::new(NodeType::Paragraph));

            // Add some text children
            for _ in 0..5 {
                let text = arena.alloc(Node::new(NodeType::Text));
                append_child(&mut arena, para, text);
            }

            append_child(&mut arena, doc, para);
        }

        (arena, doc)
    }

    /// Traverse the tree
    pub fn traverse(arena: &Arena, node_id: NodeId) -> usize {
        let mut count = 1;
        let node = arena.get(node_id);

        if let Some(first_child) = node.first_child {
            count += traverse(arena, first_child);
        }

        if let Some(next) = node.next {
            count += traverse(arena, next);
        }

        count
    }
}

fn main() {
    println!("Arena vs Rc<RefCell> Performance Comparison\n");

    let sizes = [100, 1000, 10000];
    let iterations = 1000;

    for &size in &sizes {
        println!("--- Tree size: {} paragraphs ---", size);

        // Benchmark Rc<RefCell> version
        let start = Instant::now();
        for _ in 0..iterations {
            let doc = rc_version::build_tree(size);
            let count = rc_version::traverse(&doc);
            assert_eq!(count, 1 + size * 6); // doc + n * (para + 5 text)
        }
        let rc_time = start.elapsed();

        // Benchmark Arena version
        let start = Instant::now();
        for _ in 0..iterations {
            let (arena, doc) = arena_version::build_tree(size);
            let count = arena_version::traverse(&arena, doc);
            assert_eq!(count, 1 + size * 6);
        }
        let arena_time = start.elapsed();

        println!(
            "  Rc<RefCell>:  {:?} ({:?} per iteration)",
            rc_time,
            rc_time / iterations
        );
        println!(
            "  Arena:        {:?} ({:?} per iteration)",
            arena_time,
            arena_time / iterations
        );

        if arena_time < rc_time {
            let speedup = rc_time.as_secs_f64() / arena_time.as_secs_f64();
            println!("  Arena is {:.2}x faster", speedup);
        } else {
            let slowdown = arena_time.as_secs_f64() / rc_time.as_secs_f64();
            println!("  Arena is {:.2}x slower", slowdown);
        }
        println!();
    }

    println!("\nConclusion:");
    println!("If Arena shows significant speedup (>20%), it's worth migrating the full codebase.");
    println!("Otherwise, the current Rc<RefCell> approach is acceptable.");
}
