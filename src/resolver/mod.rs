pub mod builder;
pub mod executor;
pub mod primitives;
pub mod scope;
pub mod stack;

use crate::model::*;
use primitives::PrimitiveRegistry;

/// Orchestrates the two-phase Name Resolution:
/// 1. Query Building (Lexical Substitution)
/// 2. Query Execution (Global Navigation)
pub fn resolve_type_refs(
    modules: Vec<Module>,
    primitives: &PrimitiveRegistry,
    config: &crate::config::AnalyzerConfig,
) -> Vec<Module> {
    // Phase 1: Substitution
    let modules_with_queries = builder::build_queries(modules, config);

    // Phase 2: Navigation

    executor::execute_queries(modules_with_queries, primitives, config)
}
