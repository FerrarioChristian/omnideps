use crate::model::Module;
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
                imports: vec![],
                sub_modules: vec![],
                structured_types: vec![],
                free_functions: vec![],
                impl_blocks: vec![],
            });
        }
        
        match comp {
            crate::heuristics::ParsedItem::Component(crate::model::Component::Module(m)) => {
                // Traverse children to populate the new module before adding it
                let mut cursor = node.walk();
                let mut new_modules = vec![m];
                for child in node.children(&mut cursor) {
                    walk_cst(child, source, &mut new_modules);
                }
                // The new module is now populated (it is located at new_modules[0])
                modules[0].sub_modules.push(new_modules.remove(0));
            }
            crate::heuristics::ParsedItem::Component(crate::model::Component::StructuredType(st)) => modules[0].structured_types.push(st),
            crate::heuristics::ParsedItem::Component(crate::model::Component::Function(ff)) => {
                modules[0].free_functions.push(ff);
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind().contains("body") || child.kind().contains("block") {
                        walk_cst(child, source, modules);
                    }
                }
            }
            crate::heuristics::ParsedItem::ImplBlock(ib) => modules[0].impl_blocks.push(ib),
            crate::heuristics::ParsedItem::Import(i) => modules[0].imports.push(i),
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_cst(child, source, modules);
    }
}

use crate::language::SupportedLanguage;
use crate::resolver::primitives::PrimitiveRegistry;

/// Step 1: Extracts the basic Intermediate Representation (IR) modules from a source file.
pub fn parse_source(
    lang: SupportedLanguage,
    source: &str,
) -> anyhow::Result<(Vec<Module>, PrimitiveRegistry)> {
    let modules = generic_extract(lang.to_tree_sitter_lang(), source)?;
    
    // Load primitives from external registry
    let prim_registry = PrimitiveRegistry::load(lang.name()).unwrap_or_else(|_| {
        PrimitiveRegistry::empty()
    });

    Ok((modules, prim_registry))
}

/// Step 2-4: Resolves references and builds the final Dependency Graph for an entire project.
pub fn analyze_project(
    modules: Vec<Module>,
    prim_registry: PrimitiveRegistry,
) -> (
    Vec<Module>,
    crate::model::DependencyGraph,
    crate::model::AnalysisSummary,
) {
    let resolved = crate::resolver::resolve_type_refs(modules, &prim_registry);
    let graph = crate::export::graph::build_dependency_graph(&resolved);
    let summary = crate::export::summary::build_analysis_summary(&resolved);
    (resolved, graph, summary)
}
