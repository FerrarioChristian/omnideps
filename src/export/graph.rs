use crate::ir::*;

/// Constructs a dependency graph linking components based on inheritance, types used in fields, parameters, etc.
pub fn build_dependency_graph(modules: &[Module]) -> DependencyGraph {
    let nodes = flatten_modules(modules);
    let mut edges = vec![];

    for m in modules {
        traverse_module_for_edges(m, None, &mut edges);
    }

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
        match sup {
            TypeRef::Resolved(to) | TypeRef::External(to) => {
                edges.push(Dependency {
                    from: st.name.clone(),
                    to: to.clone(),
                    kind: DependencyEdgeKind::Inherits,
                });
            }
            _ => {}
        }
    }
}

fn add_field_edges(st: &StructuredType, edges: &mut Vec<Dependency>) {
    for f in &st.fields {
        match &f.ty {
            TypeRef::Resolved(to) | TypeRef::External(to) => {
                edges.push(Dependency {
                    from: st.name.clone(),
                    to: to.clone(),
                    kind: DependencyEdgeKind::UsesFieldType,
                });
            }
            _ => {}
        }
    }
}

fn add_function_edges(ff: &Function, edges: &mut Vec<Dependency>) {
    for p in &ff.signature.parameters {
        match &p.ty {
            TypeRef::Resolved(to) | TypeRef::External(to) => {
                edges.push(Dependency {
                    from: ff.name.clone(),
                    to: to.clone(),
                    kind: DependencyEdgeKind::UsesParamType,
                });
            }
            _ => {}
        }
    }
    match &ff.signature.return_type {
        TypeRef::Resolved(to) | TypeRef::External(to) => {
            edges.push(Dependency {
                from: ff.name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::UsesReturnType,
            });
        }
        _ => {}
    }
    for call in &ff.calls {
        match call {
            TypeRef::Resolved(to) | TypeRef::External(to) => {
                edges.push(Dependency {
                    from: ff.name.clone(),
                    to: to.clone(),
                    kind: DependencyEdgeKind::Calls,
                });
            }
            _ => {}
        }
    }
    for inst in &ff.instantiates {
        match inst {
            TypeRef::Resolved(to) | TypeRef::External(to) => {
                edges.push(Dependency {
                    from: ff.name.clone(),
                    to: to.clone(),
                    kind: DependencyEdgeKind::Instantiates,
                });
            }
            _ => {}
        }
    }
}

fn add_impl_edges(ib: &ImplBlock, edges: &mut Vec<Dependency>) {
    match &ib.impl_for {
        TypeRef::Resolved(to) | TypeRef::External(to) => {
            edges.push(Dependency {
                from: ib.name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::Implements,
            });
        }
        _ => {}
    }
    match &ib.implements_trait {
        Some(TypeRef::Resolved(to)) | Some(TypeRef::External(to)) => {
            edges.push(Dependency {
                from: ib.name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::Implements,
            });
        }
        _ => {}
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
