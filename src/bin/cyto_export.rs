use anyhow::{Context, Result};
use clap::Parser;
use language_agnostic_analyzer::model::DependencyGraph;
use std::fs;
use std::path::PathBuf;

/// Converte un grafo IR (singolo o lista) in un grafo compatibile con Cytoscape.
#[derive(Parser)]
#[command(author, version, about = "Cytoscape Graph Exporter")]
struct Cli {
    /// File JSON di input (IR Graph)
    #[arg(required = true)]
    input: PathBuf,

    /// File JSON di output (Cytoscape Graph)
    #[arg(required = true)]
    output: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let input_str = fs::read_to_string(&cli.input)
        .with_context(|| format!("Impossibile leggere il file di input: {}", cli.input.display()))?;

    // Prova a deserializzare come Vec<DependencyGraph>, se fallisce prova come singolo DependencyGraph
    let graphs: Vec<DependencyGraph> = match serde_json::from_str(&input_str) {
        Ok(list) => list,
        Err(_) => {
            let single: DependencyGraph = serde_json::from_str(&input_str)
                .context("Formato JSON non supportato: deve essere un DependencyGraph o un array di DependencyGraph")?;
            vec![single]
        }
    };

    language_agnostic_analyzer::export::cytoscape::export_graphs(&graphs, &cli.output)
        .with_context(|| "Errore durante l'esportazione del grafo Cytoscape")?;

    println!("Grafo esportato con successo in {}", cli.output.display());

    Ok(())
}
