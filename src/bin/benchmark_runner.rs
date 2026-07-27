use anyhow::{Context, Result};
use clap::Parser;
use language_agnostic_analyzer::{
    analyzer::{analyze_project, parse_source},
    config::AnalyzerConfig,
    language::SupportedLanguage,
    model::{Component, DependencyGraph, TestManifest, TestReport, TestReportEdge, TestReportNode},
    resolver::primitives::PrimitiveRegistry,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about = "Benchmark runner for language-agnostic-analyzer", long_about = None)]
struct Cli {
    /// Directory containing the test benchmark (e.g. tests/benchmark-java)
    #[arg(index = 1)]
    testdir: PathBuf,

    /// Optional config file path
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn analyze_directory(dir: &std::path::Path, config: &AnalyzerConfig) -> Result<DependencyGraph> {
    let mut all_modules = vec![];
    let mut combined_primitives = PrimitiveRegistry::empty();

    let src_dir = dir.join("src");
    if !src_dir.exists() {
        anyhow::bail!("src directory not found in {}", dir.display());
    }

    for entry in WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && let Some(lang) = SupportedLanguage::from_path(entry.path())
            && let Ok(source) = fs::read_to_string(entry.path())
        {
            let rel_path = entry.path().strip_prefix(&src_dir).unwrap_or(entry.path());
            if let Ok((mut file_modules, file_primitives)) =
                parse_source(lang, &source, rel_path, config)
            {
                all_modules.append(&mut file_modules);
                combined_primitives.merge(file_primitives);
            }
        }
    }

    let (_, graph, _) = analyze_project(all_modules, combined_primitives, config);
    Ok(graph)
}

fn flatten_name(name: &[String]) -> String {
    if name.first().map(|s| s.as_str()) == Some("root") {
        name[1..].join(".")
    } else {
        name.join(".")
    }
}

fn verify_graph_adherence(graph: &DependencyGraph, manifest: &TestManifest) -> TestReport {
    // Index components by name
    let mut nodes_map: HashMap<String, &Component> = HashMap::new();
    for node in &graph.nodes {
        let name = match node {
            Component::Module(m) => flatten_name(&m.name),
            Component::StructuredType(s) => flatten_name(&s.name),
            Component::Function(f) => flatten_name(&f.name),
            Component::Field(name, _) => flatten_name(name),
            Component::TypeAlias(t) => flatten_name(&t.name),
            Component::Primitive(p) => p.clone(),
            Component::External(u) => flatten_name(u),
        };
        // Some nodes like roots might be prefixed or not depending on config, but let's try direct matches
        nodes_map.insert(name, node);
    }

    // Index dependencies by source -> sinks
    let mut edges_map: HashMap<String, HashSet<String>> = HashMap::new();
    for edge in &graph.edges {
        let from = flatten_name(&edge.from);
        let to = flatten_name(&edge.to);
        println!("EDGE: {} -> {} ({:?})", from, to, edge.kind);
        edges_map.entry(from).or_default().insert(to);
    }

    let mut report = TestReport::craft(manifest);

    // Verify Nodes
    for node in &manifest.nodes {
        let exists = nodes_map.contains_key(&node.name);

        // TODO: Implement strict kind checking for nodes if needed in the future.
        // For now, as agreed, we ignore strict kind matching between tree-sitter AST kinds and our high-level Component kinds.
        let same_kind = exists;

        let actual_kind = if exists {
            match nodes_map.get(&node.name).unwrap() {
                Component::Module(_) => "Module".to_string(),
                Component::StructuredType(st) => format!("{:?}", st.kind),
                Component::Function(_) => "Function".to_string(),
                Component::Field(_, _) => "Field".to_string(),
                Component::TypeAlias(_) => "TypeAlias".to_string(),
                Component::Primitive(_) => "Primitive".to_string(),
                Component::External(_) => "External".to_string(),
            }
        } else {
            "-".to_string()
        };

        if !exists {
            report.node_not_found_count += 1;
        }

        report
            .nodes
            .push(TestReportNode::craft(node, exists, same_kind, actual_kind));
    }

    // Verify Edges
    for edge in &manifest.edges {
        let source_exists = nodes_map.contains_key(&edge.source);
        let sink_exists = nodes_map.contains_key(&edge.sink);

        let mut edge_exists = false;
        if source_exists
            && let Some(sinks) = edges_map.get(&edge.source)
            && sinks.contains(&edge.sink)
        {
            edge_exists = true;
        }

        if !edge_exists {
            report.edge_not_found_count += 1;
            println!("MISSING EDGE: {} -> {}", edge.source, edge.sink);
            if let Some(sinks) = edges_map.get(&edge.source) {
                println!("  Available sinks for source: {:?}", sinks);
            } else {
                println!("  Source node not in edges_map!");
            }
        }

        // TODO: Implement strict kind checking for edges if needed in the future.
        // Like the predecessor, we don't strictly check edge kinds for now.
        // We just verify that *a* dependency exists between the two nodes.
        let same_kind = edge_exists;

        report.edges.push(TestReportEdge::craft(
            edge,
            source_exists,
            sink_exists,
            edge_exists,
            same_kind,
        ));
    }

    report
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = if let Some(config_path) = cli.config {
        let content = fs::read_to_string(&config_path)?;
        serde_json::from_str(&content)?
    } else {
        AnalyzerConfig::default_strategies()
    };

    let manifest_path = cli.testdir.join("test.yml");
    if !manifest_path.exists() {
        anyhow::bail!("test.yml not found in {}", cli.testdir.display());
    }

    let manifest = TestManifest::load(&manifest_path.to_string_lossy())
        .with_context(|| format!("Failed to load test.yml in {}", cli.testdir.display()))?;

    println!("Analizzando directory test: {}", cli.testdir.display());
    let graph = analyze_directory(&cli.testdir, &config)?;

    let report = verify_graph_adherence(&graph, &manifest);

    let md_path = cli.testdir.join("report.md");
    let json_path = cli.testdir.join("report.json");

    report.save_to_markdown(&md_path.to_string_lossy())?;
    report.save_to_json(&json_path.to_string_lossy())?;

    println!("Report Markdown salvato in {}", md_path.display());
    println!("Report JSON salvato in {}", json_path.display());

    println!(
        "Statistiche nodi: Trovati {}, Non Trovati {}, Totali {}",
        report.nodes.len() - report.node_not_found_count,
        report.node_not_found_count,
        report.nodes.len()
    );
    println!(
        "Statistiche archi: Trovati {}, Non Trovati {}, Totali {}",
        report.edges.len() - report.edge_not_found_count,
        report.edge_not_found_count,
        report.edges.len()
    );

    Ok(())
}
