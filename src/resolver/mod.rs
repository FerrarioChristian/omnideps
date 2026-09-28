//! # Phase 2: Symbolic Reference Resolution ($\rho : \mathcal{D} \to \mathcal{D}_{\text{res}}$)
//!
//! Transforms syntactic type references extracted during Phase 1 ($\varepsilon$) into fully
//! resolved semantic pointers to concrete declarations, as formalised in Chapter 7.
//!
//! The resolution process is partitioned into two functional stages:
//!
//! 1. **Query Building ($\rho_{\text{build}}$)** ([`builder::build_queries`]):
//!    Traverses local lexical scopes using the Symbol Stack $\Delta = [\sigma_0, \sigma_1, \dots, \sigma_k]$
//!    to rewrite raw identifiers, local variable uses, and implicit `self`/`this` parameters
//!    into structured resolution queries ([`crate::model::Query`]).
//!
//! 2. **Query Execution ($\rho_{\text{exec}}$)** ([`executor::execute_queries`]):
//!    Constructs the global Scope Tree $\mathcal{E}$ and evaluates all queries within
//!    an immutable execution context $\Gamma = \langle \mathcal{E}, \mathcal{P}, \mathcal{K} \rangle$,
//!    where $\mathcal{P}$ is the [`PrimitiveRegistry`] and $\mathcal{K}$ is the [`crate::config::AnalyzerConfig`].
//!
//! Complete formalisation:
//! $$\rho = \rho_{\text{exec}} \circ \rho_{\text{build}} : \mathcal{D} \to \mathcal{D}_{\text{res}}$$

pub mod builder;
pub mod executor;
pub mod primitives;
pub mod scope;
pub mod stack;

use crate::config::AnalyzerConfig;
use crate::model::Module;
use primitives::PrimitiveRegistry;

/// Executes the complete two-phase Name Resolution pipeline ($\rho = \rho_{\text{exec}} \circ \rho_{\text{build}}$).
///
/// Linearly coordinates:
/// 1. Query Building via [`builder::build_queries`].
/// 2. Query Execution via [`executor::execute_queries`].
///
/// # Arguments
/// * `modules` - Unresolved IR modules $\mathcal{D}$ extracted during Phase 1.
/// * `primitives` - Primitive type registry $\mathcal{P}$.
/// * `config` - Analyzer configuration $\mathcal{K}$.
///
/// # Returns
/// Fully resolved IR modules $\mathcal{D}_{\text{res}}$.
pub fn resolve_type_refs(
    modules: Vec<Module>,
    primitives: &PrimitiveRegistry,
    config: &AnalyzerConfig,
) -> Vec<Module> {
    let modules_with_queries = builder::build_queries(modules, config);
    executor::execute_queries(modules_with_queries, primitives, config)
}
