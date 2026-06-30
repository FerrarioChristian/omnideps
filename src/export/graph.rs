use crate::model::*;

/// Constructs a dependency graph linking components based on inheritance, types used in fields, parameters, etc.
pub fn build_dependency_graph(modules: &[Module]) -> DependencyGraph {
    let nodes = flatten_modules(modules);
    let mut edges = vec![];

    for m in modules {
        traverse_module_for_edges(m, None, &mut edges);
    }
    
    // Deduplicate edges to prevent inflated coupling metrics
    edges.sort();
    edges.dedup();

    DependencyGraph { nodes, edges }
}

fn traverse_module_for_edges(
    m: &Module,
    parent_id: Option<&QualifiedName>,
    edges: &mut Vec<Dependency>,
) {
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

fn type_ref_target(tr: &TypeRef) -> Option<&QualifiedName> {
    match tr {
        TypeRef::Resolved(to) | TypeRef::External(to) => Some(to),
        _ => None,
    }
}

fn add_super_edges(st: &StructuredType, edges: &mut Vec<Dependency>) {
    for sup in &st.super_types {
        if let Some(to) = type_ref_target(sup) {
            edges.push(Dependency {
                from: st.name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::IsA,
            });
        }
    }
}

fn add_field_edges(st: &StructuredType, edges: &mut Vec<Dependency>) {
    for f in &st.fields {
        if let Some(to) = type_ref_target(&f.ty) {
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
        if let Some(to) = type_ref_target(&p.ty) {
            edges.push(Dependency {
                from: ff.name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::UsesParamType,
            });
        }
    }
    if let Some(to) = type_ref_target(&ff.signature.return_type) {
        edges.push(Dependency {
            from: ff.name.clone(),
            to: to.clone(),
            kind: DependencyEdgeKind::UsesReturnType,
        });
    }

    if let Some(body) = &ff.body {
        add_block_edges(ff, body, edges);
    }
}

fn add_block_edges(ff: &Function, block: &Block, edges: &mut Vec<Dependency>) {
    // 1. Declarations (Local variables)
    for decl in &block.declarations {
        if let Some(to) = type_ref_target(&decl.ty) {
            edges.push(Dependency {
                from: ff.name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::UsesLocalType,
            });
        }
    }

    // 2. Behavioral dependencies
    for call in &block.calls {
        if let Some(to) = type_ref_target(call) {
            edges.push(Dependency {
                from: ff.name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::Calls,
            });
        }
    }
    for inst in &block.instantiates {
        if let Some(to) = type_ref_target(inst) {
            edges.push(Dependency {
                from: ff.name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::Instantiates,
            });
        }
    }

    // 3. Recurse into sub-blocks
    for sub in &block.sub_blocks {
        add_block_edges(ff, sub, edges);
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
