use language_agnostic_analyzer::analyzer::generic_extract;
use language_agnostic_analyzer::language::SupportedLanguage;
use std::fs;

fn main() {
    let source = fs::read_to_string("benchmarks/benchmark.java").unwrap();
    let lang = SupportedLanguage::Java;
    let modules = generic_extract(lang.to_tree_sitter_lang(), &source).unwrap();
    println!("{:#?}", modules);
}
