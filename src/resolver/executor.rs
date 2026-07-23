use crate::model::*;
use super::scope::{ScopeTree, ScopeId, Symbol};
use super::primitives::PrimitiveRegistry;

pub struct ExecutorContext<'a> {
    pub tree: &'a ScopeTree,
    pub primitives: &'a PrimitiveRegistry,
    pub config: &'a crate::config::AnalyzerConfig,
}

pub fn execute_queries(modules: Vec<Module>, primitives: &PrimitiveRegistry, config: &crate::config::AnalyzerConfig) -> Vec<Module> {
    let tree = ScopeTree::build(&modules, config);
    
    let ctx = ExecutorContext {
        tree: &tree,
        primitives,
        config,
    };

    modules.into_iter().map(|m| execute_module(&ctx, m, tree.root)).collect()
}

fn find_child_scope(tree: &ScopeTree, parent: ScopeId, name: &str, is_module: bool) -> Option<ScopeId> {
    if let Some(sym) = tree.arena[parent].symbols.get(name) {
        match sym {
            Symbol::Module(id) if is_module => return Some(*id),
            Symbol::Type(id) if !is_module => return Some(*id),
            _ => {}
        }
    }
    None
}

pub fn execute_module(ctx: &ExecutorContext, mut m: Module, parent_scope: ScopeId) -> Module {
    let module_name = m.name.last().cloned().unwrap_or_else(|| "unknown".to_string());
    
    let scope_id = if module_name == "root" {
        parent_scope
    } else {
        find_child_scope(ctx.tree, parent_scope, &module_name, true).unwrap_or(parent_scope)
    };

    for fv in m.free_variables.iter_mut() {
        fv.ty = evaluate_typeref(ctx, fv.ty.clone(), scope_id, true);
    }

    m.free_functions = m.free_functions.into_iter().map(|ff| execute_function(ctx, ff, scope_id)).collect();
    m.structured_types = m.structured_types.into_iter().map(|st| execute_structured_type(ctx, st, scope_id)).collect();
    m.impl_blocks = m.impl_blocks.into_iter().map(|ib| execute_impl_block(ctx, ib, scope_id)).collect();
    m.sub_modules = m.sub_modules.into_iter().map(|sub| execute_module(ctx, sub, scope_id)).collect();

    m
}

fn execute_structured_type(ctx: &ExecutorContext, mut st: StructuredType, parent_scope: ScopeId) -> StructuredType {
    let name = st.name.last().cloned().unwrap_or_default();
    let scope_id = find_child_scope(ctx.tree, parent_scope, &name, false).unwrap_or(parent_scope);

    st.super_types = st.super_types.into_iter().map(|t| evaluate_typeref(ctx, t, scope_id, true)).collect();
    st.fields = st.fields.into_iter().map(|mut f| { f.ty = evaluate_typeref(ctx, f.ty, scope_id, true); f }).collect();
    st.methods = st.methods.into_iter().map(|m| execute_function(ctx, m, scope_id)).collect();
    st.nested_types = st.nested_types.into_iter().map(|n| execute_structured_type(ctx, n, scope_id)).collect();
    st
}

fn execute_impl_block(ctx: &ExecutorContext, mut ib: ImplBlock, parent_scope: ScopeId) -> ImplBlock {
    ib.impl_for = evaluate_typeref(ctx, ib.impl_for, parent_scope, true);
    ib.implements_trait = ib.implements_trait.map(|t| evaluate_typeref(ctx, t, parent_scope, true));

    let target_name = match &ib.impl_for {
        TypeRef::Resolved(qn) | TypeRef::External(qn) => qn.last().cloned().unwrap_or_default(),
        TypeRef::ResolutionQuery(q) => extract_base_name(q),
        _ => "".to_string(),
    };

    let scope_id = find_child_scope(ctx.tree, parent_scope, &target_name, false).unwrap_or(parent_scope);

    ib.methods = ib.methods.into_iter().map(|m| execute_function(ctx, m, scope_id)).collect();
    ib.nested_types = ib.nested_types.into_iter().map(|n| execute_structured_type(ctx, n, scope_id)).collect();
    ib
}

fn execute_function(ctx: &ExecutorContext, mut f: Function, parent_scope: ScopeId) -> Function {
    let name = f.name.last().cloned().unwrap_or_default();
    
    let mut func_scope_id = parent_scope;
    for child in &ctx.tree.arena {
        if child.parent == Some(parent_scope) && child.name == name {
            func_scope_id = child.id;
            break;
        }
    }

    f.signature.parameters = f.signature.parameters.into_iter().map(|mut p| { p.ty = evaluate_typeref(ctx, p.ty, func_scope_id, true); p }).collect();
    f.signature.return_type = evaluate_typeref(ctx, f.signature.return_type, func_scope_id, true);
    f.body = f.body.map(|b| execute_block(ctx, b, func_scope_id, 0));
    
    f
}

fn execute_block(ctx: &ExecutorContext, mut b: Block, parent_scope: ScopeId, index: usize) -> Block {
    let block_name = format!("block_{}", index);
    let mut block_scope_id = parent_scope;
    for child in &ctx.tree.arena {
        if child.parent == Some(parent_scope) && child.name == block_name {
            block_scope_id = child.id;
            break;
        }
    }

    b.declarations = b.declarations.into_iter().map(|mut d| { d.ty = evaluate_typeref(ctx, d.ty, block_scope_id, true); d }).collect();
    b.calls = b.calls.into_iter().map(|c| evaluate_typeref(ctx, c, block_scope_id, false)).collect();
    b.instantiates = b.instantiates.into_iter().map(|i| evaluate_typeref(ctx, i, block_scope_id, false)).collect();
    b.accesses = b.accesses.into_iter().map(|a| evaluate_typeref(ctx, a, block_scope_id, false)).collect();
    
    let sub_blocks: Vec<Block> = b.sub_blocks.into_iter().enumerate().map(|(i, sub)| execute_block(ctx, sub, block_scope_id, i)).collect();
    b.sub_blocks = sub_blocks;
    b
}

pub fn evaluate_typeref(ctx: &ExecutorContext, tr: TypeRef, scope_id: ScopeId, resolve_type: bool) -> TypeRef {
    match tr {
        TypeRef::ResolutionQuery(query) => {
            let mut visited = std::collections::HashSet::new();
            if let Some(resolved) = evaluate_query(ctx, &query, scope_id, resolve_type, &mut visited) {
                resolved
            } else {
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
            let res = if let Some(resolved) = evaluate_query(ctx, &query, scope_id, resolve_type, &mut visited) {
                resolved
            } else {
                TypeRef::Unresolved(qn.clone())
            };
            println!("evaluate_typeref for {:?}: returned {:?}", qn, res);
            res
        }
        _ => tr,
    }
}

pub fn extract_base_name(query: &Query) -> String {
    match query {
        Query::Find(name) => name.clone(),
        Query::Extract(parent, member) => format!("{}::{}", extract_base_name(parent), member),
        Query::Call(parent) => format!("{}()", extract_base_name(parent)),
    }
}

fn build_path_from_scope(tree: &ScopeTree, scope_id: ScopeId) -> QualifiedName {
    let mut path = vec![];
    let mut curr = Some(scope_id);
    while let Some(id) = curr {
        if !tree.arena[id].name.starts_with("block") {
            path.push(tree.arena[id].name.clone());
        }
        curr = tree.arena[id].parent;
    }
    path.reverse();
    path
}

pub fn find_symbol_in_scope_and_supers(
    ctx: &ExecutorContext,
    scope_id: ScopeId,
    name: &str,
    resolve_type: bool,
    visited: &mut std::collections::HashSet<String>,
) -> Option<TypeRef> {
    if let Some(sym) = ctx.tree.arena[scope_id].symbols.get(name) {
        return Some(symbol_to_typeref(ctx, scope_id, sym, name, resolve_type, visited));
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
            _ => st.clone()
        };

        if let Some(super_scope) = find_scope_for_type(ctx.tree, &resolved_st) {
            // Prevent infinite loops if circular inheritance
            if scope_id != super_scope {
                if let Some(res) = find_symbol_in_scope_and_supers(ctx, super_scope, name, resolve_type, visited) {
                    return Some(res);
                }
            }
        }
    }
    
    None
}

fn evaluate_query(ctx: &ExecutorContext, query: &Query, scope_id: ScopeId, resolve_type: bool, visited: &mut std::collections::HashSet<String>) -> Option<TypeRef> {
    let q_str = extract_base_name(query);
    if !visited.insert(q_str.clone()) {
        return None;
    }
    println!("evaluate_query: {:?}", query);

    let result = match query {
        Query::Find(name) => {
            let mut curr = Some(scope_id);
            while let Some(id) = curr {
                if let Some(res) = find_symbol_in_scope_and_supers(ctx, id, name, resolve_type, visited) {
                    return Some(res);
                }
                
                if name == "BadgesDAO" {
                    println!("Checking imports for scope {:?} ({:?})", id, ctx.tree.arena[id].name);
                }
                for imp in &ctx.tree.arena[id].imports {
                    if name == "BadgesDAO" {
                        println!("  Import: {:?}", imp.path);
                    }
                    if let Some(last) = imp.path.last() {
                        if last == name || last == "*" {
                            if let Some(resolved) = find_global(ctx, &imp.path) {
                                return Some(resolved);
                            } else {
                                // If not found in the tree, it might be an external library
                                return Some(TypeRef::External(imp.path.clone()));
                            }
                        }
                    }
                }
                
                curr = ctx.tree.arena[id].parent;
            }
            
            find_global(ctx, &[name.clone()])
        }
        Query::Extract(parent_q, member) => {
            let parent_ty = evaluate_query(ctx, parent_q, scope_id, true, visited);
            println!("  parent_ty: {:?}", parent_ty);
            let parent_ty = parent_ty?;
            
            if let Some(target_scope) = find_scope_for_type(ctx.tree, &parent_ty) {
                println!("  target_scope: {:?}", target_scope);
                if let Some(res) = find_symbol_in_scope_and_supers(ctx, target_scope, member, resolve_type, visited) {
                    println!("  res: {:?}", res);
                    return Some(res);
                }
                println!("  find_symbol failed for {:?}", member);
            }
            None
        }
        Query::Call(target_q) => {
             evaluate_query(ctx, target_q, scope_id, true, visited)
        }
    };

    visited.remove(&q_str);
    result
}

fn symbol_to_typeref(ctx: &ExecutorContext, scope_id: ScopeId, sym: &Symbol, name: &str, resolve_type: bool, visited: &mut std::collections::HashSet<String>) -> TypeRef {
    match sym {
        Symbol::Module(id) | Symbol::Type(id) => {
            let path = build_path_from_scope(ctx.tree, *id);
            TypeRef::Resolved(path)
        }
        Symbol::Value(ty) => {
            if resolve_type {
                match ty {
                    TypeRef::ResolutionQuery(q) => {
                        evaluate_query(ctx, q, scope_id, true, visited).unwrap_or_else(|| ty.clone())
                    }
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

pub fn find_scope_for_type(tree: &ScopeTree, ty: &TypeRef) -> Option<ScopeId> {
    match ty {
        TypeRef::Resolved(qn) | TypeRef::External(qn) => {
            let mut curr = tree.root;
            for (i, part) in qn.iter().enumerate() {
                if part == "root" && i == 0 {
                    continue;
                }
                if let Some(sym) = tree.arena[curr].symbols.get(part) {
                    match sym {
                        Symbol::Module(id) | Symbol::Type(id) => curr = *id,
                        _ => return None,
                    }
                } else {
                    return None;
                }
            }
            Some(curr)
        }
        _ => None,
    }
}

pub fn find_global(ctx: &ExecutorContext, path: &[String]) -> Option<TypeRef> {
    let mut curr = ctx.tree.root;
    
    if path.len() == 1 && ctx.primitives.is_primitive(&path[0]) {
        return Some(TypeRef::Primitive(path[0].clone()));
    }

    for (i, part) in path.iter().enumerate() {
        if part == "root" && i == 0 {
            continue;
        }
        if let Some(sym) = ctx.tree.arena[curr].symbols.get(part) {
            match sym {
                Symbol::Module(id) | Symbol::Type(id) => curr = *id,
                Symbol::Value(ty) => {
                    if i == path.len() - 1 {
                        return Some(ty.clone());
                    } else {
                        return None;
                    }
                }
            }
        } else {
            return None;
        }
    }
    
    let final_path = build_path_from_scope(ctx.tree, curr);
    Some(TypeRef::Resolved(final_path))
}