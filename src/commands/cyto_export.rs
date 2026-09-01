use anyhow::{Context, Result};
use omnideps::model::DependencyGraph;
use std::fs;
 

pub fn execute(input: &std::path::Path, output: &std::path::Path) -> Result<()> {
    let input_str = fs::read_to_string(input).with_context(|| {
        format!("Impossibile leggere il file di input: {}", input.display())
    })?;

    let graphs: Vec<DependencyGraph> = match serde_json::from_str(&input_str) {
        Ok(list) => list,
        Err(_) => {
            let single: DependencyGraph = serde_json::from_str(&input_str)
                .context("Formato JSON non supportato: deve essere un DependencyGraph o un array di DependencyGraph")?;
            vec![single]
        }
    };

    omnideps::export::cytoscape::export_graphs(&graphs, output)
        .with_context(|| "Errore durante l'esportazione del grafo Cytoscape")?;

    println!("Grafo esportato con successo in {}", output.display());
    Ok(())
}
