use language_agnostic_analyzer::language::SupportedLanguage;
use std::fs;

fn main() {
    let source = "class Robot { void start() { Cat myCat = new Cat(\"Tom\"); } }";
    let lang = SupportedLanguage::Java;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang.to_tree_sitter_lang()).unwrap();
    let tree = parser.parse(source, None).unwrap();
    
    let fields = language_agnostic_analyzer::heuristics::structural_extraction::extract_fields(tree.root_node(), source);
    println!("{:#?}", fields);
}
