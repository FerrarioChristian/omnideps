use anyhow::Result;
use clap::Parser;
use language_agnostic_analyzer::{
    analyzer::{analyze_project, parse_source},
    debug::print_references,
    language::SupportedLanguage,
    model::AnalysisSummary,
    resolver::primitives::PrimitiveRegistry,
};
use std::fs;
use walkdir::WalkDir;

mod cli;
use cli::Cli;

/// Entry point of the analyzer.
/// Parses arguments and routes to either single file analysis or recursive directory analysis.
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = if let Some(config_path) = cli.config {
        let content = fs::read_to_string(&config_path)?;
        serde_json::from_str(&content)?
    } else {
        language_agnostic_analyzer::config::AnalyzerConfig::default_strategies()
    };

    if cli.path.is_file() {
        analyze_single_file(
            &cli.path,
            cli.output.as_deref(),
            cli.csv.as_deref(),
            cli.debug_refs,
            &config,
        )?;
    } else if cli.path.is_dir() {
        analyze_directory(
            &cli.path,
            cli.output.as_deref(),
            cli.csv.as_deref(),
            cli.debug_refs,
            &config,
        )?;
    } else {
        println!("Percorso non valido!");
    }
    Ok(())
}

/// Runs the full analysis pipeline on a single file, printing the summary
/// and optionally saving the graph to JSON and the summary to CSV.
fn analyze_single_file(
    path: &std::path::Path,
    json_out: Option<&std::path::Path>,
    csv_out: Option<&std::path::Path>,
    debug_refs: bool,
    config: &language_agnostic_analyzer::config::AnalyzerConfig,
) -> anyhow::Result<()> {
    let lang = SupportedLanguage::from_path(path)
        .ok_or_else(|| anyhow::anyhow!("Language not supported for file: {}", path.display()))?;
    let source = fs::read_to_string(path)?;

    let rel_path = path.file_name().map(std::path::Path::new).unwrap_or(path);
    // Phase 1: Parse
    let (modules, primitives) = parse_source(lang, &source, rel_path, config)?;
    // Phase 2-4: Resolve and Graph
    let (resolved_modules, graph, summary) = analyze_project(modules, primitives, config);

    println!("=== ANALISI {} ===", path.display());
    print_summary(&summary);

    if debug_refs {
        print_references(&resolved_modules);
    }

    if let Some(out) = json_out {
        let json = serde_json::to_string_pretty(&graph)?;
        fs::write(out, json)?;
        println!("Grafo salvato in {}", out.display());

        // Esportazione automatica Cytoscape
        if let Some(parent) = out.parent() {
            let file_name = out
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("graph.json");
            let cyto_path = parent.join(format!("cyto_{}", file_name));
            language_agnostic_analyzer::export::cytoscape::export_graphs(
                std::slice::from_ref(&graph),
                &cyto_path,
            )?;
            println!("Grafo Cytoscape salvato in {}", cyto_path.display());
        }
    }

    if let Some(csv) = csv_out {
        save_summary_csv(&summary, csv)?;
    }
    Ok(())
}

/// Recursively processes all files in a directory to build a unified workspace.
/// Resolves cross-file references natively using the algebraic Query Engine.
fn analyze_directory(
    dir: &std::path::Path,
    json_out: Option<&std::path::Path>,
    csv_out: Option<&std::path::Path>,
    debug_refs: bool,
    config: &language_agnostic_analyzer::config::AnalyzerConfig,
) -> anyhow::Result<()> {
    let mut all_modules = vec![];
    let mut combined_primitives = PrimitiveRegistry::empty();

    // Phase 1: Unified Extraction
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && let Some(lang) = SupportedLanguage::from_path(entry.path())
            && let Ok(source) = fs::read_to_string(entry.path())
        {
            let rel_path = entry.path().strip_prefix(dir).unwrap_or(entry.path());
            if let Ok((mut file_modules, file_primitives)) =
                parse_source(lang, &source, rel_path, config)
            {
                all_modules.append(&mut file_modules);
                combined_primitives.merge(file_primitives);
            }
        }
    }

    // Phase 2-4: Unified Resolution and Graph Building
    let (resolved_modules, graph, summary) =
        analyze_project(all_modules, combined_primitives, config);

    println!("=== ANALISI CARTELLA {} ===", dir.display());
    print_summary(&summary);

    if debug_refs {
        print_references(&resolved_modules);
    }

    if let Some(out) = json_out {
        let json = serde_json::to_string_pretty(&graph)?;
        fs::write(out, json)?;
        println!("Grafo dell'intero Workspace salvato in {}", out.display());

        // Esportazione automatica Cytoscape
        if let Some(parent) = out.parent() {
            let file_name = out
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("graph.json");
            let cyto_path = parent.join(format!("cyto_{}", file_name));
            language_agnostic_analyzer::export::cytoscape::export_graphs(
                std::slice::from_ref(&graph),
                &cyto_path,
            )?;
            println!("Grafo Cytoscape salvato in {}", cyto_path.display());
        }
    }
    if let Some(csv) = csv_out {
        save_summary_csv(&summary, csv)?;
    }
    Ok(())
}

/// Prints a basic aggregation of the extracted components to standard output.
fn print_summary(s: &AnalysisSummary) {
    println!("Moduli: {}", s.total_modules);
    println!("Structured types: {}", s.total_structured_types);
    println!("Free functions: {}", s.total_free_functions);
    println!("Riferimenti risolti: {}", s.resolved_refs);
    println!("Riferimenti sconosciuti: {}", s.failed_refs);
}

/// Appends a CSV header and row detailing the total component counts.
fn save_summary_csv(s: &AnalysisSummary, path: &std::path::Path) -> Result<()> {
    let csv = format!(
        "total_modules,total_structured,total_free\n{},{},{}",
        s.total_modules, s.total_structured_types, s.total_free_functions
    );
    fs::write(path, csv)?;
    Ok(())
}
