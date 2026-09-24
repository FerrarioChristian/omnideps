use anyhow::Result;
use omnideps::{
    analyzer::analyze_project,
    config::AnalyzerConfig,
    debug::print_references,
    export::build_analysis_summary,
    model::{AnalysisSummary, DependencyGraph, Module},
};
use std::fs;
use std::path::Path;

/// Executes the CLI `analyze` command on a specified file or directory path.
///
/// Workflow:
/// 1. Executes the complete analysis pipeline via [`analyze_project`].
/// 2. Calculates analysis summary statistics via [`build_analysis_summary`].
/// 3. Prints reports to stdout.
/// 4. Optionally exports graph results to JSON and Cytoscape formats.
/// 5. Optionally exports analysis summaries to CSV format.
pub fn execute(
    path: &Path,
    json_out: Option<&Path>,
    csv_out: Option<&Path>,
    debug_refs: bool,
    config_path: Option<&Path>,
) -> Result<()> {
    let config = AnalyzerConfig::load_or_default(config_path)?;
    let (resolved_modules, graph) = analyze_project(path, &config)?;
    let summary = build_analysis_summary(&resolved_modules);

    print_report(&summary, &resolved_modules, path, debug_refs);
    export_results(&graph, &summary, json_out, csv_out)?;

    Ok(())
}

/// Prints the formatted analysis summary and optional debug references to stdout.
fn print_report(
    summary: &AnalysisSummary,
    resolved_modules: &[Module],
    path: &Path,
    debug_refs: bool,
) {
    let target_kind = if path.is_dir() { "CARTELLA " } else { "" };
    println!("=== ANALYSIS {}{} ===", target_kind, path.display());
    print_summary(summary);

    if debug_refs {
        print_references(resolved_modules);
    }
}

/// Exports the dependency graph and analysis summary to JSON, Cytoscape, and/or CSV files if requested.
fn export_results(
    graph: &DependencyGraph,
    summary: &AnalysisSummary,
    json_out: Option<&Path>,
    csv_out: Option<&Path>,
) -> Result<()> {
    if let Some(out) = json_out {
        let json = serde_json::to_string_pretty(graph)?;
        fs::write(out, json)?;
        println!("Graph saved to {}", out.display());

        if let Some(parent) = out.parent() {
            let file_name = out
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("graph.json");
            let cyto_path = parent.join(format!("cyto_{}", file_name));
            omnideps::export::cytoscape::export_graphs(std::slice::from_ref(graph), &cyto_path)?;
            println!("Cytoscape graph saved to {}", cyto_path.display());
        }
    }

    if let Some(csv) = csv_out {
        save_summary_csv(summary, csv)?;
    }

    Ok(())
}

/// Formats and prints high-level summary metrics to standard output.
fn print_summary(s: &AnalysisSummary) {
    println!("Modules: {}", s.total_modules);
    println!("Structured types: {}", s.total_structured_types);
    println!("Free functions: {}", s.total_free_functions);
    println!("Resolved references: {}", s.resolved_refs);
    println!("Unknown references: {}", s.failed_refs);
}

/// Serializes and saves high-level analysis metrics into a comma-separated values (CSV) file.
fn save_summary_csv(s: &AnalysisSummary, path: &Path) -> Result<()> {
    let csv = format!(
        "total_modules,total_structured,total_free\n{},{},{}",
        s.total_modules, s.total_structured_types, s.total_free_functions
    );
    fs::write(path, csv)?;
    Ok(())
}
