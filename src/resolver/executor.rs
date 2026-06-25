use crate::model::*;
use super::registry::GlobalRegistry;
use super::primitives::PrimitiveRegistry;

pub struct ExecutorContext<'a> {
    pub current_prefix: QualifiedName,
    pub imports_stack: Vec<Vec<Import>>,
    pub registry: &'a GlobalRegistry,
    pub primitives: &'a PrimitiveRegistry,
    pub config: &'a crate::config::AnalyzerConfig,
    pub current_lang: Option<String>,
}

pub fn execute_queries(modules: Vec<Module>, primitives: &PrimitiveRegistry, config: &crate::config::AnalyzerConfig) -> Vec<Module> {
    let registry = GlobalRegistry::build(&modules);
    
    let ctx = ExecutorContext {
        current_prefix: vec![],
        imports_stack: vec![],
        registry: &registry,
        primitives,
        config,
        current_lang: None,
    };

    modules.into_iter().map(|m| execute_module(&ctx, m)).collect()
}

fn execute_module(ctx: &ExecutorContext, mut m: Module) -> Module {
    let mut new_prefix = ctx.current_prefix.clone();
    new_prefix.extend(m.name.clone());
    m.name = new_prefix.clone();

    let mut new_imports = ctx.imports_stack.clone();
    new_imports.push(m.imports.clone());

    let new_ctx = ExecutorContext {
        current_prefix: new_prefix.clone(),
        imports_stack: new_imports,
        registry: ctx.registry,
        primitives: ctx.primitives,
        config: ctx.config,
        current_lang: m.language.clone().or_else(|| ctx.current_lang.clone()),
    };

    m.structured_types = m.structured_types.into_iter().map(|st| execute_structured_type(&new_ctx, st)).collect();
    m.free_functions = m.free_functions.into_iter().map(|ff| execute_function(&new_ctx, ff)).collect();
    m.impl_blocks = m.impl_blocks.into_iter().map(|ib| execute_impl_block(&new_ctx, ib)).collect();
    m.sub_modules = m.sub_modules.into_iter().map(|sub| execute_module(&new_ctx, sub)).collect();

    // Flatten impl blocks
    let mut unfused_impls = vec![];
    for ib in m.impl_blocks {
        let mut fused = false;
        if let TypeRef::Resolved(ref target_name) = ib.impl_for {
            if let Some(target_st) = m.structured_types.iter_mut().find(|st| &st.name == target_name) {
                target_st.methods.extend(ib.methods.clone());
                target_st.nested_types.extend(ib.nested_types.clone());
                if let Some(trait_ref) = ib.implements_trait.clone() {
                    target_st.super_types.push(trait_ref);
                }
                fused = true;
            }
        }
        if !fused {
            unfused_impls.push(ib);
        }
    }
    m.impl_blocks = unfused_impls;

    m
}

fn execute_structured_type(ctx: &ExecutorContext, mut st: StructuredType) -> StructuredType {
    let mut new_prefix = ctx.current_prefix.clone();
    new_prefix.extend(st.name.clone());
    st.name = new_prefix.clone();

    let new_ctx = ExecutorContext {
        current_prefix: new_prefix,
        imports_stack: ctx.imports_stack.clone(),
        registry: ctx.registry,
        primitives: ctx.primitives,
        config: ctx.config,
        current_lang: ctx.current_lang.clone(),
    };

    st.super_types = st.super_types.into_iter().map(|t| evaluate_typeref(&new_ctx, t)).collect();
    st.fields = st.fields.into_iter().map(|mut f| { f.ty = evaluate_typeref(&new_ctx, f.ty); f }).collect();
    st.methods = st.methods.into_iter().map(|m| execute_function(&new_ctx, m)).collect();
    st.nested_types = st.nested_types.into_iter().map(|n| execute_structured_type(&new_ctx, n)).collect();

    st
}

fn execute_function(ctx: &ExecutorContext, mut f: Function) -> Function {
    let mut new_prefix = ctx.current_prefix.clone();
    new_prefix.extend(f.name.clone());
    f.name = new_prefix.clone();

    let new_ctx = ExecutorContext {
        current_prefix: new_prefix,
        imports_stack: ctx.imports_stack.clone(),
        registry: ctx.registry,
        primitives: ctx.primitives,
        config: ctx.config,
        current_lang: ctx.current_lang.clone(),
    };

    f.signature.parameters = f.signature.parameters.into_iter().map(|mut p| { p.ty = evaluate_typeref(&new_ctx, p.ty); p }).collect();
    f.signature.return_type = evaluate_typeref(&new_ctx, f.signature.return_type);
    f.body = f.body.map(|b| execute_block(&new_ctx, b));
    
    f
}

fn execute_block(ctx: &ExecutorContext, mut b: Block) -> Block {
    b.declarations = b.declarations.into_iter().map(|mut d| { d.ty = evaluate_typeref(ctx, d.ty); d }).collect();
    b.calls = b.calls.into_iter().map(|c| evaluate_typeref(ctx, c)).collect();
    b.instantiates = b.instantiates.into_iter().map(|i| evaluate_typeref(ctx, i)).collect();
    b.sub_blocks = b.sub_blocks.into_iter().map(|sub| execute_block(ctx, sub)).collect();
    b
}

fn execute_impl_block(ctx: &ExecutorContext, mut ib: ImplBlock) -> ImplBlock {
    ib.impl_for = evaluate_typeref(ctx, ib.impl_for);
    ib.implements_trait = ib.implements_trait.map(|t| evaluate_typeref(ctx, t));

    let mut target_prefix = ctx.current_prefix.clone();
    if let TypeRef::Resolved(ref qn) = ib.impl_for {
        target_prefix.extend(qn.last().cloned());
    } else {
        target_prefix.extend(ib.name.clone());
    }
    ib.name = target_prefix.clone();

    let new_ctx = ExecutorContext {
        current_prefix: target_prefix,
        imports_stack: ctx.imports_stack.clone(),
        registry: ctx.registry,
        primitives: ctx.primitives,
        config: ctx.config,
        current_lang: ctx.current_lang.clone(),
    };

    ib.methods = ib.methods.into_iter().map(|m| execute_function(&new_ctx, m)).collect();
    ib.nested_types = ib.nested_types.into_iter().map(|n| execute_structured_type(&new_ctx, n)).collect();
    ib
}

fn evaluate_typeref(ctx: &ExecutorContext, tr: TypeRef) -> TypeRef {
    match tr {
        TypeRef::ResolutionQuery(query) => {
            if let Some(res) = evaluate_query(ctx, &query, false) {
                // If evaluating `Call` or `Extract` succeeds, `res` is an absolute path.
                // We check if it's local or external.
                if ctx.registry.exists(&res) {
                    TypeRef::Resolved(res)
                } else if res.first() == Some(&"crate".to_string()) {
                    let mut crate_cand = vec!["root".to_string()];
                    crate_cand.extend(res.into_iter().skip(1));
                    if ctx.registry.exists(&crate_cand) {
                        TypeRef::Resolved(crate_cand)
                    } else {
                        TypeRef::External(crate_cand)
                    }
                } else {
                    // For now, if it resolves via fallback but isn't in registry, it's external.
                    TypeRef::External(res)
                }
            } else {
                // Fallback to primitive
                let name = extract_base_name(&query);
                if ctx.primitives.is_primitive(&name) {
                    TypeRef::Primitive(PrimitiveType::Other(name))
                } else {
                    TypeRef::Failed(vec![name])
                }
            }
        }
        _ => tr,
    }
}

pub fn extract_base_name(query: &Query) -> String {
    match query {
        Query::Find(n) => n.clone(),
        Query::Extract(q, _) => extract_base_name(q),
        Query::Call(q) => extract_base_name(q),
    }
}

/// Evaluates a query algebraically against the global registry and current context.
/// If `resolve_type` is true, an operation (like `Call` or `Extract` of a field) evaluates to its resulting type.
/// If false, it evaluates to the path itself (useful for logging dependency edges).
fn evaluate_query(ctx: &ExecutorContext, query: &Query, resolve_type: bool) -> Option<QualifiedName> {
    match query {
        Query::Find(name) => {
            // Find-going-up: Ascend the current prefix
            let mut prefix = ctx.current_prefix.clone();
            loop {
                let mut candidate = prefix.clone();
                candidate.push(name.clone());
                if ctx.registry.exists(&candidate) {
                    return Some(candidate);
                }
                
                // Check imports at this level
                // We iterate imports stack in reverse
                for level_imports in ctx.imports_stack.iter().rev() {
                    for imp in level_imports {
                        if let Some(alias) = &imp.alias {
                            if alias == name {
                                return Some(imp.path.clone());
                            }
                        } else if let Some(last) = imp.path.last() {
                            if last == name {
                                return Some(imp.path.clone());
                            }
                        }
                    }
                }

                if prefix.is_empty() {
                    break;
                }
                prefix.pop();
            }

            // Absolute fallbacks
            let direct = vec![name.clone()];
            if ctx.registry.exists(&direct) {
                return Some(direct);
            }
            let root_cand = vec!["root".to_string(), name.clone()];
            if ctx.registry.exists(&root_cand) {
                return Some(root_cand);
            }

            None
        }
        Query::Extract(parent_query, member_name) => {
            // Find-going-down
            let parent_path = evaluate_query(ctx, parent_query, true)?;
            
            // Function to recursively search for a member in a class and its super_types
            fn find_member(ctx: &ExecutorContext, current_path: QualifiedName, member: &String, visited: &mut std::collections::HashSet<QualifiedName>) -> Option<QualifiedName> {
                let mut candidate = current_path.clone();
                candidate.push(member.clone());
                
                // 1. Direct match
                if ctx.registry.exists(&candidate) {
                    return Some(candidate);
                }

                // Cycle check for MRO and Transitive Imports
                let check_cycles = ctx.current_lang.as_deref().map(|l| ctx.config.get_for(l).cyclic_mro_check).unwrap_or(true);
                if check_cycles {
                    if !visited.insert(current_path.clone()) {
                        return None; // Cycle detected
                    }
                }
                
                let entry = ctx.registry.get(&current_path);

                // 2. Transitive Imports Fallback (if current_path is a Module)
                let allow_transitive = ctx.current_lang.as_deref().map(|l| ctx.config.get_for(l).transitive_imports).unwrap_or(false);
                if allow_transitive {
                    if let Some(crate::resolver::registry::RegistryEntry::Module { imports }) = entry {
                        for imp in imports {
                            if imp.is_wildcard {
                                // Wildcard import: search recursively in the imported module
                                if let Some(found) = find_member(ctx, imp.path.clone(), member, visited) {
                                    return Some(found);
                                }
                            } else if let Some(alias) = &imp.alias {
                                if alias == member {
                                    return Some(imp.path.clone());
                                }
                            } else if let Some(last) = imp.path.last() {
                                if last == member {
                                    return Some(imp.path.clone());
                                }
                            }
                        }
                    }
                }

                // 3. Inheritance fallback (MRO: Depth-First, Left-to-Right)
                if let Some(crate::resolver::registry::RegistryEntry::StructuredType { super_types }) = entry {
                    for sup in super_types {
                        if let TypeRef::ResolutionQuery(q) = sup {
                            if let Some(sup_path) = evaluate_query(ctx, q, true) {
                                if let Some(found) = find_member(ctx, sup_path, member, visited) {
                                    return Some(found);
                                }
                            }
                        }
                    }
                }
                None
            }

            let mut target_path = parent_path.clone();
            target_path.push(member_name.clone());

            let mut visited = std::collections::HashSet::new();
            if let Some(found_path) = find_member(ctx, parent_path.clone(), member_name, &mut visited) {
                target_path = found_path;
            }

            if resolve_type {
                // If we need the type of this field (e.g. chained access `a.b.c`), look up the field in the registry
                if let Some(crate::resolver::registry::RegistryEntry::Field { field_type }) = ctx.registry.get(&target_path) {
                    match field_type {
                        TypeRef::ResolutionQuery(ret_q) => evaluate_query(ctx, ret_q, true),
                        TypeRef::Resolved(qn) => Some(qn.clone()),
                        TypeRef::External(qn) => Some(qn.clone()),
                        TypeRef::Unresolved(qn) => Some(qn.clone()),
                        _ => None,
                    }
                } else {
                    // Not a known field or missing type, but we might still return the path 
                    // in case it's a module/struct member used directly.
                    Some(target_path)
                }
            } else {
                Some(target_path)
            }
        }
        Query::Call(target_query) => {
            let target_path = evaluate_query(ctx, target_query, false)?;

            if resolve_type {
                // If we need the return type (e.g. chained calls `a.f().g()`), we look up the target_path in the registry
                if let Some(crate::resolver::registry::RegistryEntry::Function { return_type }) = ctx.registry.get(&target_path) {
                    // Try to evaluate the return type mathematically!
                    match return_type {
                        TypeRef::ResolutionQuery(ret_q) => evaluate_query(ctx, ret_q, true),
                        TypeRef::Resolved(qn) => Some(qn.clone()),
                        TypeRef::External(qn) => Some(qn.clone()),
                        TypeRef::Unresolved(qn) => Some(qn.clone()), // Fallback, though builder substitutes all
                        _ => None,
                    }
                } else {
                    None // Not a known function or missing return type
                }
            } else {
                // Standard behavior: evaluate to the function path itself
                Some(target_path)
            }
        }
    }
}