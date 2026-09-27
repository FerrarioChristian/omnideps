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

        /// Output Cytoscape JSON for graph
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Export raw DependencyGraph format alongside (named raw_<output>)
        #[arg(long)]
        raw: bool,

        /// Output CSV summary
        #[arg(short, long)]
        csv: Option<PathBuf>,

        /// Print debug info for all resolved and failed references
        #[arg(short = 'd', long)]
        debug_refs: bool,

        /// Print debug info ONLY for failed references
        #[arg(short = 'f', long)]
        failed_only: bool,

        /// Calculate and display analysis summary statistics (modules, types, references)
        #[arg(short = 's', long)]
        summary: bool,

        /// Path to a JSON configuration file defining architectural strategies
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Dump full Intermediate Representation (IR) without pruning for debugging
    #[command(name = "dump-ir")]
    DumpIr {
        /// File or directory to analyze
        #[arg(required = true)]
        path: PathBuf,

        /// Output JSON file path (default: ir_dump.json)
        #[arg(short, long, default_value = "ir_dump.json")]
        output: PathBuf,

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
