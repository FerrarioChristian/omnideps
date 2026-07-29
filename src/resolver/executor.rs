use super::primitives::PrimitiveRegistry;
use super::scope::{ScopeId, ScopeTree, Symbol};
use crate::model::*;

pub struct ExecutorContext<'a> {
    pub tree: &'a ScopeTree,
    pub primitives: &'a PrimitiveRegistry,
    pub config: &'a crate::config::AnalyzerConfig,
}

/// Entry point for Phase 2b (Name Resolution).
/// Takes the extracted modules, builds the `ScopeTree`, and resolves all `TypeRef` queries.
pub fn execute_queries(
    modules: Vec<Module>,
    primitives: &PrimitiveRegistry,
    config: &crate::config::AnalyzerConfig,
) -> Vec<Module> {
    let tree = ScopeTree::build(&modules, config);

    let ctx = ExecutorContext {
        tree: &tree,
        primitives,
        config,
    };

    modules
        .into_iter()
        .map(|m| execute_module(&ctx, m, tree.root))
        .collect()
}

/// Helper function to find a direct child scope by name.
/// `is_module` differentiates between searching for a module or a structured type (class/struct).
fn find_child_scope(
    tree: &ScopeTree,
    parent: ScopeId,
    name: &str,
    is_module: bool,
) -> Option<ScopeId> {
    if let Some(sym) = tree.arena[parent].symbols.get(name) {
        match sym {
            Symbol::Module(id) if is_module => return Some(*id),
            Symbol::Type(id) if !is_module => return Some(*id),
            _ => {}
        }
    }
    None
}

/// Recursively resolves all declarations and queries inside a `Module`.
pub fn execute_module(ctx: &ExecutorContext, mut m: Module, parent_scope: ScopeId) -> Module {
    let module_name = m
        .name
        .last()
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    let scope_id = if module_name == "root" {
        parent_scope
    } else {
        find_child_scope(ctx.tree, parent_scope, &module_name, true).unwrap_or(parent_scope)
    };

    for fv in m.free_variables.iter_mut() {
        fv.ty = evaluate_typeref(ctx, fv.ty.clone(), scope_id, true);
    }

    for ta in m.type_aliases.iter_mut() {
        ta.target = evaluate_typeref(ctx, ta.target.clone(), scope_id, true);
    }

    // We do not mutate `imp.path` here anymore.
    // The graph generator should emit dependencies exactly as written in the source code (e.g. `transitive_main -> lib_b.TransitiveClass`).

    m.free_functions = m
        .free_functions
        .into_iter()
        .map(|ff| execute_function(ctx, ff, scope_id))
        .collect();
    m.structured_types = m
        .structured_types
        .into_iter()
        .map(|st| execute_structured_type(ctx, st, scope_id))
        .collect();
    m.impl_blocks = m
        .impl_blocks
        .into_iter()
        .map(|ib| execute_impl_block(ctx, ib, scope_id))
        .collect();
    m.sub_modules = m
        .sub_modules
        .into_iter()
        .map(|sub| execute_module(ctx, sub, scope_id))
        .collect();

    m
}

/// Recursively resolves all declarations and queries inside a `StructuredType` (Class, Struct, ecc.).
fn execute_structured_type(
    ctx: &ExecutorContext,
    mut st: StructuredType,
    parent_scope: ScopeId,
) -> StructuredType {
    let name = st.name.last().cloned().unwrap_or_default();
    let scope_id = find_child_scope(ctx.tree, parent_scope, &name, false).unwrap_or(parent_scope);

    st.super_types = st
        .super_types
        .into_iter()
        .map(|t| evaluate_typeref(ctx, t, scope_id, true))
        .collect();
    st.annotations = st
        .annotations
        .into_iter()
        .map(|a| evaluate_typeref(ctx, a, scope_id, false))
        .collect();
    st.fields = st
        .fields
        .into_iter()
        .map(|mut f| {
            f.ty = evaluate_typeref(ctx, f.ty, scope_id, true);
            f.annotations = f
                .annotations
                .into_iter()
                .map(|a| evaluate_typeref(ctx, a, scope_id, false))
                .collect();
            f
        })
        .collect();
    st.methods = st
        .methods
        .into_iter()
        .map(|m| execute_function(ctx, m, scope_id))
        .collect();
    st.nested_types = st
        .nested_types
        .into_iter()
        .map(|n| execute_structured_type(ctx, n, scope_id))
        .collect();
    st
}

/// Recursively resolves all declarations and queries inside an `ImplBlock`.
fn execute_impl_block(
    ctx: &ExecutorContext,
    mut ib: ImplBlock,
    parent_scope: ScopeId,
) -> ImplBlock {
    ib.impl_for = evaluate_typeref(ctx, ib.impl_for, parent_scope, true);
    ib.implements_trait = ib
        .implements_trait
        .map(|t| evaluate_typeref(ctx, t, parent_scope, true));

    let target_name = match &ib.impl_for {
        TypeRef::Resolved(qn) | TypeRef::External(qn) => qn.last().cloned().unwrap_or_default(),
        TypeRef::ResolutionQuery(q) => extract_base_name(q),
        _ => "".to_string(),
    };

    let scope_id =
        find_child_scope(ctx.tree, parent_scope, &target_name, false).unwrap_or(parent_scope);

    ib.methods = ib
        .methods
        .into_iter()
        .map(|m| execute_function(ctx, m, scope_id))
        .collect();
    ib.nested_types = ib
        .nested_types
        .into_iter()
        .map(|n| execute_structured_type(ctx, n, scope_id))
        .collect();
    ib.type_aliases = ib
        .type_aliases
        .into_iter()
        .map(|mut ta| {
            ta.target = evaluate_typeref(ctx, ta.target, scope_id, true);
            ta
        })
        .collect();
    ib
}

/// Resolves parameter types, return type, and body block of a `Function`.
fn execute_function(ctx: &ExecutorContext, mut f: Function, parent_scope: ScopeId) -> Function {
    let name = f.name.last().cloned().unwrap_or_default();

    let mut func_scope_id = parent_scope;
    for child in &ctx.tree.arena {
        if child.parent == Some(parent_scope) && child.name == name {
            func_scope_id = child.id;
            break;
        }
    }

    f.signature.parameters = f
        .signature
        .parameters
        .into_iter()
        .map(|mut p| {
            p.ty = evaluate_typeref(ctx, p.ty, func_scope_id, true);
            p
        })
        .collect();
    f.signature.return_type = evaluate_typeref(ctx, f.signature.return_type, func_scope_id, true);
    f.annotations = f
        .annotations
        .into_iter()
        .map(|a| evaluate_typeref(ctx, a, func_scope_id, false))
        .collect();
    f.body = f.body.map(|b| execute_block(ctx, b, func_scope_id, 0));

    f
}

/// Resolves variable declarations, instantiations, function calls, and field accesses within a `Block`.
fn execute_block(
    ctx: &ExecutorContext,
    mut b: Block,
    parent_scope: ScopeId,
    index: usize,
) -> Block {
    let block_name = format!("block_{}", index);
    let mut block_scope_id = parent_scope;
    for child in &ctx.tree.arena {
        if child.parent == Some(parent_scope) && child.name == block_name {
            block_scope_id = child.id;
            break;
        }
    }

    b.declarations = b
        .declarations
        .into_iter()
        .map(|mut d| {
            d.ty = evaluate_typeref(ctx, d.ty, block_scope_id, true);
            d.annotations = d
                .annotations
                .into_iter()
                .map(|a| evaluate_typeref(ctx, a, block_scope_id, false))
                .collect();
            d
        })
        .collect();
    b.calls = b
        .calls
        .into_iter()
        .map(|c| {
            let tr = evaluate_typeref(ctx, c, block_scope_id, false);
            redirect_to_constructor(ctx, tr)
        })
        .collect();
    b.instantiates = b
        .instantiates
        .into_iter()
        .map(|i| {
            let tr = evaluate_typeref(ctx, i, block_scope_id, false);
            redirect_to_constructor(ctx, tr)
        })
        .collect();
    b.accesses = b
        .accesses
        .into_iter()
        .map(|a| evaluate_typeref(ctx, a, block_scope_id, false))
        .collect();
    b.type_casts = b
        .type_casts
        .into_iter()
        .map(|c| evaluate_typeref(ctx, c, block_scope_id, false))
        .collect();

    let sub_blocks: Vec<Block> = b
        .sub_blocks
        .into_iter()
        .enumerate()
        .map(|(i, sub)| execute_block(ctx, sub, block_scope_id, i))
        .collect();
    b.sub_blocks = sub_blocks;
    b
}

fn redirect_to_constructor(ctx: &ExecutorContext, tr: TypeRef) -> TypeRef {
    if let TypeRef::Resolved(ref path) = tr {
        if let Some(scope_id) = find_scope_for_type(ctx.tree, &tr) {
            // Check if it's a class/type scope by verifying if there are any constructors.
            let ctor_names = ["__init__", "constructor", path.last().unwrap().as_str()];
            for cname in ctor_names {
                if ctx.tree.arena[scope_id].symbols.contains_key(cname) {
                    let mut new_path = path.clone();
                    new_path.push(cname.to_string());
                    return TypeRef::Resolved(new_path);
                }
            }
        }
    }
    tr
}

/// Core function to resolve a `TypeRef`.
/// If the `TypeRef` is a `ResolutionQuery` or `Unresolved`, it tries to evaluate it dynamically against the ScopeTree.
pub fn evaluate_typeref(
    ctx: &ExecutorContext,
    tr: TypeRef,
    scope_id: ScopeId,
    resolve_type: bool,
) -> TypeRef {
    match tr {
        TypeRef::ResolutionQuery(query) => {
            let mut visited = std::collections::HashSet::new();
            if let Some(resolved) =
                evaluate_query(ctx, &query, scope_id, resolve_type, &mut visited)
            {
                println!("RESOLUTION QUERY {:?} EVALUATED TO: {:?}", query, resolved);
                resolved
            } else {
                println!("RESOLUTION QUERY {:?} FAILED", query);
                TypeRef::Failed(vec![extract_base_name(&query)])
            }
        }
        TypeRef::Unresolved(ref qn) => {
            if qn.is_empty() {
                return tr;
            }
            let mut query = Query::Find(qn[0].clone());
            for part in &qn[1..] {
                query = Query::Extract(Box::new(query), part.clone());
            }

            let mut visited = std::collections::HashSet::new();
            if let Some(resolved) =
                evaluate_query(ctx, &query, scope_id, resolve_type, &mut visited)
            {
                if qn[0] == "StructA" {
                    println!("EVALUATED Unresolved StructA to: {:?}", resolved);
                }
                resolved
            } else {
                if qn[0] == "StructA" {
                    println!("EVALUATED Unresolved StructA to NONE");
                }
                TypeRef::Unresolved(qn.clone())
            }
        }
        TypeRef::Union(variants) => {
            let evaluated = variants
                .into_iter()
                .map(|v| evaluate_typeref(ctx, v, scope_id, resolve_type))
                .collect::<Vec<_>>();
            println!("UNION EVALUATED TO: {:?}", evaluated);
            TypeRef::Union(evaluated)
        }
        _ => tr,
    }
}

/// Extracts the human-readable string representation of a `Query`.
pub fn extract_base_name(query: &Query) -> String {
    match query {
        Query::Find(name) => name.clone(),
        Query::Extract(parent, member) => format!("{}::{}", extract_base_name(parent), member),
        Query::Call(parent) => format!("{}()", extract_base_name(parent)),
    }
}

/// Reconstructs the fully qualified path from the root down to a specific `ScopeId`.
fn build_path_from_scope(tree: &ScopeTree, scope_id: ScopeId) -> QualifiedName {
    let mut path = vec![];
    let mut curr = Some(scope_id);
    while let Some(id) = curr {
        if !tree.arena[id].name.starts_with("block") && tree.arena[id].name != "root" {
            path.push(tree.arena[id].name.clone());
        }
        curr = tree.arena[id].parent;
    }
    path.reverse();
    path
}

/// Attempts to find a symbol strictly within the given scope or its inherited super types.
/// Does not perform lexical climbing.
pub fn find_symbol_in_scope_and_supers(
    ctx: &ExecutorContext,
    scope_id: ScopeId,
    name: &str,
    resolve_type: bool,
    visited: &mut std::collections::HashSet<String>,
) -> Option<TypeRef> {
    if let Some(sym) = ctx.tree.arena[scope_id].symbols.get(name) {
        return Some(symbol_to_typeref(
            ctx,
            scope_id,
            sym,
            name,
            resolve_type,
            visited,
        ));
    }

    for st in &ctx.tree.arena[scope_id].super_types {
        // Resolve super type first
        let resolved_st = match st {
            TypeRef::ResolutionQuery(q) => {
                evaluate_query(ctx, q, scope_id, true, visited).unwrap_or_else(|| st.clone())
            }
            TypeRef::Unresolved(qn) => {
                let query = Query::Find(qn.last().cloned().unwrap_or_default());
                evaluate_query(ctx, &query, scope_id, true, visited).unwrap_or_else(|| st.clone())
            }
            _ => st.clone(),
        };

        if let Some(super_scope) = find_scope_for_type(ctx.tree, &resolved_st) {
            // Prevent infinite loops if circular inheritance
            if scope_id != super_scope
                && let Some(res) =
                    find_symbol_in_scope_and_supers(ctx, super_scope, name, resolve_type, visited)
            {
                return Some(res);
            }
        }
    }

    None
}

/// Evaluates a `Query` (Find, Extract, or Call) dynamically against the ScopeTree.
/// Employs lexical climbing for `Find`, and hierarchical resolution for `Extract`.
fn evaluate_query(
    ctx: &ExecutorContext,
    query: &Query,
    scope_id: ScopeId,
    resolve_type: bool,
    visited: &mut std::collections::HashSet<String>,
) -> Option<TypeRef> {
    let q_str = extract_base_name(query);
    if !visited.insert(q_str.clone()) {
        return None;
    }

    let result = match query {
        Query::Find(name) => evaluate_query_find(ctx, name, scope_id, resolve_type, visited),
        Query::Extract(parent_q, member) => {
            evaluate_query_extract(ctx, parent_q, member, scope_id, resolve_type, visited)
        }
        Query::Call(target_q) => evaluate_query(ctx, target_q, scope_id, true, visited),
    };

    visited.remove(&q_str);
    result
}

/// Helper function to resolve the "super" or "super()" keyword dynamically.
/// It climbs the scope tree to find the nearest enclosing class/struct and returns its first base type.
fn resolve_super_keyword(
    ctx: &ExecutorContext,
    scope_id: ScopeId,
    resolve_type: bool,
    visited: &mut std::collections::HashSet<String>,
) -> Option<TypeRef> {
    let mut curr = Some(scope_id);
    while let Some(id) = curr {
        let scope = &ctx.tree.arena[id];
        if !scope.super_types.is_empty() {
            let st = &scope.super_types[0];
            return match st {
                TypeRef::ResolutionQuery(q) => {
                    evaluate_query(ctx, q, id, resolve_type, visited).or(Some(st.clone()))
                }
                _ => Some(st.clone()),
            };
        }
        curr = scope.parent;
    }
    None
}

/// Helper function to evaluate `Query::Find`. Performs lexical climbing up the scope tree.
fn evaluate_query_find(
    ctx: &ExecutorContext,
    name: &str,
    scope_id: ScopeId,
    resolve_type: bool,
    visited: &mut std::collections::HashSet<String>,
) -> Option<TypeRef> {
    if name == "super()" || name == "super" {
        return resolve_super_keyword(ctx, scope_id, resolve_type, visited);
    }

    let mut curr = Some(scope_id);
    while let Some(id) = curr {
        if let Some(res) = find_symbol_in_scope_and_supers(ctx, id, name, resolve_type, visited) {
            return Some(res);
        }

        for imp in &ctx.tree.arena[id].imports {
            if let Some(last) = imp.path.last() {
                if last == name {
                    if let Some(resolved) = find_global(ctx, &imp.path) {
                        return Some(resolved);
                    } else {
                        // If not found in the tree, it might be an external library
                        return Some(TypeRef::External(imp.path.clone()));
                    }
                } else if last == "*" || imp.is_wildcard {
                    let mut specific_path = imp.path.clone();
                    if last == "*" {
                        specific_path.pop();
                    }
                    specific_path.push(name.to_string());

                    if let Some(resolved) = find_global(ctx, &specific_path) {
                        return Some(resolved);
                    }

                    // Check if the base module exists. If not, we assume the symbol comes from it.
                    let mut base_path = imp.path.clone();
                    if last == "*" {
                        base_path.pop();
                    }
                    if find_global(ctx, &base_path).is_none() {
                        return Some(TypeRef::External(specific_path));
                    }
                }
            }
        }

        curr = ctx.tree.arena[id].parent;
    }

    find_global(ctx, &[name.to_string()])
}

/// Helper function to evaluate `Query::Extract`. Resolves the parent and extracts the member.
fn evaluate_query_extract(
    ctx: &ExecutorContext,
    parent_q: &Query,
    member: &str,
    scope_id: ScopeId,
    resolve_type: bool,
    visited: &mut std::collections::HashSet<String>,
) -> Option<TypeRef> {
    let parent_ty = evaluate_query(ctx, parent_q, scope_id, true, visited)?;

    // First resolve the parent type if it's Unresolved, to ensure find_scope_for_type works
    let mut resolved_parent_ty = parent_ty.clone();
    if let TypeRef::Unresolved(_) | TypeRef::ResolutionQuery(_) = resolved_parent_ty {
        resolved_parent_ty = evaluate_typeref(ctx, resolved_parent_ty, scope_id, true);
    }

    if let Some(target_scope) = find_scope_for_type(ctx.tree, &resolved_parent_ty) {
        if let Some(res) =
            find_symbol_in_scope_and_supers(ctx, target_scope, member, resolve_type, visited)
        {
            return Some(res);
        }

        // Transitive imports check
        if let Some(res) = resolve_via_transitive_imports(ctx, target_scope, member) {
            return Some(res);
        }
    }
    
    // Fallback: append member to the resolved parent type
    match resolved_parent_ty {
        TypeRef::Resolved(mut path) => {
            path.push(member.to_string());
            Some(TypeRef::Resolved(path))
        }
        TypeRef::External(mut path) => {
            path.push(member.to_string());
            Some(TypeRef::External(path))
        }
        TypeRef::Unresolved(mut path) => {
            path.push(member.to_string());
            Some(TypeRef::Unresolved(path))
        }
        _ => None,
    }
}

/// Converts a raw `Symbol` found in the tree into a properly formatted `TypeRef`.
fn symbol_to_typeref(
    ctx: &ExecutorContext,
    scope_id: ScopeId,
    sym: &Symbol,
    name: &str,
    resolve_type: bool,
    visited: &mut std::collections::HashSet<String>,
) -> TypeRef {
    match sym {
        Symbol::Module(id) | Symbol::Type(id) => {
            let path = build_path_from_scope(ctx.tree, *id);
            TypeRef::Resolved(path)
        }
        Symbol::Value(ty) | Symbol::TypeAlias(ty) => {
            if resolve_type {
                match ty {
                    TypeRef::ResolutionQuery(q) => evaluate_query(ctx, q, scope_id, true, visited)
                        .unwrap_or_else(|| ty.clone()),
                    TypeRef::Unresolved(_) => evaluate_typeref(ctx, ty.clone(), scope_id, true),
                    _ => ty.clone(),
                }
            } else {
                let mut path = build_path_from_scope(ctx.tree, scope_id);
                path.push(name.to_string());
                TypeRef::Resolved(path)
            }
        }
    }
}

/// Given a resolved `TypeRef`, attempts to locate its corresponding `ScopeId` in the ScopeTree.
pub fn find_scope_for_type(tree: &ScopeTree, ty: &TypeRef) -> Option<ScopeId> {
    match ty {
        TypeRef::Resolved(qn) | TypeRef::External(qn) => {
            let mut curr = tree.root;
            for (i, part) in qn.iter().enumerate() {
                if part == "root" && i == 0 {
                    continue;
                }
                let sym = tree.arena[curr].symbols.get(part)?;
                match sym {
                    Symbol::Module(id) | Symbol::Type(id) => curr = *id,
                    _ => return None,
                }
            }
            Some(curr)
        }
        _ => None,
    }
}

/// Resolves a fully qualified path starting from the global root.
/// Also handles transitive imports resolution natively.
pub fn find_global(ctx: &ExecutorContext, path: &[String]) -> Option<TypeRef> {
    let mut curr = ctx.tree.root;

    if path.len() == 1 && ctx.primitives.is_primitive(&path[0]) {
        return Some(TypeRef::Primitive(path[0].clone()));
    }

    for (i, part) in path.iter().enumerate() {
        if (part == "root" || part == "crate") && i == 0 {
            continue;
        }
        if let Some(sym) = ctx.tree.arena[curr].symbols.get(part) {
            match sym {
                Symbol::Module(id) | Symbol::Type(id) => curr = *id,
                Symbol::Value(ty) | Symbol::TypeAlias(ty) => {
                    if i == path.len() - 1 {
                        return Some(ty.clone());
                    } else {
                        return None;
                    }
                }
            }
        } else {
            // Check transitive imports
            if let Some(resolved) = resolve_via_transitive_imports(ctx, curr, part) {
                if i == path.len() - 1 {
                    return Some(resolved);
                } else {
                    if let Some(next_scope) = find_scope_for_type(ctx.tree, &resolved) {
                        curr = next_scope;
                        continue;
                    }
                }
            }

            return None;
        }
    }

    let final_path = build_path_from_scope(ctx.tree, curr);
    Some(TypeRef::Resolved(final_path))
}

/// Checks if the target scope is a module with `transitive_imports` enabled,
/// and attempts to resolve the member through its exported imports.
fn resolve_via_transitive_imports(
    ctx: &ExecutorContext,
    scope_id: ScopeId,
    member: &str,
) -> Option<TypeRef> {
    let node = &ctx.tree.arena[scope_id];
    if node.is_module {
        let lang = node.language.as_deref().unwrap_or("root");
        if ctx.config.get_for(lang).transitive_imports {
            for imp in &node.imports {
                if let Some(last) = imp.path.last()
                    && (last == member || last == "*")
                {
                    if let Some(resolved) = find_global(ctx, &imp.path) {
                        return Some(resolved);
                    } else {
                        return Some(TypeRef::External(imp.path.clone()));
                    }
                }
            }
        }
    }
    None
}
