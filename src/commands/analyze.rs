use anyhow::Result;
use omnideps::{
    analyzer::{analyze_project, parse_source},
    debug::print_references,
    language::SupportedLanguage,
    model::AnalysisSummary,
    resolver::primitives::PrimitiveRegistry,
};
use std::fs;
use walkdir::WalkDir;

pub fn execute(
    path: &std::path::Path,
    json_out: Option<&std::path::Path>,
    csv_out: Option<&std::path::Path>,
    debug_refs: bool,
    config: &omnideps::config::AnalyzerConfig,
) -> Result<()> {
    if path.is_file() {
        analyze_single_file(path, json_out, csv_out, debug_refs, config)
    } else if path.is_dir() {
        analyze_directory(path, json_out, csv_out, debug_refs, config)
    } else {
        println!("Invalid path!");
        Ok(())
    }
}

fn analyze_single_file(
    path: &std::path::Path,
    json_out: Option<&std::path::Path>,
    csv_out: Option<&std::path::Path>,
    debug_refs: bool,
    config: &omnideps::config::AnalyzerConfig,
) -> Result<()> {
    let lang = SupportedLanguage::from_path(path)
        .ok_or_else(|| anyhow::anyhow!("Language not supported for file: {}", path.display()))?;
    let source = fs::read_to_string(path)?;

    let rel_path = path.file_name().map(std::path::Path::new).unwrap_or(path);
    let (modules, primitives) = parse_source(lang, &source, rel_path, config)?;
    let (resolved_modules, graph, summary) = analyze_project(modules, primitives, config);

    println!("=== ANALYSIS {} ===", path.display());
    print_summary(&summary);

    if debug_refs {
        print_references(&resolved_modules);
    }

    if let Some(out) = json_out {
        let json = serde_json::to_string_pretty(&graph)?;
        fs::write(out, json)?;
        println!("Graph saved to {}", out.display());

        if let Some(parent) = out.parent() {
            let file_name = out
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("graph.json");
            let cyto_path = parent.join(format!("cyto_{}", file_name));
            omnideps::export::cytoscape::export_graphs(
                std::slice::from_ref(&graph),
                &cyto_path,
            )?;
            println!("Cytoscape graph saved to {}", cyto_path.display());
        }
    }

    if let Some(csv) = csv_out {
        save_summary_csv(&summary, csv)?;
    }
    Ok(())
}

fn analyze_directory(
    dir: &std::path::Path,
    json_out: Option<&std::path::Path>,
    csv_out: Option<&std::path::Path>,
    debug_refs: bool,
    config: &omnideps::config::AnalyzerConfig,
) -> Result<()> {
    let mut all_modules = vec![];
    let mut combined_primitives = PrimitiveRegistry::empty();

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

    let (resolved_modules, graph, summary) =
        analyze_project(all_modules, combined_primitives, config);

    println!("=== ANALYSIS CARTELLA {} ===", dir.display());
    print_summary(&summary);

    if debug_refs {
        print_references(&resolved_modules);
    }

    if let Some(out) = json_out {
        let json = serde_json::to_string_pretty(&graph)?;
        fs::write(out, json)?;
        println!("Workspace graph saved to {}", out.display());

        if let Some(parent) = out.parent() {
            let file_name = out
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("graph.json");
            let cyto_path = parent.join(format!("cyto_{}", file_name));
            omnideps::export::cytoscape::export_graphs(
                std::slice::from_ref(&graph),
                &cyto_path,
            )?;
            println!("Cytoscape graph saved to {}", cyto_path.display());
        }
    }
    if let Some(csv) = csv_out {
        save_summary_csv(&summary, csv)?;
    }
    Ok(())
}

fn print_summary(s: &AnalysisSummary) {
    println!("Modules: {}", s.total_modules);
    println!("Structured types: {}", s.total_structured_types);
    println!("Free functions: {}", s.total_free_functions);
    println!("Resolved references: {}", s.resolved_refs);
    println!("Unknown references: {}", s.failed_refs);
}

fn save_summary_csv(s: &AnalysisSummary, path: &std::path::Path) -> Result<()> {
    let csv = format!(
        "total_modules,total_structured,total_free\n{},{},{}",
        s.total_modules, s.total_structured_types, s.total_free_functions
    );
    fs::write(path, csv)?;
    Ok(())
}
