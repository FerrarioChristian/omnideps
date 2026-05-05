use crate::ir::Module;
use tree_sitter::Language;

/// Extracts the basic Intermediate Representation (IR) modules from a source file given its Tree-sitter Language.
/// It parses the source code into an AST and delegates node matching to the heuristics dispatcher.
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

/// Recursively traverses the Concrete Syntax Tree (CST).
/// When a recognized component is found, it's added to the IR and the recursion stops for that branch
/// to prevent duplicating internal methods/functions as top-level components.
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
        
        match comp {
            crate::heuristics::ParsedItem::Component(crate::ir::Component::Module(m)) => {
                // Attraversa i figli popolando il nuovo modulo prima di aggiungerlo
                let mut cursor = node.walk();
                let mut new_modules = vec![m];
                for child in node.children(&mut cursor) {
                    walk_cst(child, source, &mut new_modules);
                }
                // Il nuovo modulo ora è popolato (si trova in new_modules[0])
                modules[0].sub_modules.push(new_modules.remove(0));
            }
            crate::heuristics::ParsedItem::Component(crate::ir::Component::StructuredType(st)) => modules[0].structured_types.push(st),
            crate::heuristics::ParsedItem::Component(crate::ir::Component::Function(ff)) => modules[0].free_functions.push(ff),
            crate::heuristics::ParsedItem::ImplBlock(ib) => modules[0].impl_blocks.push(ib),
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_cst(child, source, modules);
    }
}

/// Wrappers around tree-sitter language loading.
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

/// End-to-end analysis pipeline for a single source file:
/// 1. Extraction into initial IR modules
/// 2. Symbol resolution mapping names to types
/// 3. Dependency graph construction
/// 4. Statistics aggregation (summary)
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
