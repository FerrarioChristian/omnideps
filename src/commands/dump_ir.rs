use anyhow::Result;
use omnideps::{analyzer::analyze_project, config::AnalyzerConfig};
use std::fs;
use std::path::Path;

/// Executes the CLI `dump-ir` command, serializing the full, uncut Intermediate Representation.
pub fn execute(path: &Path, output: &Path, config_path: Option<&Path>) -> Result<()> {
    let config = AnalyzerConfig::load_or_default(config_path)?;
    let (resolved_modules, _graph) = analyze_project(path, &config)?;

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = fs::File::create(output)?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &resolved_modules)?;
    println!("Full IR dumped to {}", output.display());

    Ok(())
}
