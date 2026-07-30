use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
struct ReportJson {
    nodes: Vec<serde_json::Value>,
    edges: Vec<serde_json::Value>,
    node_not_found_count: usize,
    edge_not_found_count: usize,
}

fn main() -> Result<()> {
    let benchmarks_dir = Path::new("tests/benchmarks");

    if !benchmarks_dir.exists() {
        anyhow::bail!("Directory tests/benchmarks does not exist.");
    }

    let mut results = BTreeMap::new();

    for entry in fs::read_dir(benchmarks_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy();
            if dir_name.starts_with("benchmark-") {
                let lang = dir_name.strip_prefix("benchmark-").unwrap().to_string();
                println!("Running benchmark for {}...", lang);

                let status = Command::new("cargo")
                    .arg("run")
                    .arg("--bin")
                    .arg("benchmark_runner")
                    .arg(&path)
                    .status()
                    .context(format!("Failed to run benchmark_runner on {:?}", path))?;

                if !status.success() {
                    eprintln!("Warning: benchmark_runner failed on {:?}", path);
                }

                let report_path = path.join("report.json");
                if report_path.exists() {
                    let content = fs::read_to_string(&report_path)?;
                    let report: ReportJson = serde_json::from_str(&content)?;

                    let total_nodes = report.nodes.len();
                    let nodes_found = total_nodes.saturating_sub(report.node_not_found_count);

                    let total_edges = report.edges.len();
                    let edges_found = total_edges.saturating_sub(report.edge_not_found_count);

                    results.insert(lang, (nodes_found, total_nodes, edges_found, total_edges));
                } else {
                    eprintln!("Warning: report.json not found in {:?}", path);
                }
            }
        }
    }

    let csv_path = benchmarks_dir.join("results.csv");
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

    // Get current time as timestamp string
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
