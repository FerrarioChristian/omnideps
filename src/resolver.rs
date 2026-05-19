use crate::ir::*;
use std::collections::HashMap;

// ==================== CONTEXT PER RISOLUZIONE NOMI ====================
/// Contains the current prefix (namespace) and a flat symbol table for fast lookups.
#[derive(Debug)]
pub struct ResolutionContext {
    pub current_prefix: QualifiedName,
    pub symbol_table: HashMap<QualifiedName, Component>,
    pub imports: Vec<Import>,
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

        fn populate_st(
            st: &StructuredType,
            table: &mut HashMap<QualifiedName, Component>,
            prefix: &mut QualifiedName,
        ) {
            let mut name = prefix.clone();
            name.extend(st.name.clone());
            table.insert(name.clone(), Component::StructuredType(st.clone()));

            for nested in &st.nested_types {
                populate_st(nested, table, &mut name);
            }
        }

        for st in &m.structured_types {
            populate_st(st, table, prefix);
        }
        for f in &m.free_functions {
            let mut name = prefix.clone();
            name.extend(f.name.clone());
            table.insert(name, Component::Function(f.clone()));
        }
        for ib in &m.impl_blocks {
            // Un ImplBlock è zucchero sintattico e verrà appiattito nella struct target.
            // Poiché la Symbol Table viene costruita prima del flattening,
            // indicizziamo i tipi annidati dell'impl block simulando che si trovino
            // già sotto il percorso assoluto della struct target.
            if let TypeRef::Unresolved(target) = &ib.impl_for {
                let mut base_name = prefix.clone();
                base_name.extend(target.clone());
                for nested in &ib.nested_types {
                    populate_st(nested, table, &mut base_name);
                }
            } else if let TypeRef::Resolved(target) = &ib.impl_for {
                let mut base_name = target.clone();
                for nested in &ib.nested_types {
                    populate_st(nested, table, &mut base_name);
                }
            }
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
        imports: vec![],
    };
    modules
        .into_iter()
        .map(|m| resolve_module_in_context(&ctx, m))
        .collect()
}

enum ResolutionResult {
    Local(QualifiedName),
    External(QualifiedName),
}

// Regole di risoluzione (come formalizzate: assoluto -> relativo -> enclosing)
fn resolve_name_in_context(
    ctx: &ResolutionContext,
    name: &QualifiedName,
) -> Option<ResolutionResult> {
    // 0. Imports (Punti di salto)
    if let Some(first_part) = name.first() {
        for imp in &ctx.imports {
            if let Some(alias) = &imp.alias {
                if alias == first_part {
                    let mut candidate = imp.path.clone();
                    candidate.extend(name.iter().skip(1).cloned());
                    if ctx.symbol_table.contains_key(&candidate) {
                        return Some(ResolutionResult::Local(candidate));
                    } else {
                        return Some(ResolutionResult::External(candidate));
                    }
                }
            } else if !imp.is_wildcard {
                if let Some(last_part) = imp.path.last() {
                    if last_part == first_part {
                        let mut candidate = imp.path.clone();
                        candidate.extend(name.iter().skip(1).cloned());
                        if ctx.symbol_table.contains_key(&candidate) {
                            return Some(ResolutionResult::Local(candidate));
                        } else {
                            return Some(ResolutionResult::External(candidate));
                        }
                    }
                }
            }
        }
        // Wildcards (richiede validazione in symbol_table)
        for imp in &ctx.imports {
            if imp.is_wildcard {
                let mut candidate = imp.path.clone();
                candidate.extend(name.clone());
                if ctx.symbol_table.contains_key(&candidate) {
                    return Some(ResolutionResult::Local(candidate));
                }
            }
        }
    }

    // 1. Assoluto
    if ctx.symbol_table.contains_key(name) {
        return Some(ResolutionResult::Local(name.clone()));
    }
    // 2. Relativo al current_prefix
    let mut relative = ctx.current_prefix.clone();
    relative.extend(name.clone());
    if ctx.symbol_table.contains_key(&relative) {
        return Some(ResolutionResult::Local(relative));
    }
    // 3. Climb sugli enclosing scopes
    let mut prefix = ctx.current_prefix.clone();
    while !prefix.is_empty() {
        prefix.pop();
        let mut candidate = prefix.clone();
        candidate.extend(name.clone());
        if ctx.symbol_table.contains_key(&candidate) {
            return Some(ResolutionResult::Local(candidate));
        }
    }
    None
}

fn resolve_type_ref(ctx: &ResolutionContext, tr: TypeRef) -> TypeRef {
    match tr {
        TypeRef::Unresolved(name) => {
            if let Some(resolved) = resolve_name_in_context(ctx, &name) {
                match resolved {
                    ResolutionResult::Local(n) => TypeRef::Resolved(n),
                    ResolutionResult::External(n) => TypeRef::External(n),
                }
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
        imports: module.imports.clone(),
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
            if let Some(target_st) = module
                .structured_types
                .iter_mut()
                .find(|st| &st.name == target_name)
            {
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
        current_prefix: new_prefix.clone(),
        symbol_table: ctx.symbol_table.clone(),
        imports: ctx.imports.clone(),
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
        .signature
        .parameters
        .into_iter()
        .map(|p| Parameter {
            name: p.name,
            ty: resolve_type_ref(ctx, p.ty),
            is_variadic: p.is_variadic,
        })
        .collect();
    f.signature.return_type = resolve_type_ref(ctx, f.signature.return_type);

    f.calls = f
        .calls
        .into_iter()
        .map(|tr| resolve_type_ref(ctx, tr))
        .collect();
    f.instantiates = f
        .instantiates
        .into_iter()
        .map(|tr| resolve_type_ref(ctx, tr))
        .collect();

    f
}

fn resolve_impl_block(ctx: &ResolutionContext, mut i: ImplBlock) -> ImplBlock {
    let mut new_prefix = ctx.current_prefix.clone();
    new_prefix.extend(i.name.clone());
    i.name = new_prefix.clone();

    let new_ctx = ResolutionContext {
        current_prefix: new_prefix.clone(),
        symbol_table: ctx.symbol_table.clone(),
        imports: ctx.imports.clone(),
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
