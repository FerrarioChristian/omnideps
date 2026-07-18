use super::components::{Component, QualifiedName};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum DependencyEdgeKind {
    IsA,
    Implements,
    UsesFieldType,
    UsesLocalType,
    UsesParamType,
    UsesReturnType,
    NestedIn,
    ModuleContainment,
    Calls,
    Instantiates,
    AccessesField,
    Imports,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct Dependency {
    pub from: QualifiedName,
    pub to: QualifiedName,
    pub kind: DependencyEdgeKind,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<Component>,
    pub edges: Vec<Dependency>,
}

// Per i benchmark
#[derive(Debug, Default)]
pub struct AnalysisSummary {
    pub total_modules: usize,
    pub total_structured_types: usize,
    pub total_free_functions: usize,
    pub resolved_refs: usize,
    pub failed_refs: usize,
}