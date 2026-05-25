use language_agnostic_analyzer::analyzer::{self, full_analysis};
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

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file() {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

            // Analyze only Rust files in the src directory
            if ext == "rs" {
                let lang = analyzer::languages::rust();
                let source = fs::read_to_string(path).expect("Unable to read source file for testing");

                let result = full_analysis(lang, &source);
                assert!(result.is_ok(), "Self-analysis failed for file {:?}", path);

                let (_modules, graph, _summary) = result.unwrap();
                all_graphs.push(graph);
            }
        }
    }

    assert!(
        !all_graphs.is_empty(),
        "No graphs generated for src directory."
    );

    let json_path = outputs_dir.join("self_analysis.json");
    let json = serde_json::to_string_pretty(&all_graphs).expect("Failed to serialize self-analysis graph.");
    fs::write(&json_path, json).expect("Unable to write self-analysis graph JSON.");

    let cyto_path = outputs_dir.join("cyto_self_analysis.json");
    language_agnostic_analyzer::export::cytoscape::export_graphs(&all_graphs, &cyto_path)
        .expect("Unable to export Cytoscape graph for self-analysis.");
}
