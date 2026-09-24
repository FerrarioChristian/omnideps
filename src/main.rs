use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
use cli::{BenchmarkCommands, Cli, Commands, ConfigCommands};

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Analyze {
            path,
            output,
            csv,
            debug_refs,
            config,
        } => {
            commands::analyze::execute(
                path,
                output.as_deref(),
                csv.as_deref(),
                *debug_refs,
                config.as_deref(),
            )?;
        }
        Commands::Benchmark { cmd } => match cmd {
            BenchmarkCommands::Run {
                testdir,
                output,
                config,
            } => {
                commands::benchmark::execute_run(testdir, output.as_deref(), config.as_deref())?;
            }
            BenchmarkCommands::All { output, config } => {
                commands::benchmark::execute_all(output.as_deref(), config.as_deref())?;
            }
        },
        Commands::ExportCyto { input, output } => {
            commands::cyto_export::execute(input, output)?;
        }
        Commands::Serve { port } => {
            commands::serve::execute(*port)?;
        }
        Commands::Config { cmd } => match cmd {
            ConfigCommands::Init { output } => {
                commands::config::execute_init(output)?;
            }
        },
    }

    Ok(())
}
