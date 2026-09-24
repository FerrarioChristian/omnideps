use anyhow::Result;
use std::path::Path;

use crate::config::AnalyzerConfig;
use crate::export::build_dependency_graph;
pub use crate::extraction::{extract_from_cst, extract_ir as parse_path};
use crate::extraction::{extract_ir, parse_source};
use crate::language::SupportedLanguage;
use crate::model::{DependencyGraph, Module};
use crate::resolver::resolve_type_refs;

/// Executes the complete OmniDeps analysis pipeline on a project path ($\phi = \gamma \circ \rho \circ \varepsilon : \mathcal{W} \to \mathcal{G}$).
///
/// Linearly orchestrates the three fundamental macro-phases formalised in Chapter 5:
/// 1. **Phase 1: Syntactic Extraction** ([`extract_ir`]): $\mathcal{W} \to \mathcal{D}$
/// 2. **Phase 2: Name Resolution** ([`resolve_type_refs`]): $\mathcal{D} \to \mathcal{D}_{\text{res}}$
/// 3. **Phase 3: Graph Construction** ([`build_dependency_graph`]): $\mathcal{D}_{\text{res}} \to \mathcal{G}$
///
/// # Arguments
/// * `path` - A path to a single source file or root project directory.
/// * `config` - Global analyzer configuration $\mathcal{K}$.
///
/// # Returns
/// A tuple containing:
/// * `Vec<Module>`: Fully resolved IR modules.
/// * `DependencyGraph`: Directed dependency graph with all structural and type edges.
pub fn analyze_project(
    path: &Path,
    config: &AnalyzerConfig,
) -> Result<(Vec<Module>, DependencyGraph)> {
    // Phase 1: Syntactic Extraction from filesystem (epsilon)
    let (modules, prim_registry) = extract_ir(path, config)?;

    // Phase 2: Symbolic Reference Resolution (rho)
    let resolved = resolve_type_refs(modules, &prim_registry, config);

    // Phase 3: Dependency Graph Construction (gamma)
    let graph = build_dependency_graph(&resolved, &prim_registry);

    Ok((resolved, graph))
}

/// Analyzes a standalone source code snippet directly from an in-memory string.
///
/// Executes the complete pipeline without touching the filesystem:
/// - Phase 1 ($\varepsilon$): parses the source string via [`parse_source`].
/// - Phase 2 ($\rho$): resolves symbolic references via [`resolve_type_refs`].
/// - Phase 3 ($\gamma$): constructs the dependency graph via [`build_dependency_graph`].
///
/// Primarily intended for unit tests and interactive Web API snippet evaluations.
pub fn analyze_code_snippet(
    lang: SupportedLanguage,
    source: &str,
    virtual_filename: &str,
    config: &AnalyzerConfig,
) -> Result<(Vec<Module>, DependencyGraph)> {
    let path = Path::new(virtual_filename);

    // Phase 1: Syntactic Extraction (epsilon)
    let (modules, prim_registry) = parse_source(lang, source, path, config)?;

    // Phase 2: Symbolic Reference Resolution (rho)
    let resolved = resolve_type_refs(modules, &prim_registry, config);

    // Phase 3: Dependency Graph Construction (gamma)
    let graph = build_dependency_graph(&resolved, &prim_registry);

    Ok((resolved, graph))
}
