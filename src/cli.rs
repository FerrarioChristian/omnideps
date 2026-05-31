use clap::Parser;
use std::path::PathBuf;

/// Defines the command-line interface for the `language-agnostic-analyzer`.
///
/// Uses `clap` to parse arguments for input files/directories,
/// and optional paths for outputting JSON dependency graphs and CSV summaries.
#[derive(Parser)]
#[command(
    author,
    version,
    about = "Language-agnostic Architectural Dependency Analyzer"
)]
pub struct Cli {
    /// File o cartella da analizzare
    #[arg(required = true)]
    pub path: PathBuf,

    /// Output JSON (default: stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output CSV summary
    #[arg(short, long)]
    pub csv: Option<PathBuf>,

    /// Print debug info for all resolved and failed references
    #[arg(short = 'd', long)]
    pub debug_refs: bool,
}
