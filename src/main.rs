use anyhow::Result;
use clap::Parser;
use language_agnostic_analyzer::{
    analyzer::{self, full_analysis},
    ir::AnalysisSummary,
    debug::print_references,
};
use std::fs;
use walkdir::WalkDir;

mod cli;
use cli::Cli;

/// Entry point of the analyzer.
/// Parses arguments and routes to either single file analysis or recursive directory analysis.
fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.path.is_file() {
        analyze_single_file(&cli.path, cli.output.as_deref(), cli.csv.as_deref(), cli.debug_refs)?;
    } else if cli.path.is_dir() {
        analyze_directory(&cli.path, cli.output.as_deref(), cli.csv.as_deref(), cli.debug_refs)?;
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
) -> Result<()> {
    let lang = detect_language(path)?;
    let source = fs::read_to_string(path)?;
    let (modules, graph, summary) = full_analysis(lang, &source)?;

    println!("=== ANALISI {} ===", path.display());
    print_summary(&summary);

    if debug_refs {
        print_references(&modules);
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

/// Recursively processes all files in a directory.
/// Combines the graphs from all recognized source files and aggregates their summaries.
fn analyze_directory(
    dir: &std::path::Path,
    json_out: Option<&std::path::Path>,
    csv_out: Option<&std::path::Path>,
    debug_refs: bool,
) -> Result<()> {
    let mut total_summary = AnalysisSummary::default();
    let mut all_graphs = vec![];
    let mut all_modules = vec![];

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && let Ok(lang) = detect_language(entry.path())
        {
            let source = fs::read_to_string(entry.path())?;
            let (modules, graph, summary) = analyzer::full_analysis(lang, &source)?;
            total_summary.total_modules += summary.total_modules;
            total_summary.total_structured_types += summary.total_structured_types;
            total_summary.total_free_functions += summary.total_free_functions;
            total_summary.resolved_refs += summary.resolved_refs;
            total_summary.failed_refs += summary.failed_refs;
            all_graphs.push(graph);
            if debug_refs {
                all_modules.extend(modules);
            }
        }
    }

    println!("=== ANALISI CARTELLA {} ===", dir.display());
    print_summary(&total_summary);
    
    if debug_refs {
        print_references(&all_modules);
    }

    // Salva tutto (opzionale)
    if let Some(out) = json_out {
        let json = serde_json::to_string_pretty(&all_graphs)?;
        fs::write(out, json)?;

        // Esportazione automatica Cytoscape
        if let Some(parent) = out.parent() {
            let file_name = out
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("graph.json");
            let cyto_path = parent.join(format!("cyto_{}", file_name));
            language_agnostic_analyzer::export::cytoscape::export_graphs(&all_graphs, &cyto_path)?;
            println!("Grafo Cytoscape salvato in {}", cyto_path.display());
        }
    }
    if let Some(csv) = csv_out {
        save_summary_csv(&total_summary, csv)?;
    }
    Ok(())
}

/// Helper function to match file extensions to tree-sitter language parsers.
fn detect_language(path: &std::path::Path) -> Result<analyzer::languages::SupportedLanguage> {
    use analyzer::languages::SupportedLanguage;
    match path.extension().and_then(|s| s.to_str()) {
        Some("rs") => Ok(SupportedLanguage::Rust),
        Some("java") => Ok(SupportedLanguage::Java),
        Some("py") => Ok(SupportedLanguage::Python),
        Some("c") | Some("h") => Ok(SupportedLanguage::C),
        Some("cpp") | Some("cxx") | Some("cc") | Some("hxx") => Ok(SupportedLanguage::Cpp),
        _ => anyhow::bail!("Language not supported for file: {}", path.display()),
    }
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

