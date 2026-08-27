use omnideps::analyzer::{analyze_project, parse_source};
use omnideps::language::SupportedLanguage;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[test]
fn test_benchmarks_analysis() {
    let benchmarks_dir = Path::new("tests/generics");
    assert!(
        benchmarks_dir.exists(),
        "La directory benchmarks non esiste!"
    );

    let outputs_dir = Path::new("tests/outputs");
    if !outputs_dir.exists() {
        fs::create_dir_all(outputs_dir)
            .expect("Impossibile creare la directory di output dei test");
    }

    for entry in WalkDir::new(benchmarks_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

            // Definisce il parser Tree-sitter corrispondente in base all'estensione
            let lang = match ext {
                "rs" => SupportedLanguage::Rust,
                "java" => SupportedLanguage::Java,
                "py" => SupportedLanguage::Python,
                "c" | "h" => SupportedLanguage::C,
                "cpp" | "cxx" | "cc" | "hxx" => SupportedLanguage::Cpp,
                _ => continue, // Ignora file JSON e altri non supportati
            };

            let source = fs::read_to_string(path).expect("Impossibile leggere il file");
            let config = omnideps::config::AnalyzerConfig::default();

            // Verifica che l'analisi non vada in panico
            let parse_result = parse_source(lang, &source, path, &config);
            assert!(
                parse_result.is_ok(),
                "Il parsing ha fallito per il file {:?}",
                path
            );

            let (modules, primitives) = parse_result.unwrap();
            let (_resolved_modules, graph, summary) = analyze_project(modules, primitives, &config);

            // Assicuriamoci che il grafo contenga almeno dei nodi di base se il file non è vuoto
            if source.len() > 10 {
                assert!(
                    !graph.nodes.is_empty(),
                    "Il file {:?} ha restituito un grafo vuoto pur non essendo vuoto",
                    path
                );
            }

            assert!(
                summary.total_modules > 0,
                "Dovrebbe esserci almeno il root module per {:?}",
                path
            );

            let filename = path.file_name().unwrap().to_str().unwrap();
            let json_path = outputs_dir.join(format!("{}.json", filename));
            let json =
                serde_json::to_string_pretty(&graph).expect("Impossibile serializzare il grafo");
            fs::write(&json_path, json).expect("Impossibile salvare il JSON di test");

            // Esporta anche la versione Cytoscape per i test
            let cyto_path = outputs_dir.join(format!("cyto_{}.json", filename));
            omnideps::export::cytoscape::export_graphs(
                std::slice::from_ref(&graph),
                &cyto_path,
            )
            .expect("Impossibile esportare il grafo Cytoscape di test");
        }
    }
}
