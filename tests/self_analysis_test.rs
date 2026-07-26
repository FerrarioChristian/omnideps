use language_agnostic_analyzer::analyzer::{analyze_project, parse_source};
use language_agnostic_analyzer::config::AnalyzerConfig;
use language_agnostic_analyzer::language::SupportedLanguage;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[test]
fn test_self_analysis() {
    let src_dir = Path::new("src");
    assert!(src_dir.exists(), "The src directory does not exist.");

    let outputs_dir = Path::new("tests/outputs");
    if !outputs_dir.exists() {
        fs::create_dir_all(outputs_dir).expect("Unable to create outputs directory for tests.");
    }

    let mut all_graphs = vec![];
    let config = AnalyzerConfig::default();

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file() {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

            // Analyze only Rust files in the src directory
            if ext == "rs" {
                let lang = SupportedLanguage::Rust;
                let source =
                    fs::read_to_string(path).expect("Unable to read source file for testing");

                let parse_result = parse_source(lang, &source, path, &config);
                assert!(
                    parse_result.is_ok(),
                    "Self-analysis failed for file {:?}",
                    path
                );

                let (modules, primitives) = parse_result.unwrap();
                let (_resolved_modules, graph, _summary) =
                    analyze_project(modules, primitives, &config);
                all_graphs.push(graph);
            }
        }
    }

    assert!(
        !all_graphs.is_empty(),
        "No graphs generated for src directory."
    );

    let json_path = outputs_dir.join("self_analysis.json");
    let json = serde_json::to_string_pretty(&all_graphs)
        .expect("Failed to serialize self-analysis graph.");
    fs::write(&json_path, json).expect("Unable to write self-analysis graph JSON.");

    let cyto_path = outputs_dir.join("cyto_self_analysis.json");
    language_agnostic_analyzer::export::cytoscape::export_graphs(&all_graphs, &cyto_path)
        .expect("Unable to export Cytoscape graph for self-analysis.");
}
