// src/analysis.rs
use crate::ir::*;
use std::collections::HashMap;

// ==================== CONTEXT PER RISOLUZIONE NOMI ====================
#[derive(Debug)]
pub struct ResolutionContext {
    pub current_prefix: QualifiedName,
    pub symbol_table: HashMap<QualifiedName, Component>,
}

pub fn build_symbol_table(_tree: &[Module]) -> HashMap<QualifiedName, Component> {
    let table = HashMap::new();
    // TODO: popolamento ricorsivo (per ora vuoto, lo espanderemo)
    table
}

pub fn resolve_type_refs(modules: Vec<Module>) -> Vec<Module> {
    let symbol_table = build_symbol_table(&modules);
    let ctx = ResolutionContext {
        current_prefix: vec![],
        symbol_table,
    };
    modules
        .into_iter()
        .map(|m| resolve_module_in_context(&ctx, m))
        .collect()
}

fn resolve_module_in_context(ctx: &ResolutionContext, mut module: Module) -> Module {
    // Risolvi structured_types, free_functions, impl_blocks, sub_modules...
    module.structured_types = module
        .structured_types
        .into_iter()
        .map(|st| resolve_structured_type(ctx, st))
        .collect();
    module.free_functions = module
        .free_functions
        .into_iter()
        .map(|f| resolve_free_function(ctx, f))
        .collect();
    module.impl_blocks = module
        .impl_blocks
        .into_iter()
        .map(|i| resolve_impl_block(ctx, i))
        .collect();
    module.sub_modules = module
        .sub_modules
        .into_iter()
        .map(|sub| resolve_module_in_context(ctx, sub))
        .collect();
    module
}

fn resolve_structured_type(ctx: &ResolutionContext, mut st: StructuredType) -> StructuredType {
    st.super_types = st
        .super_types
        .into_iter()
        .map(|tr| resolve_type_ref(ctx, tr))
        .collect();
    st.fields = st
        .fields
        .into_iter()
        .map(|f| Field {
            name: f.name,
            ty: resolve_type_ref(ctx, f.ty),
        })
        .collect();
    st.methods = st
        .methods
        .into_iter()
        .map(|m| resolve_method(ctx, m))
        .collect();
    st.nested_types = st
        .nested_types
        .into_iter()
        .map(|n| resolve_structured_type(ctx, n))
        .collect();
    st
}

fn resolve_free_function(ctx: &ResolutionContext, mut f: FreeFunction) -> FreeFunction {
    f.parameters = f
        .parameters
        .into_iter()
        .map(|p| Parameter {
            name: p.name,
            ty: resolve_type_ref(ctx, p.ty),
            is_variadic: p.is_variadic,
        })
        .collect();
    f.return_type = resolve_type_ref(ctx, f.return_type);
    f
}

fn resolve_impl_block(ctx: &ResolutionContext, mut i: ImplBlock) -> ImplBlock {
    i.impl_for = resolve_type_ref(ctx, i.impl_for);
    if let Some(t) = i.implements_trait {
        i.implements_trait = Some(resolve_type_ref(ctx, t));
    }
    i.methods = i
        .methods
        .into_iter()
        .map(|m| resolve_method(ctx, m))
        .collect();
    i
}

fn resolve_method(ctx: &ResolutionContext, mut m: Method) -> Method {
    m.parameters = m
        .parameters
        .into_iter()
        .map(|p| Parameter {
            name: p.name,
            ty: resolve_type_ref(ctx, p.ty),
            is_variadic: p.is_variadic,
        })
        .collect();
    m.return_type = resolve_type_ref(ctx, m.return_type);
    m
}

fn resolve_type_ref(_ctx: &ResolutionContext, tr: TypeRef) -> TypeRef {
    // TODO: qui useremo il contesto per risolvere UserDefined
    tr // per ora identità
}

// ==================== BUILD DEPENDENCY GRAPH ====================
pub fn build_dependency_graph(modules: &[Module]) -> DependencyGraph {
    let nodes = flatten_modules(modules);
    let edges = vec![]; // TODO: generazione edge (prossimo passo)
    DependencyGraph { nodes, edges }
}

fn flatten_modules(modules: &[Module]) -> Vec<Component> {
    let mut flat = vec![];
    for m in modules {
        flat.push(Component::Module(m.clone()));
        flat.extend(
            m.structured_types
                .iter()
                .cloned()
                .map(Component::StructuredType),
        );
        flat.extend(
            m.free_functions
                .iter()
                .cloned()
                .map(Component::FreeFunction),
        );
        flat.extend(m.impl_blocks.iter().cloned().map(Component::ImplBlock));
        flat.extend(flatten_modules(&m.sub_modules));
    }
    flat
}

// ==================== BENCHMARK SUMMARY ====================
pub fn build_analysis_summary(modules: &[Module]) -> AnalysisSummary {
    let mut summary = AnalysisSummary::default();
    summary.total_modules = modules.len();
    for m in modules {
        summary.total_structured_types += m.structured_types.len();
        summary.total_free_functions += m.free_functions.len();
        summary.total_impl_blocks += m.impl_blocks.len();
    }
    summary
}
