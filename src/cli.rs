use clap::Parser;
use std::path::PathBuf;

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
}
