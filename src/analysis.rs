use crate::ir::*;
use std::collections::HashMap;

// ==================== CONTEXT PER RISOLUZIONE NOMI ====================
#[derive(Debug)]
pub struct ResolutionContext {
    pub current_prefix: QualifiedName,
    pub symbol_table: HashMap<QualifiedName, Component>,
}

pub fn build_symbol_table(modules: &[Module]) -> HashMap<QualifiedName, Component> {
    let mut table = HashMap::new();
    fn populate(
        m: &Module,
        table: &mut HashMap<QualifiedName, Component>,
        prefix: &mut QualifiedName,
    ) {
        prefix.push(m.name.last().cloned().unwrap_or_default());
        table.insert(prefix.clone(), Component::Module(m.clone()));

        for st in &m.structured_types {
            let mut name = prefix.clone();
            name.extend(st.name.clone());
            table.insert(name, Component::StructuredType(st.clone()));
        }
        for f in &m.free_functions {
            let mut name = prefix.clone();
            name.push(f.name.clone());
            table.insert(name, Component::FreeFunction(f.clone()));
        }
        for i in &m.impl_blocks {
            let mut name = prefix.clone();
            name.extend(i.name.clone());
            table.insert(name, Component::ImplBlock(i.clone()));
        }
        for sub in &m.sub_modules {
            populate(sub, table, prefix);
        }
        prefix.pop();
    }

    let mut prefix = vec![];
    for m in modules {
        populate(m, &mut table, &mut prefix);
    }
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

// Regole di risoluzione (come formalizzate: assoluto -> relativo -> enclosing)
fn resolve_name_in_context(ctx: &ResolutionContext, name: &QualifiedName) -> Option<QualifiedName> {
    // 1. Assoluto
    if ctx.symbol_table.contains_key(name) {
        return Some(name.clone());
    }
    // 2. Relativo al current_prefix
    let mut relative = ctx.current_prefix.clone();
    relative.extend(name.clone());
    if ctx.symbol_table.contains_key(&relative) {
        return Some(relative);
    }
    // 3. Climb sugli enclosing scopes
    let mut prefix = ctx.current_prefix.clone();
    while !prefix.is_empty() {
        prefix.pop();
        let mut candidate = prefix.clone();
        candidate.extend(name.clone());
        if ctx.symbol_table.contains_key(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn resolve_type_ref(ctx: &ResolutionContext, tr: TypeRef) -> TypeRef {
    match tr {
        TypeRef::UserDefined(name) => {
            if let Some(resolved) = resolve_name_in_context(ctx, &name) {
                TypeRef::UserDefined(resolved)
            } else {
                TypeRef::Unknown
            }
        }
        other => other,
    }
}

// Risoluzione ricorsiva (stessa struttura delle regole di inferenza)
fn resolve_module_in_context(ctx: &ResolutionContext, mut module: Module) -> Module {
    let mut new_prefix = ctx.current_prefix.clone();
    new_prefix.extend(module.name.clone());

    let new_ctx = ResolutionContext {
        current_prefix: new_prefix.clone(),
        symbol_table: ctx.symbol_table.clone(),
    };

    module.structured_types = module
        .structured_types
        .into_iter()
        .map(|st| resolve_structured_type(&new_ctx, st))
        .collect();
    module.free_functions = module
        .free_functions
        .into_iter()
        .map(|f| resolve_free_function(&new_ctx, f))
        .collect();
    module.impl_blocks = module
        .impl_blocks
        .into_iter()
        .map(|i| resolve_impl_block(&new_ctx, i))
        .collect();
    module.sub_modules = module
        .sub_modules
        .into_iter()
        .map(|sub| resolve_module_in_context(&new_ctx, sub))
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
    i.implements_trait = i.implements_trait.map(|t| resolve_type_ref(ctx, t));
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

// ==================== BUILD DEPENDENCY GRAPH ====================
pub fn build_dependency_graph(modules: &[Module]) -> DependencyGraph {
    let nodes = flatten_modules(modules);
    let mut edges = vec![];

    for node in &nodes {
        match node {
            Component::StructuredType(st) => {
                add_super_edges(st, &mut edges);
                add_field_edges(st, &mut edges);
                add_method_edges(st, &mut edges);
            }
            Component::FreeFunction(ff) => add_free_function_edges(ff, &mut edges),
            Component::ImplBlock(ib) => add_impl_edges(ib, &mut edges),
            _ => {}
        }
    }

    DependencyGraph { nodes, edges }
}

fn add_super_edges(st: &StructuredType, edges: &mut Vec<Dependency>) {
    for sup in &st.super_types {
        if let TypeRef::UserDefined(to) = sup {
            edges.push(Dependency {
                from: st.name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::Inherits,
            });
        }
    }
}

fn add_field_edges(st: &StructuredType, edges: &mut Vec<Dependency>) {
    for f in &st.fields {
        if let TypeRef::UserDefined(to) = &f.ty {
            edges.push(Dependency {
                from: st.name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::UsesFieldType,
            });
        }
    }
}

fn add_method_edges(st: &StructuredType, edges: &mut Vec<Dependency>) {
    for m in &st.methods {
        for p in &m.parameters {
            if let TypeRef::UserDefined(to) = &p.ty {
                edges.push(Dependency {
                    from: st.name.clone(),
                    to: to.clone(),
                    kind: DependencyEdgeKind::UsesMethodParam,
                });
            }
        }
        if let TypeRef::UserDefined(to) = &m.return_type {
            edges.push(Dependency {
                from: st.name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::UsesMethodReturn,
            });
        }
    }
}

fn add_free_function_edges(ff: &FreeFunction, edges: &mut Vec<Dependency>) {
    for p in &ff.parameters {
        if let TypeRef::UserDefined(to) = &p.ty {
            edges.push(Dependency {
                from: vec![ff.name.clone()],
                to: to.clone(),
                kind: DependencyEdgeKind::UsesFreeFunctionParam,
            });
        }
    }
    if let TypeRef::UserDefined(to) = &ff.return_type {
        edges.push(Dependency {
            from: vec![ff.name.clone()],
            to: to.clone(),
            kind: DependencyEdgeKind::UsesFreeFunctionReturn,
        });
    }
}

fn add_impl_edges(ib: &ImplBlock, edges: &mut Vec<Dependency>) {
    if let TypeRef::UserDefined(to) = &ib.impl_for {
        edges.push(Dependency {
            from: ib.name.clone(),
            to: to.clone(),
            kind: DependencyEdgeKind::ImplementsFor,
        });
    }
    if let Some(TypeRef::UserDefined(to)) = &ib.implements_trait {
        edges.push(Dependency {
            from: ib.name.clone(),
            to: to.clone(),
            kind: DependencyEdgeKind::ImplementsTrait,
        });
    }
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

// ==================== BENCHMARK ====================
pub fn build_analysis_summary(modules: &[Module]) -> AnalysisSummary {
    let mut s = AnalysisSummary::default();
    s.total_modules = modules.len();
    for m in modules {
        s.total_structured_types += m.structured_types.len();
        s.total_free_functions += m.free_functions.len();
        s.total_impl_blocks += m.impl_blocks.len();
    }
    s
}
