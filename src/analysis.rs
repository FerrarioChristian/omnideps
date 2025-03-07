use crate::ir::*;
use std::collections::HashMap;

// ==================== CONTEXT PER RISOLUZIONE NOMI ====================
/// Contains the current prefix (namespace) and a flat symbol table for fast lookups.
#[derive(Debug)]
pub struct ResolutionContext {
    pub current_prefix: QualifiedName,
    pub symbol_table: HashMap<QualifiedName, Component>,
}

/// Builds a flat symbol table from a list of root modules, indexing every nested component.
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
            name.extend(f.name.clone());
            table.insert(name, Component::Function(f.clone()));
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

/// Resolves type references across all modules by matching them against the global symbol table.
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
        TypeRef::Unresolved(name) => {
            if let Some(resolved) = resolve_name_in_context(ctx, &name) {
                TypeRef::Resolved(resolved)
            } else {
                TypeRef::Failed(name)
            }
        }
        other => other,
    }
}

// Risoluzione ricorsiva (stessa struttura delle regole di inferenza)
fn resolve_module_in_context(ctx: &ResolutionContext, mut module: Module) -> Module {
    let mut new_prefix = ctx.current_prefix.clone();
    new_prefix.extend(module.name.clone());

    module.name = new_prefix.clone();

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
        .map(|f| resolve_function(&new_ctx, f))
        .collect();

    let resolved_impls: Vec<ImplBlock> = module
        .impl_blocks
        .into_iter()
        .map(|i| resolve_impl_block(&new_ctx, i))
        .collect();

    for ib in resolved_impls {
        if let TypeRef::Resolved(target_name) = &ib.impl_for {
            if let Some(target_st) = module.structured_types.iter_mut().find(|st| {
                &st.name == target_name
            }) {
                target_st.methods.extend(ib.methods.clone());
                target_st.nested_types.extend(ib.nested_types.clone());
                if let Some(trait_ref) = ib.implements_trait.clone() {
                    target_st.super_types.push(trait_ref);
                }
            }
        }
    }
    module.impl_blocks = vec![]; // Flattened

    module.sub_modules = module
        .sub_modules
        .into_iter()
        .map(|sub| resolve_module_in_context(&new_ctx, sub))
        .collect();

    module
}

fn resolve_structured_type(ctx: &ResolutionContext, mut st: StructuredType) -> StructuredType {
    let mut new_prefix = ctx.current_prefix.clone();
    new_prefix.extend(st.name.clone());
    st.name = new_prefix.clone();

    let new_ctx = ResolutionContext {
        current_prefix: new_prefix,
        symbol_table: ctx.symbol_table.clone(),
    };

    st.super_types = st
        .super_types
        .into_iter()
        .map(|tr| resolve_type_ref(&new_ctx, tr))
        .collect();
    st.fields = st
        .fields
        .into_iter()
        .map(|f| Field {
            name: f.name,
            ty: resolve_type_ref(&new_ctx, f.ty),
        })
        .collect();
    st.methods = st
        .methods
        .into_iter()
        .map(|m| resolve_function(&new_ctx, m))
        .collect();
    st.nested_types = st
        .nested_types
        .into_iter()
        .map(|n| resolve_structured_type(&new_ctx, n))
        .collect();
    st
}

fn resolve_function(ctx: &ResolutionContext, mut f: Function) -> Function {
    let mut new_prefix = ctx.current_prefix.clone();
    new_prefix.extend(f.name.clone());
    f.name = new_prefix;

    f.signature.parameters = f
        .signature.parameters
        .into_iter()
        .map(|p| Parameter {
            name: p.name,
            ty: resolve_type_ref(ctx, p.ty),
            is_variadic: p.is_variadic,
        })
        .collect();
    f.signature.return_type = resolve_type_ref(ctx, f.signature.return_type);
    f
}

fn resolve_impl_block(ctx: &ResolutionContext, mut i: ImplBlock) -> ImplBlock {
    let mut new_prefix = ctx.current_prefix.clone();
    new_prefix.extend(i.name.clone());
    i.name = new_prefix.clone();

    let new_ctx = ResolutionContext {
        current_prefix: new_prefix,
        symbol_table: ctx.symbol_table.clone(),
    };

    i.impl_for = resolve_type_ref(&new_ctx, i.impl_for);
    i.implements_trait = i.implements_trait.map(|t| resolve_type_ref(&new_ctx, t));
    i.methods = i
        .methods
        .into_iter()
        .map(|m| resolve_function(&new_ctx, m))
        .collect();
    i.nested_types = i
        .nested_types
        .into_iter()
        .map(|n| resolve_structured_type(&new_ctx, n))
        .collect();
    i
}

// ==================== BUILD DEPENDENCY GRAPH ====================
/// Constructs a dependency graph linking components based on inheritance, types used in fields, parameters, etc.
pub fn build_dependency_graph(modules: &[Module]) -> DependencyGraph {
    let nodes = flatten_modules(modules);
    let mut edges = vec![];
    
    for m in modules {
        traverse_module_for_edges(m, None, &mut edges);
    }

    DependencyGraph { nodes, edges }
}

fn traverse_module_for_edges(m: &Module, parent_id: Option<&QualifiedName>, edges: &mut Vec<Dependency>) {
    if let Some(parent) = parent_id {
        edges.push(Dependency {
            from: parent.clone(),
            to: m.name.clone(),
            kind: DependencyEdgeKind::ModuleContainment,
        });
    }
    
    for st in &m.structured_types {
        edges.push(Dependency {
            from: m.name.clone(),
            to: st.name.clone(),
            kind: DependencyEdgeKind::ModuleContainment,
        });
        traverse_structured_type_edges(st, edges);
    }
    
    for ff in &m.free_functions {
        edges.push(Dependency {
            from: m.name.clone(),
            to: ff.name.clone(),
            kind: DependencyEdgeKind::ModuleContainment,
        });
        add_function_edges(ff, edges);
    }
    
    for ib in &m.impl_blocks {
        edges.push(Dependency {
            from: m.name.clone(),
            to: ib.name.clone(),
            kind: DependencyEdgeKind::ModuleContainment,
        });
        add_impl_edges(ib, edges);
    }
    
    for sub in &m.sub_modules {
        traverse_module_for_edges(sub, Some(&m.name), edges);
    }
}

fn traverse_structured_type_edges(st: &StructuredType, edges: &mut Vec<Dependency>) {
    add_super_edges(st, edges);
    add_field_edges(st, edges);
    
    for m in &st.methods {
        edges.push(Dependency {
            from: st.name.clone(),
            to: m.name.clone(),
            kind: DependencyEdgeKind::NestedIn,
        });
        add_function_edges(m, edges);
    }
    
    for nested in &st.nested_types {
        edges.push(Dependency {
            from: st.name.clone(),
            to: nested.name.clone(),
            kind: DependencyEdgeKind::NestedIn,
        });
        traverse_structured_type_edges(nested, edges);
    }
}

fn add_super_edges(st: &StructuredType, edges: &mut Vec<Dependency>) {
    for sup in &st.super_types {
        if let TypeRef::Resolved(to) = sup {
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
        if let TypeRef::Resolved(to) = &f.ty {
            edges.push(Dependency {
                from: st.name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::UsesFieldType,
            });
        }
    }
}

fn add_function_edges(ff: &Function, edges: &mut Vec<Dependency>) {
    for p in &ff.signature.parameters {
        if let TypeRef::Resolved(to) = &p.ty {
            edges.push(Dependency {
                from: ff.name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::UsesParamType,
            });
        }
    }
    if let TypeRef::Resolved(to) = &ff.signature.return_type {
        edges.push(Dependency {
            from: ff.name.clone(),
            to: to.clone(),
            kind: DependencyEdgeKind::UsesReturnType,
        });
    }
}

fn add_impl_edges(ib: &ImplBlock, edges: &mut Vec<Dependency>) {
    if let TypeRef::Resolved(to) = &ib.impl_for {
        edges.push(Dependency {
            from: ib.name.clone(),
            to: to.clone(),
            kind: DependencyEdgeKind::Implements,
        });
    }
    if let Some(TypeRef::Resolved(to)) = &ib.implements_trait {
        edges.push(Dependency {
            from: ib.name.clone(),
            to: to.clone(),
            kind: DependencyEdgeKind::Implements,
        });
    }
    for m in &ib.methods {
        edges.push(Dependency {
            from: ib.name.clone(),
            to: m.name.clone(),
            kind: DependencyEdgeKind::NestedIn,
        });
        add_function_edges(m, edges);
    }
    for nested in &ib.nested_types {
        edges.push(Dependency {
            from: ib.name.clone(),
            to: nested.name.clone(),
            kind: DependencyEdgeKind::NestedIn,
        });
        traverse_structured_type_edges(nested, edges);
    }
}

fn flatten_modules(modules: &[Module]) -> Vec<Component> {
    let mut flat = vec![];
    for m in modules {
        flat.push(Component::Module(m.clone()));
        for st in &m.structured_types {
            flat.extend(flatten_structured_type(st));
        }
        flat.extend(m.free_functions.iter().cloned().map(Component::Function));
        flat.extend(flatten_modules(&m.sub_modules));
    }
    flat
}

fn flatten_structured_type(st: &StructuredType) -> Vec<Component> {
    let mut flat = vec![Component::StructuredType(st.clone())];
    flat.extend(st.methods.iter().cloned().map(Component::Function));
    for nested in &st.nested_types {
        flat.extend(flatten_structured_type(nested));
    }
    flat
}

// ==================== BENCHMARK ====================
/// Aggregates basic statistics about the extracted components across all provided modules.
pub fn build_analysis_summary(modules: &[Module]) -> AnalysisSummary {
    let mut s = AnalysisSummary {
        total_modules: modules.len(),
        ..Default::default()
    };
    for m in modules {
        s.total_structured_types += m.structured_types.len();
        for st in &m.structured_types {
            s.total_structured_types += count_nested_types(st);
        }
        s.total_free_functions += m.free_functions.len();

        let sub_s = build_analysis_summary(&m.sub_modules);
        s.total_modules += sub_s.total_modules;
        s.total_structured_types += sub_s.total_structured_types;
        s.total_free_functions += sub_s.total_free_functions;
    }
    s
}

fn count_nested_types(st: &StructuredType) -> usize {
    let mut count = st.nested_types.len();
    for nested in &st.nested_types {
        count += count_nested_types(nested);
    }
    count
}
