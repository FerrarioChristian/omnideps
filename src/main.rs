use anyhow::Result;
use clap::Parser;
use std::fs;

mod cli;
mod commands;
use cli::{Cli, Commands, BenchmarkCommands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Analyze { path, output, csv, debug_refs, config } => {
            let config_val = if let Some(config_path) = config {
                let content = fs::read_to_string(config_path)?;
                serde_json::from_str(&content)?
            } else {
                omnideps::config::AnalyzerConfig::default_strategies()
            };
            commands::analyze::execute(path, output.as_deref(), csv.as_deref(), *debug_refs, &config_val)?;
        }
        Commands::Benchmark { cmd } => match cmd {
            BenchmarkCommands::Run { testdir, output, config } => {
                let config_val = if let Some(config_path) = config {
                    let content = fs::read_to_string(config_path)?;
                    serde_json::from_str(&content)?
                } else {
                    omnideps::config::AnalyzerConfig::default_strategies()
                };
                commands::benchmark::execute_run(testdir, output.as_deref(), &config_val)?;
            }
            BenchmarkCommands::All => {
                let config_val = omnideps::config::AnalyzerConfig::default_strategies();
                commands::benchmark::execute_all(&config_val)?;
            }
        },
        Commands::ExportCyto { input, output } => {
            commands::cyto_export::execute(input, output)?;
        }
        Commands::Serve { port } => {
            commands::serve::execute(*port)?;
        }
    }

    Ok(())
}
