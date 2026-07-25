use crate::model::*;

/// Constructs a dependency graph linking components based on inheritance, types used in fields, parameters, etc.
pub fn build_dependency_graph(modules: &[Module], primitives: &crate::resolver::primitives::PrimitiveRegistry) -> DependencyGraph {
    let mut nodes = flatten_modules(modules, vec![]);
    let mut edges = vec![];

    for m in modules {
        traverse_module_for_edges(m, None, vec![], &mut edges);
    }
    
    // Create primitive nodes only on demand (if they are actually targeted by an edge)
    let mut used_primitives = std::collections::HashSet::new();
    for edge in &edges {
        if edge.to.len() == 1 && primitives.is_primitive(&edge.to[0]) {
            used_primitives.insert(edge.to[0].clone());
        }
    }
    for prim in used_primitives {
        nodes.push(Component::Primitive(prim));
    }
    
    // Deduplicate edges to prevent inflated coupling metrics
    edges.sort();
    edges.dedup();

    DependencyGraph { nodes, edges }
}

fn traverse_module_for_edges(
    m: &Module,
    parent_id: Option<&QualifiedName>,
    prefix: QualifiedName,
    edges: &mut Vec<Dependency>,
) {
    let mut m_name = prefix.clone();
    if !m.name.is_empty() && (m.name.len() > 1 || m.name[0] != "root" || prefix.is_empty()) {
        m_name.extend(m.name.clone());
    }

    if let Some(parent) = parent_id {
        edges.push(Dependency {
            from: parent.clone(),
            to: m_name.clone(),
            kind: DependencyEdgeKind::ModuleContainment,
        });
    }

    for import in &m.imports {
        edges.push(Dependency {
            from: m_name.clone(),
            to: import.path.clone(),
            kind: DependencyEdgeKind::Imports,
        });
    }

    for ta in &m.type_aliases {
        let mut ta_name = m_name.clone();
        ta_name.extend(ta.name.clone());
        if let Some(to) = type_ref_target(&ta.target) {
            edges.push(Dependency {
                from: ta_name.clone(),
                to,
                kind: DependencyEdgeKind::Aliases,
            });
        }
    }

    for st in &m.structured_types {
        let mut st_name = m_name.clone();
        st_name.extend(st.name.clone());
        edges.push(Dependency {
            from: m_name.clone(),
            to: st_name.clone(),
            kind: DependencyEdgeKind::ModuleContainment,
        });
        traverse_structured_type_edges(st, &m_name, edges);
    }

    for ff in &m.free_functions {
        let mut ff_name = m_name.clone();
        ff_name.extend(ff.name.clone());
        edges.push(Dependency {
            from: m_name.clone(),
            to: ff_name.clone(),
            kind: DependencyEdgeKind::ModuleContainment,
        });
        add_function_edges(ff, &ff_name, edges);
    }

    for fv in &m.free_variables {
        let mut fv_name = m_name.clone();
        fv_name.push(fv.name.clone());
        edges.push(Dependency {
            from: m_name.clone(),
            to: fv_name.clone(),
            kind: DependencyEdgeKind::ModuleContainment,
        });
        if let Some(to) = type_ref_target(&fv.ty) {
            edges.push(Dependency {
                from: fv_name,
                to: to.clone(),
                kind: DependencyEdgeKind::UsesFieldType,
            });
        }
    }

    for ib in &m.impl_blocks {
        if let Some(to) = type_ref_target(&ib.impl_for) {
            for mut meth in ib.methods.clone() {
                let mut m_name = to.clone();
                m_name.extend(meth.name.clone());
                meth.name = m_name.clone();
                edges.push(Dependency {
                    from: to.clone(),
                    to: m_name.clone(),
                    kind: DependencyEdgeKind::NestedIn,
                });
                add_function_edges(&meth, &m_name, edges);
            }
            for nested in &ib.nested_types {
                let mut nested_name = to.clone();
                nested_name.extend(nested.name.clone());
                edges.push(Dependency {
                    from: to.clone(),
                    to: nested_name.clone(),
                    kind: DependencyEdgeKind::NestedIn,
                });
                traverse_structured_type_edges(nested, &to, edges);
            }
        }
    }

    for sub in &m.sub_modules {
        traverse_module_for_edges(sub, Some(&m_name), m_name.clone(), edges);
    }
}

fn traverse_structured_type_edges(st: &StructuredType, prefix: &QualifiedName, edges: &mut Vec<Dependency>) {
    let mut st_name = prefix.clone();
    st_name.extend(st.name.clone());

    add_super_edges(st, &st_name, edges);
    add_field_edges(st, &st_name, edges);

    for f in &st.fields {
        let mut f_name = st_name.clone();
        f_name.push(f.name.clone());
        edges.push(Dependency {
            from: st_name.clone(),
            to: f_name,
            kind: DependencyEdgeKind::NestedIn,
        });
    }

    for m in &st.methods {
        let mut m_name = st_name.clone();
        m_name.extend(m.name.clone());
        edges.push(Dependency {
            from: st_name.clone(),
            to: m_name.clone(),
            kind: DependencyEdgeKind::NestedIn,
        });
        add_function_edges(m, &m_name, edges);
    }

    for nested in &st.nested_types {
        let mut nested_name = st_name.clone();
        nested_name.extend(nested.name.clone());
        edges.push(Dependency {
            from: st_name.clone(),
            to: nested_name.clone(),
            kind: DependencyEdgeKind::NestedIn,
        });
        traverse_structured_type_edges(nested, &st_name, edges);
    }
}

fn type_ref_target(tr: &TypeRef) -> Option<QualifiedName> {
    match tr {
        TypeRef::Resolved(to) | TypeRef::External(to) => Some(to.clone()),
        TypeRef::Primitive(s) => Some(vec![s.clone()]),
        _ => None,
    }
}

fn add_super_edges(st: &StructuredType, st_name: &QualifiedName, edges: &mut Vec<Dependency>) {
    for sup in &st.super_types {
        if let Some(to) = type_ref_target(sup) {
            edges.push(Dependency {
                from: st_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::IsA,
            });
        }
    }
}

fn add_field_edges(st: &StructuredType, st_name: &QualifiedName, edges: &mut Vec<Dependency>) {
    for f in &st.fields {
        if let Some(to) = type_ref_target(&f.ty) {
            let mut f_name = st_name.clone();
            f_name.push(f.name.clone());
            edges.push(Dependency {
                from: f_name,
                to: to.clone(),
                kind: DependencyEdgeKind::UsesFieldType,
            });
        }
    }
}

fn add_function_edges(ff: &Function, ff_name: &QualifiedName, edges: &mut Vec<Dependency>) {
    for p in &ff.signature.parameters {
        if let Some(to) = type_ref_target(&p.ty) {
            edges.push(Dependency {
                from: ff_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::UsesParamType,
            });
        }
    }
    if let Some(to) = type_ref_target(&ff.signature.return_type) {
        edges.push(Dependency {
            from: ff_name.clone(),
            to: to.clone(),
            kind: DependencyEdgeKind::UsesReturnType,
        });
    }

    if let Some(body) = &ff.body {
        add_block_edges(ff, ff_name, body, edges);
    }
}

fn add_block_edges(ff: &Function, ff_name: &QualifiedName, block: &Block, edges: &mut Vec<Dependency>) {
    // 1. Declarations (Local variables)
    for decl in &block.declarations {
        if let Some(to) = type_ref_target(&decl.ty) {
            edges.push(Dependency {
                from: ff_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::UsesLocalType,
            });
        }
    }

    // 2. Behavioral dependencies
    for call in &block.calls {
        if let Some(to) = type_ref_target(call) {
            edges.push(Dependency {
                from: ff_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::Calls,
            });
        }
    }
    for inst in &block.instantiates {
        if let Some(to) = type_ref_target(inst) {
            edges.push(Dependency {
                from: ff_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::Instantiates,
            });
        }
    }
    for acc in &block.accesses {
        if let Some(to) = type_ref_target(acc) {
            edges.push(Dependency {
                from: ff_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::AccessesField,
            });
        }
    }
    for cast in &block.type_casts {
        if let Some(to) = type_ref_target(cast) {
            edges.push(Dependency {
                from: ff_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::CastsTo,
            });
        }
    }

    // 3. Recurse into sub-blocks
    for sub in &block.sub_blocks {
        add_block_edges(ff, ff_name, sub, edges);
    }
}

fn flatten_modules(modules: &[Module], prefix: QualifiedName) -> Vec<Component> {
    let mut flat = vec![];
    for m in modules {
        let mut m_name = prefix.clone();
        if !m.name.is_empty() && (m.name.len() > 1 || m.name[0] != "root" || prefix.is_empty()) {
            m_name.extend(m.name.clone());
        }

        let mut m_clone = m.clone();
        m_clone.name = m_name.clone();
        flat.push(Component::Module(m_clone));
        
        for ta in &m.type_aliases {
            let mut ta_name = m_name.clone();
            ta_name.extend(ta.name.clone());
            let mut ta_clone = ta.clone();
            ta_clone.name = ta_name.clone();
            flat.push(Component::TypeAlias(ta_clone));
        }

        for st in &m.structured_types {
            flat.extend(flatten_structured_type(st, &m_name));
        }
        
        flat.extend(m.free_functions.iter().cloned().map(|mut ff| {
            let mut ff_name = m_name.clone();
            ff_name.extend(ff.name.clone());
            ff.name = ff_name;
            Component::Function(ff)
        }));
        
        for fv in &m.free_variables {
            let mut fv_name = m_name.clone();
            fv_name.push(fv.name.clone());
            flat.push(Component::Field(fv_name, fv.ty.clone()));
        }
        
        for ib in &m.impl_blocks {
            if let Some(to) = type_ref_target(&ib.impl_for) {
                for mut m in ib.methods.clone() {
                    let mut m_name = to.clone();
                    m_name.extend(m.name.clone());
                    m.name = m_name;
                    flat.push(Component::Function(m));
                }
                for nested in &ib.nested_types {
                    flat.extend(flatten_structured_type(nested, &to));
                }
            }
        }
        
        flat.extend(flatten_modules(&m.sub_modules, m_name.clone()));
    }
    flat
}

fn flatten_structured_type(st: &StructuredType, prefix: &QualifiedName) -> Vec<Component> {
    let mut st_name = prefix.clone();
    st_name.extend(st.name.clone());
    
    let mut st_clone = st.clone();
    st_clone.name = st_name.clone();
    let mut flat = vec![Component::StructuredType(st_clone)];
    
    for f in &st.fields {
        let mut f_name = st_name.clone();
        f_name.push(f.name.clone());
        flat.push(Component::Field(f_name, f.ty.clone()));
    }
    
    flat.extend(st.methods.iter().cloned().map(|mut m| {
        let mut m_name = st_name.clone();
        m_name.extend(m.name.clone());
        m.name = m_name;
        Component::Function(m)
    }));
    
    for nested in &st.nested_types {
        flat.extend(flatten_structured_type(nested, &st_name));
    }
    flat
}
