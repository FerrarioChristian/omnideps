use anyhow::{Context, Result};
use omnideps::{
    analyzer::{analyze_project, parse_source},
    config::AnalyzerConfig,
    language::SupportedLanguage,
    model::{Component, DependencyGraph, TestManifest, TestReport, TestReportEdge, TestReportNode},
    resolver::primitives::PrimitiveRegistry,
};
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

pub fn execute_run(testdir: &Path, output: Option<&Path>, config: &AnalyzerConfig) -> Result<()> {
    let manifest_path = testdir.join("test.yml");
    if !manifest_path.exists() {
        anyhow::bail!("test.yml not found in {}", testdir.display());
    }

    let manifest = TestManifest::load(&manifest_path.to_string_lossy())
        .with_context(|| format!("Failed to load test.yml in {}", testdir.display()))?;

    println!("Analyzing test directory: {}", testdir.display());
    let graph = analyze_directory(testdir, config)?;

    let report = verify_graph_adherence(&graph, &manifest);

    let out_dir = output.unwrap_or(testdir);
    if !out_dir.exists() {
        fs::create_dir_all(out_dir)?;
    }

    let md_path = out_dir.join("report.md");
    let json_path = out_dir.join("report.json");

    report.save_to_markdown(&md_path.to_string_lossy())?;
    report.save_to_json(&json_path.to_string_lossy())?;

    println!("Markdown report saved to {}", md_path.display());
    println!("JSON report saved to {}", json_path.display());

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

pub fn execute_all(output: Option<&Path>, config: &AnalyzerConfig) -> Result<()> {
    let benchmarks_dir = Path::new("tests/benchmarks");

    if !benchmarks_dir.exists() {
        anyhow::bail!("Directory tests/benchmarks does not exist.");
    }

    let out_dir = output.unwrap_or(benchmarks_dir);
    if !out_dir.exists() {
        fs::create_dir_all(out_dir)?;
    }

    let mut results = std::collections::BTreeMap::new();

    for entry in fs::read_dir(benchmarks_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy();
            if dir_name.starts_with("benchmark-") {
                let lang = dir_name.strip_prefix("benchmark-").unwrap().to_string();
                println!("Running benchmark for {}...", lang);

                let sub_out_dir = output.map(|p| p.join(dir_name.as_ref()));
                if let Err(e) = execute_run(&path, sub_out_dir.as_deref(), config) {
                    log::warn!(" benchmark run failed on {:?}: {}", path, e);
                }

                let report_path = sub_out_dir.unwrap_or_else(|| path.join("report.json"));
                let report_path = if report_path.is_dir() { report_path.join("report.json") } else { report_path };
                if report_path.exists() {
                    let content = fs::read_to_string(&report_path)?;
                    let report: omnideps::model::TestReport = serde_json::from_str(&content)?;

                    let total_nodes = report.nodes.len();
                    let nodes_found = total_nodes.saturating_sub(report.node_not_found_count);

                    let total_edges = report.edges.len();
                    let edges_found = total_edges.saturating_sub(report.edge_not_found_count);

                    results.insert(lang, (nodes_found, total_nodes, edges_found, total_edges));
                } else {
                    log::warn!(" report.json not found in {:?}", path);
                }
            }
        }
    }

    let csv_path = out_dir.join("results.csv");
    let file_exists = csv_path.exists();

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&csv_path)?;

    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);

    let mut headers = vec!["Timestamp".to_string()];
    let mut row = vec![];

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    row.push(timestamp);

    for (lang, (nf, tn, ef, te)) in &results {
        headers.push(format!("{} Nodes", lang));
        headers.push(format!("{} Edges", lang));

        row.push(format!("{}/{}", nf, tn));
        row.push(format!("{}/{}", ef, te));
    }

    if !file_exists {
        wtr.write_record(&headers)?;
    }
    wtr.write_record(&row)?;
    wtr.flush()?;

    println!("CSV report appended to {}", csv_path.display());

    Ok(())
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
        nodes_map.insert(name, node);
    }

    let mut edges_map: HashMap<String, HashSet<String>> = HashMap::new();
    for edge in &graph.edges {
        let from = flatten_name(&edge.from);
        let to = flatten_name(&edge.to);
        edges_map.entry(from).or_default().insert(to);
    }

    let mut report = TestReport::craft(manifest);

    for node in &manifest.nodes {
        let exists = nodes_map.contains_key(&node.name);
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
        }

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
