use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "omnideps",
    author,
    version,
    about = "Omnideps: Language-agnostic Architectural Dependency Analyzer"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze a file or directory
    Analyze {
        /// File or directory to analyze
        #[arg(required = true)]
        path: PathBuf,

        /// Output JSON for graph
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output CSV summary
        #[arg(short, long)]
        csv: Option<PathBuf>,

        /// Print debug info for all resolved and failed references
        #[arg(short = 'd', long)]
        debug_refs: bool,

        /// Path to a JSON configuration file defining architectural strategies
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Benchmark suite operations
    Benchmark {
        #[command(subcommand)]
        cmd: BenchmarkCommands,
    },
    /// Convert JSON graph to Cytoscape format
    ExportCyto {
        /// Input JSON file
        #[arg(required = true)]
        input: PathBuf,

        /// Output Cytoscape JSON file
        #[arg(required = true)]
        output: PathBuf,
    },
    /// Serve the interactive Web Visualizer
    Serve {
        /// Port to listen on (default: 3000)
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
    /// Manage configuration files
    Config {
        #[command(subcommand)]
        cmd: ConfigCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Initialize a default configuration file
    Init {
        /// Output configuration file path (default: omnideps.json)
        #[arg(default_value = "omnideps.json")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum BenchmarkCommands {
    /// Run benchmark on a single language directory
    Run {
        /// Directory containing the test benchmark (e.g. tests/benchmark-java)
        #[arg(index = 1)]
        testdir: PathBuf,

        /// Output directory for the report (defaults to testdir)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Optional config file path
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Run all benchmarks
    All {
        /// Output directory for all reports and results.csv (defaults to tests/benchmarks)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Optional config file path
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
}
