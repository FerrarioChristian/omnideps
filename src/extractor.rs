use crate::heuristics::dispatch_node;
use crate::ir::Module;
use tree_sitter::{Language, Node, Parser};

pub fn generic_extract(lang: Language, source: &str) -> anyhow::Result<Vec<Module>> {
    let mut parser = Parser::new();
    parser.set_language(&lang).unwrap();

    let tree = parser
        .parse(source, None)
        .ok_or(anyhow::anyhow!("parse failed"))?;
    let root = tree.root_node();

    let mut modules = vec![];
    walk_cst(root, source, &mut modules);
    Ok(modules)
}

fn walk_cst(node: Node, source: &str, modules: &mut Vec<Module>) {
    // Il dispatcher unico viene chiamato su OGNI nodo
    if let Some(component) = crate::heuristics::dispatch_node(node, source) {
        // Qui decidiamo dove mettere il componente (in quale Module)
        // Per ora semplifichiamo: creiamo un modulo root se non esiste
        if modules.is_empty() {
            modules.push(Module {
                name: vec!["root".to_string()],
                sub_modules: vec![],
                structured_types: vec![],
                free_functions: vec![],
                impl_blocks: vec![],
            });
        }
        let root = &mut modules[0];
        match component {
            crate::ir::Component::StructuredType(st) => root.structured_types.push(st),
            crate::ir::Component::FreeFunction(ff) => root.free_functions.push(ff),
            crate::ir::Component::ImplBlock(ib) => root.impl_blocks.push(ib),
            _ => {}
        }
    }

    // Ricorsione su tutti i figli
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_cst(child, source, modules);
    }
}

// Funzioni di comodo per i vari linguaggi
pub mod languages {
    use super::*;
    pub fn c() -> Language {
        tree_sitter_c::LANGUAGE.into()
    }
    pub fn cpp() -> Language {
        tree_sitter_cpp::LANGUAGE.into()
    }
    pub fn java() -> Language {
        tree_sitter_java::LANGUAGE.into()
    }
    pub fn python() -> Language {
        tree_sitter_python::LANGUAGE.into()
    }
    pub fn rust() -> Language {
        tree_sitter_rust::LANGUAGE.into()
    }
}
