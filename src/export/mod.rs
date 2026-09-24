//! # Phase 3: Dependency Graph Synthesis & Architectural Projection ($\gamma : \mathcal{D}_{\text{res}} \to \mathcal{G}$)
//!
//! Synthesizes a directed dependency graph $\mathcal{G} = \langle V, E \rangle$ from the resolved
//! semantic Intermediate Representation $\mathcal{D}_{\text{res}}$, as formalised in Chapter 8.
//!
//! ## Subsystems
//!
//! - **[`graph`]**: Implements graph construction $\gamma$, flattening module hierarchies into
//!   a global node set $V$ and extracting typed dependency edges $E$ across 16 formal edge kinds.
//!   Also performs edge deduplication and synthetic node generation for primitives and external libraries.
//! - **[`cytoscape`]**: Projects the dependency graph into a Cytoscape.js compatible JSON format
//!   for interactive web visualization and compound layout rendering.
//! - **[`summary`]**: Aggregates comprehensive quantitative metrics (module counts, resolved vs failed references)
//!   for benchmark verification and validation reporting.

pub mod cytoscape;
pub mod graph;
pub mod summary;

pub use graph::build_dependency_graph;
pub use summary::build_analysis_summary;
