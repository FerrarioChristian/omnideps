use crate::ir::Module;
use tree_sitter::Language;

pub fn generic_extract(lang: Language, source: &str) -> anyhow::Result<Vec<Module>> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang).unwrap();

    let tree = parser
        .parse(source, None)
        .ok_or(anyhow::anyhow!("parse failed"))?;
    let root = tree.root_node();

    let mut modules = vec![];
    walk_cst(root, source, &mut modules);
    Ok(modules)
}

fn walk_cst(node: tree_sitter::Node, source: &str, modules: &mut Vec<Module>) {
    if let Some(comp) = crate::heuristics::dispatch_node(node, source) {
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
        match comp {
            crate::ir::Component::StructuredType(st) => root.structured_types.push(st),
            crate::ir::Component::FreeFunction(ff) => root.free_functions.push(ff),
            crate::ir::Component::ImplBlock(ib) => root.impl_blocks.push(ib),
            _ => {}
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_cst(child, source, modules);
    }
}

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

pub fn full_analysis(
    lang: Language,
    source: &str,
) -> anyhow::Result<(
    Vec<Module>,
    crate::ir::DependencyGraph,
    crate::ir::AnalysisSummary,
)> {
    let modules = generic_extract(lang, source)?;
    let resolved = crate::analysis::resolve_type_refs(modules);
    let graph = crate::analysis::build_dependency_graph(&resolved);
    let summary = crate::analysis::build_analysis_summary(&resolved);
    Ok((resolved, graph, summary))
}
