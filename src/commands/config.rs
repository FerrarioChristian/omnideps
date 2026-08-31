use anyhow::Result;
use omnideps::config::AnalyzerConfig;
use std::fs;
use std::path::Path;

pub fn execute_init(output: &Path) -> Result<()> {
    let config = AnalyzerConfig::default_strategies();
    let json = serde_json::to_string_pretty(&config)?;
    
    fs::write(output, json)?;
    println!("File di configurazione creato con successo in: {}", output.display());
    Ok(())
}
