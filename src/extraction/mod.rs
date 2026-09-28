pub mod strategies;

use anyhow::{Result, anyhow, bail};
use std::fs;
use std::path::Path;
use tree_sitter::{Language, Node, Parser};
use walkdir::WalkDir;

use crate::config::AnalyzerConfig;
use crate::heuristics::parsers::try_parse_package_declaration;
use crate::heuristics::{ParsedItem, dispatch_node};
use crate::language::SupportedLanguage;
use crate::model::{Component, Field, Module, TypeRef};
use crate::resolver::primitives::PrimitiveRegistry;
use strategies::{apply_directory_strategy, apply_package_strategy, link_out_of_line_methods};

/// Phase 1 Entry Point: Recursively traverses a file or directory path, extracting all IR modules ($\varepsilon : \mathcal{W} \to \mathcal{D}$).
///
/// Universal workspace ingestion function for OmniDeps:
/// - If `path` is a single file, it parses that file directly.
/// - If `path` is a directory, it traverses the directory tree using [`WalkDir`], extracting modules from all supported files.
/// - Post-extraction, it applies universal out-of-line method linking across all extracted modules.
///
/// # Arguments
/// * `path` - A path to a single source file or root workspace directory.
/// * `config` - The global [`AnalyzerConfig`].
///
/// # Returns
/// A tuple containing:
/// * `Vec<Module>`: Complete unresolved IR module forest $\mathcal{D}$.
/// * `PrimitiveRegistry`: Merged registry of primitive types recognized across all parsed files.
///
/// # Errors
/// Returns an error if:
/// * The path does not exist.
/// * The path points to an unsupported single file.
/// * No supported source files are found or successfully parsed.
pub fn extract_ir(
    path: &Path,
    config: &AnalyzerConfig,
) -> Result<(Vec<Module>, PrimitiveRegistry)> {
    if !path.exists() {
        bail!("Path not found: {}", path.display());
    }

    if path.is_file() && SupportedLanguage::from_path(path).is_none() {
        bail!("Language not supported for file: {}", path.display());
    }

    let root_dir = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(Path::new(""))
    };

    let mut all_modules = vec![];
    let mut combined_primitives = PrimitiveRegistry::empty();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && let Some(lang) = SupportedLanguage::from_path(entry.path())
            && let Ok(source) = fs::read_to_string(entry.path())
        {
            let rel_path = entry.path().strip_prefix(root_dir).unwrap_or(entry.path());
            if let Ok((mut file_modules, file_primitives)) =
                parse_source(lang, &source, rel_path, config)
            {
                all_modules.append(&mut file_modules);
                combined_primitives.merge(file_primitives);
            }
        }
    }

    if all_modules.is_empty() {
        bail!("No supported source files found in: {}", path.display());
    }

    // Universal post-extraction pass: link out-of-line method definitions across workspace modules
    link_out_of_line_methods(&mut all_modules);

    Ok((all_modules, combined_primitives))
}

/// Extracts Intermediate Representation (IR) modules from a single source file or code snippet.
///
/// Parses the source code into a Tree-sitter CST, applies the language-specific
/// module grouping strategy configured in $\mathcal{K}$ (directory-based, package-declaration-based,
/// or global root fallback), links out-of-line methods, and loads primitive type definitions.
///
/// # Arguments
/// * `lang` - Detected [`SupportedLanguage`].
/// * `source` - Source code content as string slice.
/// * `path` - Relative filesystem path used for hierarchical module grouping.
/// * `config` - Global [`AnalyzerConfig`].
///
/// # Returns
/// A tuple of:
/// * `Vec<Module>`: The extracted module tree (rooted in a top-level module).
/// * `PrimitiveRegistry`: Language-specific primitive types loaded from configuration.
pub fn parse_source(
    lang: SupportedLanguage,
    source: &str,
    path: &Path,
    config: &AnalyzerConfig,
) -> Result<(Vec<Module>, PrimitiveRegistry)> {
    let file_path_str = path.to_string_lossy().to_string();
    let (mut modules, package_path) = extract_from_cst(
        lang.to_tree_sitter_lang(),
        source,
        lang.name(),
        Some(file_path_str.clone()),
        config,
    )?;

    let lang_config = config.get_for(lang.name());

    if lang_config.modules.package_decl_based {
        // Distribute file-scoped imports to the top-level structured types (classes/interfaces)
        if !modules.is_empty() {
            let imports = modules[0].imports.clone();
            for st in &mut modules[0].structured_types {
                st.imports = imports.clone();
            }
            // Clear module imports so they aren't incorrectly merged at the package level
            modules[0].imports.clear();
        }
    }

    if lang_config.modules.file_based && lang_config.modules.directory_based {
        apply_directory_strategy(&mut modules, path, &file_path_str, lang.name());
    } else if lang_config.modules.package_decl_based {
        if let Some(pkg_path) = package_path {
            apply_package_strategy(&mut modules, pkg_path, &file_path_str, lang.name());
        }
    } else {
        // Fallback strategy for C/C++ or simple languages without explicit path-to-module rules
        let mut global_root = Module {
            name: vec!["root".to_string()],
            language: Some(lang.name().to_string()),
            file_path: None,
            imports: vec![],
            sub_modules: vec![],
            structured_types: vec![],
            type_aliases: vec![],
            free_functions: vec![],
            impl_blocks: vec![],
            free_variables: vec![],
        };
        if !modules.is_empty() {
            let file_mod = modules.remove(0);
            global_root.sub_modules.extend(file_mod.sub_modules);
            global_root
                .structured_types
                .extend(file_mod.structured_types);
            global_root.free_functions.extend(file_mod.free_functions);
            global_root.free_variables.extend(file_mod.free_variables);
            global_root.impl_blocks.extend(file_mod.impl_blocks);
            global_root.imports.extend(file_mod.imports);
            global_root.type_aliases.extend(file_mod.type_aliases);
        }
        modules.push(global_root);
    }

    // Link out-of-line method definitions within this file's extracted module
    link_out_of_line_methods(&mut modules);

    // Load primitives from external registry
    let prim_registry =
        PrimitiveRegistry::load(lang.name()).unwrap_or_else(|_| PrimitiveRegistry::empty());

    Ok((modules, prim_registry))
}

/// Parses source code using Tree-sitter and extracts IR components via the heuristics dispatcher.
///
/// # Arguments
/// * `lang` - Tree-sitter [`Language`] grammar.
/// * `source` - Source code content.
/// * `lang_name` - Canonical language name identifier.
/// * `file_path` - Optional file path for associating components with source files.
/// * `config` - Global analyzer configuration.
///
/// # Returns
/// A tuple containing the extracted [`Module`] list and an optional package declaration path.
pub fn extract_from_cst(
    lang: Language,
    source: &str,
    lang_name: &str,
    file_path: Option<String>,
    config: &AnalyzerConfig,
) -> Result<(Vec<Module>, Option<Vec<String>>)> {
    let mut parser = Parser::new();
    parser.set_language(&lang).unwrap();

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow!("parse failed"))?;
    let root = tree.root_node();

    let mut package_path = None;
    if config.get_for(lang_name).modules.package_decl_based {
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if let Some(pkg) = try_parse_package_declaration(child, source) {
                package_path = Some(pkg);
                break;
            }
        }
    }

    let mut modules = vec![];
    let mut pending_attributes = vec![];
    walk_cst(
        root,
        source,
        &mut modules,
        &mut pending_attributes,
        lang_name,
        file_path,
        config,
    );
    Ok((modules, package_path))
}

/// Recursively traverses the Concrete Syntax Tree (CST).
/// When a recognized component is found, it's added to the IR and the recursion stops for that branch
/// to prevent duplicating internal methods/functions as top-level components.
fn walk_cst(
    node: Node,
    source: &str,
    modules: &mut Vec<Module>,
    pending_attributes: &mut Vec<TypeRef>,
    lang_name: &str,
    file_path: Option<String>,
    config: &AnalyzerConfig,
) {
    if let Some(comp) = dispatch_node(node, source, lang_name, config) {
        if modules.is_empty() {
            modules.push(Module {
                name: vec!["root".to_string()],
                language: Some(lang_name.to_string()),
                file_path: file_path.clone(),
                imports: vec![],
                sub_modules: vec![],
                structured_types: vec![],
                type_aliases: vec![],
                free_functions: vec![],
                impl_blocks: vec![],
                free_variables: vec![],
            });
        }

        match comp {
            ParsedItem::Component(Component::Module(m)) => {
                let mut cursor = node.walk();
                let mut new_modules = vec![m];
                for child in node.children(&mut cursor) {
                    walk_cst(
                        child,
                        source,
                        &mut new_modules,
                        pending_attributes,
                        lang_name,
                        file_path.clone(),
                        config,
                    );
                }
                modules[0].sub_modules.push(new_modules.remove(0));
            }
            ParsedItem::Component(Component::StructuredType(mut st)) => {
                st.annotations.append(pending_attributes);
                modules[0].structured_types.push(st);
            }
            ParsedItem::Component(Component::Function(mut ff)) => {
                ff.annotations.append(pending_attributes);
                modules[0].free_functions.push(ff);
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind().contains("body") || child.kind().contains("block") {
                        walk_cst(
                            child,
                            source,
                            modules,
                            pending_attributes,
                            lang_name,
                            file_path.clone(),
                            config,
                        );
                    }
                }
            }
            ParsedItem::Component(Component::Field(name, ty)) => {
                if let Some(n) = name.last() {
                    let annotations = std::mem::take(pending_attributes);
                    modules[0].free_variables.push(Field {
                        name: n.clone(),
                        ty,
                        annotations,
                    });
                }
            }
            ParsedItem::ImplBlock(ib) => modules[0].impl_blocks.push(ib),
            ParsedItem::Imports(i) => modules[0].imports.extend(i),
            ParsedItem::Component(Component::TypeAlias(t)) => {
                modules[0].type_aliases.push(t);
                pending_attributes.clear();
            }
            ParsedItem::Component(Component::Primitive(_)) => {}
            ParsedItem::Component(Component::External(_)) => {}
            ParsedItem::Attribute(attrs) => {
                pending_attributes.extend(attrs);
            }
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_cst(
            child,
            source,
            modules,
            pending_attributes,
            lang_name,
            file_path.clone(),
            config,
        );
    }
}
