use crate::model::{Component, DependencyGraph, QualifiedName};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct CytoscapeElement {
    data: CytoscapeData,
}

#[derive(Serialize)]
#[serde(untagged)]
enum CytoscapeData {
    Node {
        id: String,
        label: String,
        #[serde(rename = "type")]
        ty: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        parent: Option<String>,
    },
    Edge {
        id: String,
        source: String,
        target: String,
        label: String,
    },
}

fn qn_to_id(qn: &QualifiedName) -> String {
    qn.join("::")
}

fn get_parent_id(qn: &QualifiedName) -> Option<String> {
    if qn.len() > 1 {
        let parent_qn = qn[..qn.len() - 1].to_vec();
        if parent_qn == vec!["root".to_string()] {
            None
        } else {
            Some(qn_to_id(&parent_qn))
        }
    } else {
        None
    }
}

pub fn export_graphs(graphs: &[DependencyGraph], out_path: &Path) -> anyhow::Result<()> {
    let mut elements = Vec::new();
    let mut added_nodes = HashSet::new();
    let mut added_edges = HashSet::new();
    let mut global_edge_id = 0;

    // Helper to add a node
    let mut add_node = |elements: &mut Vec<CytoscapeElement>,
                        added_nodes: &mut HashSet<String>,
                        id: String,
                        label: String,
                        ty: String,
                        parent: Option<String>| {
        if !added_nodes.contains(&id) {
            added_nodes.insert(id.clone());
            elements.push(CytoscapeElement {
                data: CytoscapeData::Node { id, label, ty, parent },
            });
        }
    };

    for graph in graphs {
        // 1. Export nodes
        for node in &graph.nodes {
            match node {
                Component::Module(m) => {
                    let id = qn_to_id(&m.name);
                    if id == "root" {
                        continue;
                    }
                    let label = m.name.last().cloned().unwrap_or_else(|| "root".to_string());
                    let parent = get_parent_id(&m.name);
                    add_node(&mut elements, &mut added_nodes, id, label, "Module".to_string(), parent);
                }
                Component::StructuredType(st) => {
                    let id = qn_to_id(&st.name);
                    let label = st.name.last().cloned().unwrap_or_else(|| "Unknown".to_string());
                    let parent = get_parent_id(&st.name);
                    add_node(&mut elements, &mut added_nodes, id, label, format!("{:?}", st.kind), parent);
                }
                Component::Function(f) => {
                    let id = qn_to_id(&f.name);
                    let name = f.name.last().cloned().unwrap_or_else(|| "".to_string());
                    let label = format!("{}()", name);
                    let parent = get_parent_id(&f.name);
                    add_node(&mut elements, &mut added_nodes, id, label, "Function".to_string(), parent);
                }
            }
        }

        // 2. Export edges
        for edge in &graph.edges {
            let source_id = qn_to_id(&edge.from);
            let target_id = qn_to_id(&edge.to);
            let label = format!("{:?}", edge.kind);

            // Skip structural edges that are now implicitly represented by Compound Nodes
            if label == "ModuleContainment" || label == "NestedIn" {
                continue;
            }

            if source_id.is_empty() || target_id.is_empty() {
                continue;
            }

            // Ensure source node exists
            if !added_nodes.contains(&source_id) {
                let node_label = source_id.split("::").last().unwrap_or("").to_string();
                add_node(&mut elements, &mut added_nodes, source_id.clone(), node_label, "External".to_string(), None);
            }
            // Ensure target node exists
            if !added_nodes.contains(&target_id) {
                let node_label = target_id.split("::").last().unwrap_or("").to_string();
                add_node(&mut elements, &mut added_nodes, target_id.clone(), node_label, "External".to_string(), None);
            }

            let edge_sig = format!("{}->{}:{}", source_id, target_id, label);
            if !added_edges.contains(&edge_sig) {
                added_edges.insert(edge_sig);
                elements.push(CytoscapeElement {
                    data: CytoscapeData::Edge {
                        id: format!("e{}", global_edge_id),
                        source: source_id,
                        target: target_id,
                        label,
                    },
                });
                global_edge_id += 1;
            }
        }
    }

    let json = serde_json::to_string_pretty(&elements)?;
    fs::write(out_path, json)?;
    Ok(())
}
