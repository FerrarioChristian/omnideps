use crate::model::Module;
use tree_sitter::Language;

/// Extracts the basic Intermediate Representation (IR) modules from a source file given its Tree-sitter Language.
/// It parses the source code into an AST and delegates node matching to the heuristics dispatcher.
pub fn generic_extract(lang: Language, source: &str, lang_name: &str, file_path: Option<String>) -> anyhow::Result<Vec<Module>> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang).unwrap();

    let tree = parser
        .parse(source, None)
        .ok_or(anyhow::anyhow!("parse failed"))?;
    let root = tree.root_node();

    let mut modules = vec![];
    walk_cst(root, source, &mut modules, lang_name, file_path);
    Ok(modules)
}

/// Recursively traverses the Concrete Syntax Tree (CST).
/// When a recognized component is found, it's added to the IR and the recursion stops for that branch
/// to prevent duplicating internal methods/functions as top-level components.
fn walk_cst(node: tree_sitter::Node, source: &str, modules: &mut Vec<Module>, lang_name: &str, file_path: Option<String>) {
    if let Some(comp) = crate::heuristics::dispatch_node(node, source) {
        if modules.is_empty() {
            modules.push(Module {
                name: vec!["root".to_string()],
                language: Some(lang_name.to_string()),
                file_path: file_path.clone(),
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
                    walk_cst(child, source, &mut new_modules, lang_name, file_path.clone());
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
                        walk_cst(child, source, modules, lang_name, file_path.clone());
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
        walk_cst(child, source, modules, lang_name, file_path.clone());
    }
}

use crate::language::SupportedLanguage;
use crate::resolver::primitives::PrimitiveRegistry;

/// Step 1: Extracts the basic Intermediate Representation (IR) modules from a source file.
pub fn parse_source(
    lang: SupportedLanguage,
    source: &str,
    path: &std::path::Path,
    config: &crate::config::AnalyzerConfig,
) -> anyhow::Result<(Vec<Module>, PrimitiveRegistry)> {
    let file_path_str = path.to_string_lossy().to_string();
    let mut modules = generic_extract(lang.to_tree_sitter_lang(), source, lang.name(), Some(file_path_str.clone()))?;
    
    // Check if we need to apply DirectoryBased strategy
    let lang_config = config.get_for(lang.name());
    if lang_config.module_strategy == crate::config::ModuleStrategy::DirectoryBased {
        // Strip the common path prefixes if possible or just use the path stems
        let mut path_components: Vec<String> = path
            .components()
            .filter_map(|c| {
                let s = c.as_os_str().to_string_lossy().to_string();
                if s == "." || s == ".." { None } else { Some(s) }
            })
            .collect();
            
        // Remove the extension from the last component
        if let Some(last) = path_components.last_mut() {
            if let Some(stem) = std::path::Path::new(last).file_stem() {
                *last = stem.to_string_lossy().to_string();
            }
        }

        // We wrap the extracted root module inside the directory-based hierarchy
        if !path_components.is_empty() {
            let mut current = modules.remove(0); // This is the "root" module extracted by generic_extract
            
            // Rename the innermost module to the file name
            current.name = vec![path_components.pop().unwrap()];

            // Wrap in outer directories
            for comp in path_components.into_iter().rev() {
                let outer = Module {
                    name: vec![comp],
                    language: Some(lang.name().to_string()),
                    file_path: Some(file_path_str.clone()),
                    imports: vec![],
                    sub_modules: vec![current],
                    structured_types: vec![],
                    free_functions: vec![],
                    impl_blocks: vec![],
                };
                current = outer;
            }
            
            // Put everything back under a virtual "root" to keep compatibility with global tree
            let global_root = Module {
                name: vec!["root".to_string()],
                language: Some(lang.name().to_string()),
                file_path: None,
                imports: vec![],
                sub_modules: vec![current],
                structured_types: vec![],
                free_functions: vec![],
                impl_blocks: vec![],
            };
            modules.push(global_root);
        }
    }

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
    config: &crate::config::AnalyzerConfig,
) -> (
    Vec<Module>,
    crate::model::DependencyGraph,
    crate::model::AnalysisSummary,
) {
    let resolved = crate::resolver::resolve_type_refs(modules, &prim_registry, config);
    let graph = crate::export::graph::build_dependency_graph(&resolved);
    let summary = crate::export::summary::build_analysis_summary(&resolved);
    (resolved, graph, summary)
}
