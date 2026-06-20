use language_agnostic_analyzer::language::SupportedLanguage;
use std::fs;

fn main() {
    let source = fs::read_to_string("benchmarks/benchmark.java").unwrap();
    let lang = SupportedLanguage::Java;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang.to_tree_sitter_lang()).unwrap();
    let tree = parser.parse(&source, None).unwrap();
    
    let mut cursor = tree.walk();
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        if language_agnostic_analyzer::heuristics::classifiers::is_structured_type(node) {
            println!("Matched: {} -> {}", node.kind(), node.utf8_text(source.as_bytes()).unwrap().split('\n').next().unwrap());
        }
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }
}
