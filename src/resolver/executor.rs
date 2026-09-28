use super::primitives::PrimitiveRegistry;
use super::scope::{ScopeId, ScopeTree, Symbol};
use crate::model::*;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct ExecutorContext<'a> {
    pub tree: &'a ScopeTree,
    pub primitives: &'a PrimitiveRegistry,
    pub config: &'a crate::config::AnalyzerConfig,
    pub resolved_super_scopes: RefCell<HashMap<ScopeId, Vec<ScopeId>>>,
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
        resolved_super_scopes: RefCell::new(HashMap::new()),
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
    st.type_parameters = st
        .type_parameters
        .into_iter()
        .map(|mut tp| {
            tp.bounds = tp
                .bounds
                .into_iter()
                .map(|b| evaluate_typeref(ctx, b, scope_id, true))
                .collect();
            tp
        })
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
        TypeRef::Failed(qn) => qn.last().cloned().unwrap_or_default(),
        TypeRef::EvaluatedAccess(_, inner) => {
            if let TypeRef::Resolved(qn) | TypeRef::External(qn) = &**inner {
                qn.last().cloned().unwrap_or_default()
            } else {
                "".to_string()
            }
        }
        _ => "".to_string(),
    };

    let scope_id = find_child_scope(ctx.tree, parent_scope, &target_name, false)
        .or_else(|| {
            ctx.tree
                .type_scopes_by_name
                .get(&target_name)
                .and_then(|ids| ids.first().copied())
        })
        .unwrap_or(parent_scope);

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

    let func_scope_id = ctx.tree.arena[parent_scope]
        .children_by_name
        .get(&name)
        .and_then(|ids| ids.first().copied())
        .unwrap_or(parent_scope);

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
    f.type_parameters = f
        .type_parameters
        .into_iter()
        .map(|mut tp| {
            tp.bounds = tp
                .bounds
                .into_iter()
                .map(|b| evaluate_typeref(ctx, b, func_scope_id, true))
                .collect();
            tp
        })
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
    let block_scope_id = ctx.tree.arena[parent_scope]
        .children_by_name
        .get(&block_name)
        .and_then(|ids| ids.first().copied())
        .unwrap_or(parent_scope);

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

    let lang = get_scope_language(ctx.tree, block_scope_id).unwrap_or("");
    let lang_config = ctx.config.get_for(lang);

    let mut resolved_calls = Vec::with_capacity(b.calls.len());
    for c in b.calls {
        let tr = evaluate_typeref(ctx, c, block_scope_id, false);
        let redirected = redirect_to_constructor(ctx, tr);
        if !lang_config.callable_types && is_type_or_alias(ctx, &redirected) {
            b.type_casts.push(redirected);
        } else {
            resolved_calls.push(redirected);
        }
    }
    b.calls = resolved_calls;

    let sub_blocks: Vec<Block> = b
        .sub_blocks
        .into_iter()
        .enumerate()
        .map(|(i, sub)| execute_block(ctx, sub, block_scope_id, i))
        .collect();
    b.sub_blocks = sub_blocks;
    b
}

fn get_scope_language(tree: &ScopeTree, mut scope_id: ScopeId) -> Option<&str> {
    loop {
        let s = &tree.arena[scope_id];
        if let Some(l) = &s.language {
            return Some(l.as_str());
        }
        match s.parent {
            Some(p) => scope_id = p,
            None => return None,
        }
    }
}

fn find_symbol_by_path<'a>(tree: &'a ScopeTree, path: &[String]) -> Option<&'a Symbol> {
    let mut curr = tree.root;
    for (i, part) in path.iter().enumerate() {
        if (part == "root" || part == "crate") && i == 0 {
            continue;
        }
        if i == path.len() - 1 {
            return tree.arena[curr].symbols.get(part);
        }
        if let Some(Symbol::Module(id) | Symbol::Type(id)) = tree.arena[curr].symbols.get(part) {
            curr = *id;
            continue;
        }
        if let Some(ids) = tree.arena[curr].children_by_name.get(part) {
            if let Some(&child_id) = ids.first() {
                curr = child_id;
                continue;
            }
        }
        return None;
    }
    None
}

fn is_type_or_alias(ctx: &ExecutorContext, tr: &TypeRef) -> bool {
    match tr {
        TypeRef::Primitive(_) => true,
        TypeRef::Resolved(path) | TypeRef::External(path) => {
            if path.len() == 1 && ctx.primitives.is_primitive(&path[0]) {
                return true;
            }
            if ctx.primitives.is_primitive(&path.join("::"))
                || ctx.primitives.is_primitive(&path.join("."))
            {
                return true;
            }
            if let Some(sym) = find_symbol_by_path(ctx.tree, path) {
                return matches!(sym, Symbol::Type(_) | Symbol::TypeAlias(_));
            }
            if let Some(scope_id) = find_scope_for_type(ctx.tree, tr) {
                return !ctx.tree.arena[scope_id].is_module;
            }
            false
        }
        TypeRef::EvaluatedAccess(_, inner) => is_type_or_alias(ctx, inner),
        TypeRef::Generic { base, .. } => is_type_or_alias(ctx, base),
        _ => false,
    }
}

fn redirect_to_constructor(ctx: &ExecutorContext, tr: TypeRef) -> TypeRef {
    match &tr {
        TypeRef::Resolved(path) => {
            if let Some(scope_id) = find_scope_for_type(ctx.tree, &tr) {
                if let Some(last_name) = path.last() {
                    let ctor_names = ["__init__", "constructor", last_name.as_str()];
                    for cname in ctor_names {
                        if ctx.tree.arena[scope_id].symbols.contains_key(cname) {
                            let mut new_path = path.clone();
                            new_path.push(cname.to_string());
                            return TypeRef::Resolved(new_path);
                        }
                    }
                }
            }
        }
        TypeRef::EvaluatedAccess(base, inner) => {
            let inner_redirected = redirect_to_constructor(ctx, inner.as_ref().clone());
            if inner_redirected != **inner {
                return TypeRef::EvaluatedAccess(base.clone(), Box::new(inner_redirected));
            }
        }
        _ => {}
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
    let mut visited = std::collections::HashSet::new();
    evaluate_typeref_inner(ctx, tr, scope_id, resolve_type, &mut visited)
}

pub fn evaluate_typeref_inner(
    ctx: &ExecutorContext,
    tr: TypeRef,
    scope_id: ScopeId,
    resolve_type: bool,
    visited: &mut std::collections::HashSet<String>,
) -> TypeRef {
    match tr {
        TypeRef::ResolutionQuery(query) => {
            if let Some(resolved) = evaluate_query(ctx, &query, scope_id, resolve_type, visited) {
                log::trace!("RESOLUTION QUERY {:?} EVALUATED TO: {:?}", query, resolved);
                resolved
            } else {
                log::trace!("RESOLUTION QUERY {:?} FAILED", query);
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

            if let Some(resolved) = evaluate_query(ctx, &query, scope_id, resolve_type, visited) {
                if qn[0] == "StructA" {
                    log::trace!("EVALUATED Unresolved StructA to: {:?}", resolved);
                }
                resolved
            } else {
                if qn[0] == "StructA" {
                    log::trace!("EVALUATED Unresolved StructA to NONE");
                }
                TypeRef::Unresolved(qn.clone())
            }
        }
        TypeRef::Union(variants) => {
            let evaluated = variants
                .into_iter()
                .map(|v| evaluate_typeref_inner(ctx, v, scope_id, resolve_type, visited))
                .collect::<Vec<_>>();
            log::trace!("UNION EVALUATED TO: {:?}", evaluated);
            TypeRef::Union(evaluated)
        }
        TypeRef::Generic { base, args } => {
            let base_eval = evaluate_typeref_inner(ctx, *base, scope_id, resolve_type, visited);
            let args_eval = args
                .into_iter()
                .map(|a| evaluate_typeref_inner(ctx, a, scope_id, resolve_type, visited))
                .collect();
            TypeRef::Generic {
                base: Box::new(base_eval),
                args: args_eval,
            }
        }
        TypeRef::TypeVar { name, bounds } => {
            let bounds_eval = bounds
                .into_iter()
                .map(|b| evaluate_typeref_inner(ctx, b, scope_id, resolve_type, visited))
                .collect();
            TypeRef::TypeVar {
                name,
                bounds: bounds_eval,
            }
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

/// Extracts the qualified path components of a `Query`, if it represents a static path.
pub fn query_to_path(query: &Query) -> Option<Vec<String>> {
    match query {
        Query::Find(name) => Some(vec![name.clone()]),
        Query::Extract(parent, member) => {
            let mut p = query_to_path(parent)?;
            p.push(member.clone());
            Some(p)
        }
        Query::Call(parent) => query_to_path(parent),
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
    let mut visited_scopes = std::collections::HashSet::new();
    find_symbol_in_scope_and_supers_internal(
        ctx,
        scope_id,
        name,
        resolve_type,
        visited,
        &mut visited_scopes,
    )
}

/// Helper function to resolve and cache the `ScopeId`s of a scope's super types/interfaces.
/// Resolution is performed strictly within the enclosing `parent_scope` (never within `scope_id` itself),
/// preventing combinatorial / exponential recursion when a class implements many interfaces.
fn get_or_resolve_super_scopes(ctx: &ExecutorContext, scope_id: ScopeId) -> Vec<ScopeId> {
    if let Some(supers) = ctx.resolved_super_scopes.borrow().get(&scope_id) {
        return supers.clone();
    }

    // Insert an empty entry first to break any circular inheritance cycles during resolution
    ctx.resolved_super_scopes
        .borrow_mut()
        .insert(scope_id, Vec::new());

    let mut resolved_scopes = Vec::new();
    let mut visited = std::collections::HashSet::new();

    for st in &ctx.tree.arena[scope_id].super_types {
        let resolved_st = match st {
            TypeRef::ResolutionQuery(q) => {
                evaluate_query(ctx, q, scope_id, true, &mut visited).unwrap_or_else(|| st.clone())
            }
            TypeRef::Unresolved(qn) => {
                let query = Query::Find(qn.last().cloned().unwrap_or_default());
                evaluate_query(ctx, &query, scope_id, true, &mut visited)
                    .unwrap_or_else(|| st.clone())
            }
            TypeRef::Generic { base, args } => {
                let resolved_base = match base.as_ref() {
                    TypeRef::ResolutionQuery(q) => {
                        evaluate_query(ctx, q, scope_id, true, &mut visited)
                            .unwrap_or_else(|| *base.clone())
                    }
                    TypeRef::Unresolved(qn) => {
                        let query = Query::Find(qn.last().cloned().unwrap_or_default());
                        evaluate_query(ctx, &query, scope_id, true, &mut visited)
                            .unwrap_or_else(|| *base.clone())
                    }
                    _ => *base.clone(),
                };
                TypeRef::Generic {
                    base: Box::new(resolved_base),
                    args: args.clone(),
                }
            }
            _ => st.clone(),
        };

        if let Some(super_scope) = find_scope_for_type(ctx.tree, &resolved_st)
            && super_scope != scope_id
            && !resolved_scopes.contains(&super_scope)
        {
            resolved_scopes.push(super_scope);
        }
    }

    ctx.resolved_super_scopes
        .borrow_mut()
        .insert(scope_id, resolved_scopes.clone());
    resolved_scopes
}

fn find_symbol_in_scope_and_supers_internal(
    ctx: &ExecutorContext,
    scope_id: ScopeId,
    name: &str,
    resolve_type: bool,
    visited: &mut std::collections::HashSet<String>,
    visited_scopes: &mut std::collections::HashSet<ScopeId>,
) -> Option<TypeRef> {
    if !visited_scopes.insert(scope_id) {
        return None;
    }

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

    let super_scopes = get_or_resolve_super_scopes(ctx, scope_id);
    for super_scope in super_scopes {
        if let Some(res) = find_symbol_in_scope_and_supers_internal(
            ctx,
            super_scope,
            name,
            resolve_type,
            visited,
            visited_scopes,
        ) {
            return Some(res);
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
    let q_str = format!("{}:{}", scope_id, extract_base_name(query));
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
    let super_key = format!("super:{}", scope_id);
    if !visited.insert(super_key.clone()) {
        return None;
    }

    let result = (|| {
        let mut curr = Some(scope_id);
        while let Some(id) = curr {
            let scope = &ctx.tree.arena[id];
            if !scope.super_types.is_empty() {
                let st = &scope.super_types[0];
                return match st {
                    TypeRef::ResolutionQuery(q) => {
                        evaluate_query(ctx, q, id, resolve_type, visited).or(Some(st.clone()))
                    }
                    TypeRef::Unresolved(qn) => {
                        let query = Query::Find(qn.last().cloned().unwrap_or_default());
                        evaluate_query(ctx, &query, id, resolve_type, visited).or(Some(st.clone()))
                    }
                    _ => Some(st.clone()),
                };
            }
            curr = scope.parent;
        }

        // If no super_types were found (e.g. Rust module `super::`),
        // resolve to the parent module in the module hierarchy
        let mut mod_curr = Some(scope_id);
        while let Some(id) = mod_curr {
            let scope = &ctx.tree.arena[id];
            if scope.is_module {
                if let Some(parent_id) = scope.parent {
                    let parent_path = build_path_from_scope(ctx.tree, parent_id);
                    return Some(TypeRef::Resolved(parent_path));
                }
                break;
            }
            mod_curr = scope.parent;
        }

        None
    })();

    visited.remove(&super_key);
    result
}

/// Helper function to resolve the "Self" keyword dynamically.
/// It climbs the scope tree to find the nearest enclosing structured type (class, struct, etc.)
/// and returns a resolved reference to it.
fn resolve_self_keyword(ctx: &ExecutorContext, scope_id: ScopeId) -> Option<TypeRef> {
    let mut curr = Some(scope_id);
    while let Some(id) = curr {
        let scope = &ctx.tree.arena[id];

        // Check if this scope is a StructuredType by looking at its parent's symbols
        if let Some(parent_id) = scope.parent {
            let parent_scope = &ctx.tree.arena[parent_id];
            for symbol in parent_scope.symbols.values() {
                if let crate::resolver::scope::Symbol::Type(type_id) = symbol
                    && *type_id == id
                {
                    let path = build_path_from_scope(ctx.tree, id);
                    return Some(TypeRef::Resolved(path));
                }
            }
        }
        curr = scope.parent;
    }
    None
}

/// Resolves an import path taking into account the declaring scope (for relative imports).
/// Handles `super::`, `self::`, sibling submodules, and falls back to global lookup.
fn resolve_import_path(
    ctx: &ExecutorContext,
    scope_id: ScopeId,
    path: &[String],
    visited: &mut std::collections::HashSet<String>,
) -> Option<TypeRef> {
    if path.is_empty() {
        return None;
    }

    let cycle_key = format!("imp_path:{}:{}", scope_id, path.join("::"));
    if !visited.insert(cycle_key.clone()) {
        return None;
    }

    let result = (|| {
        // 1. If path starts with "super", climb up the module hierarchy
        if path[0] == "super" {
            let mut curr_scope = scope_id;
            let mut skip = 0;
            for part in path {
                if part == "super" {
                    let mut climbed = false;
                    let mut check = Some(curr_scope);
                    while let Some(sid) = check {
                        let sc = &ctx.tree.arena[sid];
                        if sc.is_module {
                            if let Some(parent) = sc.parent {
                                curr_scope = parent;
                                climbed = true;
                                skip += 1;
                            }
                            break;
                        }
                        check = sc.parent;
                    }
                    if !climbed {
                        break;
                    }
                } else {
                    break;
                }
            }
            if skip > 0 {
                let mut full_path = build_path_from_scope(ctx.tree, curr_scope);
                full_path.extend_from_slice(&path[skip..]);
                return find_global_internal(ctx, &full_path, visited);
            }
        }

        // 2. If path starts with "self", resolve relative to current enclosing module
        if path[0] == "self" {
            let mut check = Some(scope_id);
            while let Some(sid) = check {
                let sc = &ctx.tree.arena[sid];
                if sc.is_module {
                    let mut full_path = build_path_from_scope(ctx.tree, sid);
                    full_path.extend_from_slice(&path[1..]);
                    return find_global_internal(ctx, &full_path, visited);
                }
                check = sc.parent;
            }
        }

        // 3. Check if path[0] is a symbol (e.g. child submodule) in the current enclosing module
        let mut check = Some(scope_id);
        while let Some(sid) = check {
            let sc = &ctx.tree.arena[sid];
            if sc.is_module {
                if sc.symbols.contains_key(&path[0]) {
                    let mut full_path = build_path_from_scope(ctx.tree, sid);
                    full_path.extend_from_slice(path);
                    if let Some(res) = find_global_internal(ctx, &full_path, visited) {
                        return Some(res);
                    }
                }
                break;
            }
            check = sc.parent;
        }

        // 4. Fallback: try global resolution from root (e.g. crate::... or root module)
        find_global_internal(ctx, path, visited)
    })();

    visited.remove(&cycle_key);
    result
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
    if name == "Self"
        && let Some(resolved_self) = resolve_self_keyword(ctx, scope_id)
    {
        return Some(resolved_self);
    }

    let mut curr = Some(scope_id);
    while let Some(id) = curr {
        if let Some(res) = find_symbol_in_scope_and_supers(ctx, id, name, resolve_type, visited) {
            return Some(res);
        }

        for imp in &ctx.tree.arena[id].imports {
            if let Some(last) = imp.path.last() {
                if last == name {
                    if let Some(resolved) = resolve_import_path(ctx, id, &imp.path, visited) {
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

                    if let Some(resolved) = resolve_import_path(ctx, id, &specific_path, visited) {
                        return Some(resolved);
                    }

                    // Check if the base module exists. If not, we assume the symbol comes from it.
                    let mut base_path = imp.path.clone();
                    if last == "*" {
                        base_path.pop();
                    }
                    if resolve_import_path(ctx, id, &base_path, visited).is_none() {
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
    let parent_ty = match evaluate_query(ctx, parent_q, scope_id, true, visited) {
        Some(ty) => ty,
        None => {
            if let Some(mut full_path) = query_to_path(parent_q) {
                full_path.push(member.to_string());
                let joined_colon = full_path.join("::");
                if ctx.primitives.is_primitive(&joined_colon) {
                    return Some(TypeRef::Primitive(joined_colon));
                }
                let joined_dot = full_path.join(".");
                if ctx.primitives.is_primitive(&joined_dot) {
                    return Some(TypeRef::Primitive(joined_dot));
                }
                if let Some(res) = find_global_internal(ctx, &full_path, visited) {
                    return Some(res);
                }
            }
            return None;
        }
    };
    let mut resolved_parent_ty = parent_ty.clone();

    // If it's an EvaluatedAccess, unwrap the resolved type for further lookup,
    // but keep the base access to reconstruct the full EvaluatedAccess later.
    let base_access = if let TypeRef::EvaluatedAccess(base, ty) = &parent_ty {
        resolved_parent_ty = ty.as_ref().clone();
        Some(base.clone())
    } else {
        None
    };

    // If it's Unresolved, to ensure find_scope_for_type works
    if matches!(
        resolved_parent_ty,
        TypeRef::Unresolved(_) | TypeRef::ResolutionQuery(_) | TypeRef::Generic { .. }
    ) {
        resolved_parent_ty =
            evaluate_typeref_inner(ctx, resolved_parent_ty, scope_id, true, visited);
    }

    let candidate_scopes = find_candidate_scopes_for_type(ctx.tree, &resolved_parent_ty);
    for target_scope in candidate_scopes {
        if member == "super" && ctx.tree.arena[target_scope].is_module {
            if let Some(parent_id) = ctx.tree.arena[target_scope].parent {
                let parent_path = build_path_from_scope(ctx.tree, parent_id);
                return Some(TypeRef::Resolved(parent_path));
            }
        }

        if let Some(mut res) =
            find_symbol_in_scope_and_supers(ctx, target_scope, member, resolve_type, visited)
        {
            if let TypeRef::Generic { ref args, .. } = resolved_parent_ty {
                res = instantiate_generic_member_type(ctx, target_scope, res, args);
            }

            if let Some(base) = base_access {
                return Some(TypeRef::EvaluatedAccess(base, Box::new(res)));
            } else {
                return Some(TypeRef::EvaluatedAccess(
                    Box::new(parent_ty.clone()),
                    Box::new(res),
                ));
            }
        }

        // Transitive imports check
        if let Some(res) = resolve_via_transitive_imports(ctx, target_scope, member, visited) {
            return Some(res);
        }
    }

    // Fallback: append member to the resolved parent type
    match resolved_parent_ty {
        TypeRef::Resolved(mut path) => {
            let base = path.clone();
            path.push(member.to_string());
            Some(TypeRef::EvaluatedAccess(
                Box::new(TypeRef::Resolved(base)),
                Box::new(TypeRef::Resolved(path)),
            ))
        }
        TypeRef::External(mut path) => {
            let base = path.clone();
            path.push(member.to_string());
            Some(TypeRef::EvaluatedAccess(
                Box::new(TypeRef::External(base)),
                Box::new(TypeRef::External(path)),
            ))
        }
        TypeRef::Unresolved(mut path) => {
            let base = path.clone();
            path.push(member.to_string());
            Some(TypeRef::EvaluatedAccess(
                Box::new(TypeRef::Unresolved(base)),
                Box::new(TypeRef::Unresolved(path)),
            ))
        }
        TypeRef::Primitive(prim) => {
            let path = vec![prim.clone(), member.to_string()];
            Some(TypeRef::EvaluatedAccess(
                Box::new(TypeRef::Primitive(prim)),
                Box::new(TypeRef::External(path)),
            ))
        }
        TypeRef::Generic { base, .. } => match *base {
            TypeRef::Resolved(mut path) => {
                let b = path.clone();
                path.push(member.to_string());
                Some(TypeRef::EvaluatedAccess(
                    Box::new(TypeRef::Resolved(b)),
                    Box::new(TypeRef::Resolved(path)),
                ))
            }
            TypeRef::External(mut path) => {
                let b = path.clone();
                path.push(member.to_string());
                Some(TypeRef::EvaluatedAccess(
                    Box::new(TypeRef::External(b)),
                    Box::new(TypeRef::External(path)),
                ))
            }
            TypeRef::Unresolved(mut path) => {
                let b = path.clone();
                path.push(member.to_string());
                Some(TypeRef::EvaluatedAccess(
                    Box::new(TypeRef::Unresolved(b)),
                    Box::new(TypeRef::Unresolved(path)),
                ))
            }
            TypeRef::Failed(mut path) => {
                let b = path.clone();
                path.push(member.to_string());
                Some(TypeRef::EvaluatedAccess(
                    Box::new(TypeRef::External(b)),
                    Box::new(TypeRef::External(path)),
                ))
            }
            TypeRef::Primitive(prim) => {
                let path = vec![prim.clone(), member.to_string()];
                Some(TypeRef::EvaluatedAccess(
                    Box::new(TypeRef::Primitive(prim)),
                    Box::new(TypeRef::External(path)),
                ))
            }
            _ => None,
        },
        TypeRef::EvaluatedAccess(base, inner) => {
            let curr_base = base;
            let mut curr_inner = *inner;
            while let TypeRef::EvaluatedAccess(_, next_inner) = curr_inner {
                curr_inner = *next_inner;
            }
            match curr_inner {
                TypeRef::Resolved(mut path) => {
                    path.push(member.to_string());
                    Some(TypeRef::EvaluatedAccess(
                        curr_base,
                        Box::new(TypeRef::Resolved(path)),
                    ))
                }
                TypeRef::External(mut path) => {
                    path.push(member.to_string());
                    Some(TypeRef::EvaluatedAccess(
                        curr_base,
                        Box::new(TypeRef::External(path)),
                    ))
                }
                TypeRef::Unresolved(mut path) => {
                    path.push(member.to_string());
                    Some(TypeRef::EvaluatedAccess(
                        curr_base,
                        Box::new(TypeRef::Unresolved(path)),
                    ))
                }
                TypeRef::Primitive(prim) => {
                    let path = vec![prim, member.to_string()];
                    Some(TypeRef::EvaluatedAccess(
                        curr_base,
                        Box::new(TypeRef::External(path)),
                    ))
                }
                TypeRef::Generic { base: gen_base, .. } => match *gen_base {
                    TypeRef::Resolved(mut path) => {
                        path.push(member.to_string());
                        Some(TypeRef::EvaluatedAccess(
                            curr_base,
                            Box::new(TypeRef::Resolved(path)),
                        ))
                    }
                    TypeRef::External(mut path) => {
                        path.push(member.to_string());
                        Some(TypeRef::EvaluatedAccess(
                            curr_base,
                            Box::new(TypeRef::External(path)),
                        ))
                    }
                    TypeRef::Unresolved(mut path) => {
                        path.push(member.to_string());
                        Some(TypeRef::EvaluatedAccess(
                            curr_base,
                            Box::new(TypeRef::Unresolved(path)),
                        ))
                    }
                    TypeRef::Primitive(prim) => {
                        let path = vec![prim, member.to_string()];
                        Some(TypeRef::EvaluatedAccess(
                            curr_base,
                            Box::new(TypeRef::External(path)),
                        ))
                    }
                    _ => None,
                },
                _ => None,
            }
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
            if ctx.tree.arena[*id].is_phantom {
                let bounds = ctx.tree.arena[*id]
                    .super_types
                    .iter()
                    .map(|b| evaluate_typeref_inner(ctx, b.clone(), scope_id, true, visited))
                    .collect();
                return TypeRef::TypeVar {
                    name: ctx.tree.arena[*id].name.clone(),
                    bounds,
                };
            }
            let path = build_path_from_scope(ctx.tree, *id);
            if resolve_type {
                TypeRef::EvaluatedAccess(
                    Box::new(TypeRef::Resolved(path.clone())),
                    Box::new(TypeRef::Resolved(path)),
                )
            } else {
                TypeRef::Resolved(path)
            }
        }
        Symbol::Value(ty) | Symbol::TypeAlias(ty) => {
            let mut path = build_path_from_scope(ctx.tree, scope_id);
            path.push(name.to_string());
            let base_path = TypeRef::Resolved(path.clone());
            if resolve_type {
                let sym_key = format!("sym:{}:{}", scope_id, name);
                if !visited.insert(sym_key.clone()) {
                    return base_path;
                }

                let resolved_ty = match ty {
                    TypeRef::ResolutionQuery(q) => evaluate_query(ctx, q, scope_id, true, visited)
                        .unwrap_or_else(|| ty.clone()),
                    TypeRef::Unresolved(_) | TypeRef::TypeVar { .. } | TypeRef::Generic { .. } => {
                        evaluate_typeref_inner(ctx, ty.clone(), scope_id, true, visited)
                    }
                    _ => ty.clone(),
                };
                visited.remove(&sym_key);
                TypeRef::EvaluatedAccess(Box::new(base_path), Box::new(resolved_ty))
            } else {
                base_path
            }
        }
    }
}

/// Given a resolved `TypeRef`, attempts to locate its corresponding `ScopeId` in the ScopeTree.
/// Instantiates any type parameters of `target_scope` present in `res` with the concrete `type_args`.
fn instantiate_generic_member_type(
    ctx: &ExecutorContext,
    target_scope: ScopeId,
    res: TypeRef,
    type_args: &[TypeRef],
) -> TypeRef {
    let param_names = &ctx.tree.arena[target_scope].type_parameters;
    if param_names.is_empty() || type_args.is_empty() {
        return res;
    }

    let map: std::collections::HashMap<String, TypeRef> = param_names
        .iter()
        .zip(type_args.iter())
        .map(|(name, arg)| (name.clone(), arg.clone()))
        .collect();

    fn substitute_type_in_ref(
        tr: TypeRef,
        map: &std::collections::HashMap<String, TypeRef>,
    ) -> TypeRef {
        match tr {
            TypeRef::Unresolved(ref qn) if qn.len() == 1 => {
                if let Some(sub) = map.get(&qn[0]) {
                    return sub.clone();
                }
                tr
            }
            TypeRef::Resolved(ref qn) => {
                if let Some(last) = qn.last()
                    && let Some(sub) = map.get(last)
                {
                    return sub.clone();
                }
                tr
            }
            TypeRef::TypeVar { ref name, .. } => {
                if let Some(sub) = map.get(name) {
                    return sub.clone();
                }
                tr
            }
            TypeRef::Generic { base, args } => TypeRef::Generic {
                base: Box::new(substitute_type_in_ref(*base, map)),
                args: args
                    .into_iter()
                    .map(|a| substitute_type_in_ref(a, map))
                    .collect(),
            },
            TypeRef::EvaluatedAccess(acc, inner) => {
                TypeRef::EvaluatedAccess(acc, Box::new(substitute_type_in_ref(*inner, map)))
            }
            _ => tr,
        }
    }

    substitute_type_in_ref(res, &map)
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
                if let Some(Symbol::Module(id) | Symbol::Type(id)) =
                    tree.arena[curr].symbols.get(part)
                {
                    curr = *id;
                    continue;
                }
                if let Some(ids) = tree.arena[curr].children_by_name.get(part) {
                    if let Some(&child_id) = ids.first() {
                        curr = child_id;
                        continue;
                    }
                }
                return None;
            }
            Some(curr)
        }
        TypeRef::EvaluatedAccess(_, inner) => find_scope_for_type(tree, inner),
        TypeRef::Generic { base, .. } => find_scope_for_type(tree, base),
        TypeRef::TypeVar { bounds, .. } => {
            for b in bounds {
                if let Some(sid) = find_scope_for_type(tree, b) {
                    return Some(sid);
                }
            }
            None
        }
        _ => None,
    }
}

/// Locates all candidate `ScopeId`s for a `TypeRef` (e.g. multiple bounds for `TypeVar`).
pub fn find_candidate_scopes_for_type(tree: &ScopeTree, ty: &TypeRef) -> Vec<ScopeId> {
    match ty {
        TypeRef::TypeVar { bounds, .. } => bounds
            .iter()
            .filter_map(|b| find_scope_for_type(tree, b))
            .collect(),
        TypeRef::EvaluatedAccess(_, inner) => find_candidate_scopes_for_type(tree, inner),
        _ => find_scope_for_type(tree, ty).into_iter().collect(),
    }
}

/// Resolves a fully qualified path starting from the global root.
/// Also handles transitive imports resolution natively.
pub fn find_global(ctx: &ExecutorContext, path: &[String]) -> Option<TypeRef> {
    let mut visited = std::collections::HashSet::new();
    find_global_internal(ctx, path, &mut visited)
}

pub fn find_global_internal(
    ctx: &ExecutorContext,
    path: &[String],
    visited: &mut std::collections::HashSet<String>,
) -> Option<TypeRef> {
    let path_key = format!("global:{}", path.join("::"));
    if !visited.insert(path_key.clone()) {
        return None;
    }

    let mut curr = ctx.tree.root;

    if path.len() == 1 && ctx.primitives.is_primitive(&path[0]) {
        visited.remove(&path_key);
        return Some(TypeRef::Primitive(path[0].clone()));
    }
    let joined_colon = path.join("::");
    if ctx.primitives.is_primitive(&joined_colon) {
        visited.remove(&path_key);
        return Some(TypeRef::Primitive(joined_colon));
    }
    let joined_dot = path.join(".");
    if ctx.primitives.is_primitive(&joined_dot) {
        visited.remove(&path_key);
        return Some(TypeRef::Primitive(joined_dot));
    }

    for (i, part) in path.iter().enumerate() {
        if (part == "root" || part == "crate") && i == 0 {
            continue;
        }
        if let Some(sym) = ctx.tree.arena[curr].symbols.get(part) {
            match sym {
                Symbol::Module(id) | Symbol::Type(id) => curr = *id,
                Symbol::Value(ty) | Symbol::TypeAlias(ty) => {
                    visited.remove(&path_key);
                    if i == path.len() - 1 {
                        return Some(ty.clone());
                    } else {
                        return None;
                    }
                }
            }
        } else {
            // Check transitive imports
            if let Some(resolved) = resolve_via_transitive_imports(ctx, curr, part, visited) {
                if i == path.len() - 1 {
                    visited.remove(&path_key);
                    return Some(resolved);
                } else {
                    if let Some(next_scope) = find_scope_for_type(ctx.tree, &resolved) {
                        curr = next_scope;
                        continue;
                    }
                }
            }

            visited.remove(&path_key);
            return None;
        }
    }

    let final_path = build_path_from_scope(ctx.tree, curr);
    visited.remove(&path_key);
    Some(TypeRef::EvaluatedAccess(
        Box::new(TypeRef::Resolved(final_path.clone())),
        Box::new(TypeRef::Resolved(final_path)),
    ))
}

/// Checks if the target scope is a module with `transitive_imports` enabled,
/// and attempts to resolve the member through its exported imports.
fn resolve_via_transitive_imports(
    ctx: &ExecutorContext,
    scope_id: ScopeId,
    member: &str,
    visited: &mut std::collections::HashSet<String>,
) -> Option<TypeRef> {
    let trans_key = format!("trans:{}:{}", scope_id, member);
    if !visited.insert(trans_key.clone()) {
        return None;
    }

    let node = &ctx.tree.arena[scope_id];
    let mut result = None;

    if node.is_module {
        let lang = node.language.as_deref().unwrap_or("root");
        if ctx.config.get_for(lang).transitive_imports {
            for imp in &node.imports {
                if let Some(last) = imp.path.last() {
                    if last == member {
                        if let Some(resolved) = resolve_import_path(ctx, scope_id, &imp.path, visited) {
                            result = Some(resolved);
                            break;
                        } else {
                            result = Some(TypeRef::External(imp.path.clone()));
                            break;
                        }
                    } else if last == "*" || imp.is_wildcard {
                        let mut target_path = imp.path.clone();
                        if target_path.last().map(|s| s.as_str()) == Some("*") {
                            target_path.pop();
                        }
                        target_path.push(member.to_string());

                        if let Some(resolved) = resolve_import_path(ctx, scope_id, &target_path, visited) {
                            result = Some(resolved);
                            break;
                        }
                    }
                }
            }
        }
    }

    visited.remove(&trans_key);
    result
}
