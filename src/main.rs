use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
use cli::{BenchmarkCommands, Cli, Commands, ConfigCommands};

const STACK_SIZE: usize = 64 * 1024 * 1024;

fn main() -> Result<()> {
    // Spawn worker thread with a large stack (64MB) to prevent stack overflow on massive ASTs
    // (such as large array tables and deep macro expansions in C/C++ projects like FFmpeg),
    // following the standard architectural pattern used by rustc and other static analysis tools.
    let child = std::thread::Builder::new()
        .name("omnideps-worker".to_string())
        .stack_size(STACK_SIZE)
        .spawn(run)?;

    match child.join() {
        Ok(res) => res,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

fn run() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Analyze {
            path,
            output,
            raw,
            csv,
            debug_refs,
            failed_only,
            summary,
            config,
        } => {
            commands::analyze::execute(
                path,
                output.as_deref(),
                *raw,
                csv.as_deref(),
                *debug_refs,
                *failed_only,
                config.as_deref(),
                *summary,
            )?;
        }
        Commands::DumpIr {
            path,
            output,
            config,
        } => {
            commands::dump_ir::execute(path, output, config.as_deref())?;
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
