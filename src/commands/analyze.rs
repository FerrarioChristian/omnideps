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
/// 2. Optionally calculates analysis summary statistics via [`build_analysis_summary`].
/// 3. Prints reports to stdout.
/// 4. Optionally exports graph results to JSON and Cytoscape formats.
/// 5. Optionally exports analysis summaries to CSV format.
pub fn execute(
    path: &Path,
    output: Option<&Path>,
    raw: bool,
    csv_out: Option<&Path>,
    debug_refs: bool,
    failed_only: bool,
    config_path: Option<&Path>,
    summary: bool,
) -> Result<()> {
    let config = AnalyzerConfig::load_or_default(config_path)?;
    let (resolved_modules, graph) = analyze_project(path, &config)?;
    let analysis_summary = if summary || csv_out.is_some() {
        Some(build_analysis_summary(&resolved_modules))
    } else {
        None
    };

    print_report(
        analysis_summary.as_ref(),
        &resolved_modules,
        path,
        debug_refs,
        failed_only,
    );
    export_results(&graph, analysis_summary.as_ref(), output, raw, csv_out)?;

    Ok(())
}

/// Prints the formatted analysis summary and optional debug references to stdout.
fn print_report(
    summary: Option<&AnalysisSummary>,
    resolved_modules: &[Module],
    path: &Path,
    debug_refs: bool,
    failed_only: bool,
) {
    let target_kind = if path.is_dir() { "CARTELLA " } else { "" };
    println!("=== ANALYSIS {}{} ===", target_kind, path.display());
    if let Some(s) = summary {
        print_summary(s);
    }

    if debug_refs || failed_only {
        print_references(resolved_modules, failed_only);
    }
}

/// Exports the dependency graph and analysis summary to JSON, Cytoscape, and/or CSV files if requested.
fn export_results(
    graph: &DependencyGraph,
    summary: Option<&AnalysisSummary>,
    output: Option<&Path>,
    raw: bool,
    csv_out: Option<&Path>,
) -> Result<()> {
    if let Some(out) = output {
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        omnideps::export::cytoscape::export_graphs(std::slice::from_ref(graph), out)?;
        println!("Cytoscape graph saved to {}", out.display());

        if raw {
            let parent = out.parent().unwrap_or_else(|| Path::new(""));
            let file_name = out
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("graph.json");
            let raw_path = parent.join(format!("raw_{}", file_name));

            if let Some(p) = raw_path.parent() {
                fs::create_dir_all(p)?;
            }
            let file = fs::File::create(&raw_path)?;
            let writer = std::io::BufWriter::new(file);
            serde_json::to_writer(writer, graph)?;
            println!("Raw graph saved to {}", raw_path.display());
        }
    } else if raw {
        let raw_path = Path::new("raw_graph.json");
        let file = fs::File::create(raw_path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer(writer, graph)?;
        println!("Raw graph saved to {}", raw_path.display());
    }

    if let Some(csv) = csv_out
        && let Some(s) = summary
    {
        if let Some(parent) = csv.parent() {
            fs::create_dir_all(parent)?;
        }
        save_summary_csv(s, csv)?;
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
