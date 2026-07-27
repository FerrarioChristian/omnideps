use crate::model::*;

/// Constructs a dependency graph linking components based on inheritance, types used in fields, parameters, etc.
pub fn build_dependency_graph(
    modules: &[Module],
    primitives: &crate::resolver::primitives::PrimitiveRegistry,
) -> DependencyGraph {
    let mut nodes = flatten_modules(modules, vec![]);
    let mut edges = vec![];

    for m in modules {
        traverse_module_for_edges(m, None, vec![], &mut edges);
    }

    let mut used_primitives = std::collections::HashSet::new();
    let mut used_unresolved = std::collections::HashSet::new();

    let mut existing_node_names = std::collections::HashSet::new();
    for n in &nodes {
        match n {
            Component::Module(m) => {
                existing_node_names.insert(m.name.clone());
            }
            Component::StructuredType(s) => {
                existing_node_names.insert(s.name.clone());
            }
            Component::TypeAlias(t) => {
                existing_node_names.insert(t.name.clone());
            }
            Component::Function(f) => {
                existing_node_names.insert(f.name.clone());
            }
            Component::Field(name, _) => {
                existing_node_names.insert(name.clone());
            }
            Component::Primitive(_) | Component::External(_) => {}
        }
    }

    for edge in &edges {
        if edge.to.len() == 1 && primitives.is_primitive(&edge.to[0]) {
            used_primitives.insert(edge.to[0].clone());
        } else if !existing_node_names.contains(&edge.to) {
            used_unresolved.insert(edge.to.clone());
        }
    }

    for prim in used_primitives {
        nodes.push(Component::Primitive(prim));
    }
    for unres in used_unresolved {
        nodes.push(Component::External(unres));
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
    let name_to_add: Vec<String> = m.name.iter().filter(|s| *s != "root").cloned().collect();
    m_name.extend(name_to_add);

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
        for to in type_ref_targets(&ta.target) {
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
        add_annotation_edges(&ff.annotations, &ff_name, edges);
    }

    for fv in &m.free_variables {
        let mut fv_name = m_name.clone();
        fv_name.push(fv.name.clone());
        edges.push(Dependency {
            from: m_name.clone(),
            to: fv_name.clone(),
            kind: DependencyEdgeKind::ModuleContainment,
        });
        for to in type_ref_targets(&fv.ty) {
            edges.push(Dependency {
                from: fv_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::UsesFieldType,
            });
        }
        add_annotation_edges(&fv.annotations, &fv_name, edges);
    }

    for ib in &m.impl_blocks {
        for to in type_ref_targets(&ib.impl_for) {
            edges.push(Dependency {
                from: m_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::Implements,
            });
            if let Some(trait_ref) = &ib.implements_trait {
                for t in type_ref_targets(trait_ref) {
                    edges.push(Dependency {
                        from: to.clone(),
                        to: t.clone(),
                        kind: DependencyEdgeKind::Implements,
                    });
                }
            }
            for meth in &ib.methods {
                let mut m_name = to.clone();
                m_name.extend(meth.name.clone());
                edges.push(Dependency {
                    from: to.clone(),
                    to: m_name.clone(),
                    kind: DependencyEdgeKind::NestedIn,
                });
                add_function_edges(meth, &m_name, edges);
                add_annotation_edges(&meth.annotations, &m_name, edges);
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
            for ta in &ib.type_aliases {
                let mut ta_name = to.clone();
                ta_name.extend(ta.name.clone());
                for target_to in type_ref_targets(&ta.target) {
                    edges.push(Dependency {
                        from: ta_name.clone(),
                        to: target_to,
                        kind: DependencyEdgeKind::Aliases,
                    });
                }
            }
        }
    }

    for sub in &m.sub_modules {
        traverse_module_for_edges(sub, Some(&m_name), m_name.clone(), edges);
    }
}

fn traverse_structured_type_edges(
    st: &StructuredType,
    prefix: &QualifiedName,
    edges: &mut Vec<Dependency>,
) {
    let mut st_name = prefix.clone();
    st_name.extend(st.name.clone());

    add_super_edges(st, &st_name, edges);
    add_field_edges(st, &st_name, edges);
    add_annotation_edges(&st.annotations, &st_name, edges);

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
        add_annotation_edges(&m.annotations, &m_name, edges);
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

fn type_ref_targets(tr: &TypeRef) -> Vec<QualifiedName> {
    match tr {
        TypeRef::Resolved(to)
        | TypeRef::External(to)
        | TypeRef::Unresolved(to)
        | TypeRef::Failed(to) => {
            if to.is_empty() {
                vec![]
            } else {
                vec![to.clone()]
            }
        }
        TypeRef::Primitive(s) => {
            if s.is_empty() {
                vec![]
            } else {
                vec![vec![s.clone()]]
            }
        }
        TypeRef::Union(types) => types.iter().flat_map(type_ref_targets).collect(),
        _ => vec![],
    }
}

fn add_super_edges(st: &StructuredType, st_name: &QualifiedName, edges: &mut Vec<Dependency>) {
    for sup in &st.super_types {
        for to in type_ref_targets(sup) {
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
        let mut f_name = st_name.clone();
        f_name.push(f.name.clone());
        for to in type_ref_targets(&f.ty) {
            edges.push(Dependency {
                from: f_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::UsesFieldType,
            });
        }
        add_annotation_edges(&f.annotations, &f_name, edges);
    }
}

fn add_function_edges(ff: &Function, ff_name: &QualifiedName, edges: &mut Vec<Dependency>) {
    for p in &ff.signature.parameters {
        for to in type_ref_targets(&p.ty) {
            edges.push(Dependency {
                from: ff_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::UsesParamType,
            });
        }
    }
    for to in type_ref_targets(&ff.signature.return_type) {
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

fn add_block_edges(
    ff: &Function,
    ff_name: &QualifiedName,
    block: &Block,
    edges: &mut Vec<Dependency>,
) {
    // 1. Declarations (Local variables)
    for decl in &block.declarations {
        for to in type_ref_targets(&decl.ty) {
            edges.push(Dependency {
                from: ff_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::UsesLocalType,
            });
        }
    }

    // 2. Behavioral dependencies
    for call in &block.calls {
        for to in type_ref_targets(call) {
            edges.push(Dependency {
                from: ff_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::Calls,
            });
        }
    }
    for inst in &block.instantiates {
        for to in type_ref_targets(inst) {
            edges.push(Dependency {
                from: ff_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::Instantiates,
            });
        }
    }
    for acc in &block.accesses {
        for to in type_ref_targets(acc) {
            edges.push(Dependency {
                from: ff_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::AccessesField,
            });
        }
    }
    for cast in &block.type_casts {
        for to in type_ref_targets(cast) {
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

fn add_annotation_edges(
    annotations: &[TypeRef],
    source_name: &QualifiedName,
    edges: &mut Vec<Dependency>,
) {
    for anno in annotations {
        for to in type_ref_targets(anno) {
            edges.push(Dependency {
                from: source_name.clone(),
                to: to.clone(),
                kind: DependencyEdgeKind::AnnotatedWith,
            });
        }
    }
}

fn flatten_modules(modules: &[Module], prefix: QualifiedName) -> Vec<Component> {
    let mut flat = vec![];
    for m in modules {
        let mut m_name = prefix.clone();
        let name_to_add: Vec<String> = m.name.iter().filter(|s| *s != "root").cloned().collect();
        m_name.extend(name_to_add);

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
            for to in type_ref_targets(&ib.impl_for) {
                for mut m in ib.methods.clone() {
                    let mut m_name = to.clone();
                    m_name.extend(m.name.clone());
                    m.name = m_name;
                    flat.push(Component::Function(m));
                }
                for nested in &ib.nested_types {
                    flat.extend(flatten_structured_type(nested, &to));
                }
                for mut ta in ib.type_aliases.clone() {
                    let mut ta_name = to.clone();
                    ta_name.extend(ta.name.clone());
                    ta.name = ta_name;
                    flat.push(Component::TypeAlias(ta));
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
