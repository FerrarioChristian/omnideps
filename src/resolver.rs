use crate::ir::*;
use std::cell::RefCell;
use std::collections::HashMap;

// ==================== HIERARCHICAL SCOPE TREE ====================

#[derive(Debug)]
pub struct ScopeNode {
    pub name: Identifier,
    pub symbols: HashMap<Identifier, Component>,
    pub children: HashMap<Identifier, usize>,
    pub parent: Option<usize>,
    pub imports: Vec<Import>,
}

#[derive(Debug)]
pub struct ScopeTree {
    pub arena: Vec<ScopeNode>,
    pub root: usize,
}

impl ScopeTree {
    pub fn new() -> Self {
        let root_node = ScopeNode {
            name: "ROOT".to_string(),
            symbols: HashMap::new(),
            children: HashMap::new(),
            parent: None,
            imports: vec![],
        };
        ScopeTree {
            arena: vec![root_node],
            root: 0,
        }
    }

    fn add_node(&mut self, parent: usize, name: Identifier, imports: Vec<Import>) -> usize {
        let idx = self.arena.len();
        let node = ScopeNode {
            name: name.clone(),
            symbols: HashMap::new(),
            children: HashMap::new(),
            parent: Some(parent),
            imports,
        };
        self.arena.push(node);
        self.arena[parent].children.insert(name, idx);
        idx
    }

    pub fn build(modules: &[Module]) -> Self {
        let mut tree = ScopeTree::new();
        let root = tree.root;

        fn populate(
            m: &Module,
            tree: &mut ScopeTree,
            parent_idx: usize,
        ) {
            let m_name = m.name.last().cloned().unwrap_or_default();
            let node_idx = tree.add_node(parent_idx, m_name.clone(), m.imports.clone());
            tree.arena[parent_idx].symbols.insert(m_name.clone(), Component::Module(m.clone()));

            fn populate_st(
                st: &StructuredType,
                tree: &mut ScopeTree,
                parent_idx: usize,
            ) {
                let st_name = st.name.last().cloned().unwrap_or_default();
                let node_idx = tree.add_node(parent_idx, st_name.clone(), vec![]);
                tree.arena[parent_idx].symbols.insert(st_name.clone(), Component::StructuredType(st.clone()));

                for nested in &st.nested_types {
                    populate_st(nested, tree, node_idx);
                }
            }

            for st in &m.structured_types {
                populate_st(st, tree, node_idx);
            }
            for f in &m.free_functions {
                let f_name = f.name.last().cloned().unwrap_or_default();
                tree.arena[node_idx].symbols.insert(f_name, Component::Function(f.clone()));
            }
            for ib in &m.impl_blocks {
                let target_name = match &ib.impl_for {
                    TypeRef::Unresolved(qn) | TypeRef::Resolved(qn) => qn.last().cloned().unwrap_or_default(),
                    _ => continue,
                };
                
                let target_idx = if let Some(&idx) = tree.arena[node_idx].children.get(&target_name) {
                    idx
                } else {
                    tree.add_node(node_idx, target_name.clone(), vec![])
                };

                for nested in &ib.nested_types {
                    populate_st(nested, tree, target_idx);
                }
                for method in &ib.methods {
                    let method_name = method.name.last().cloned().unwrap_or_default();
                    tree.arena[target_idx].symbols.insert(method_name, Component::Function(method.clone()));
                }
            }
            for sub in &m.sub_modules {
                populate(sub, tree, node_idx);
            }
        }

        for m in modules {
            populate(m, &mut tree, root);
        }
        tree
    }
    
    pub fn get_path_for_node(&self, mut node_idx: usize) -> QualifiedName {
        let mut path = vec![];
        while node_idx != self.root {
            path.push(self.arena[node_idx].name.clone());
            if let Some(p) = self.arena[node_idx].parent {
                node_idx = p;
            } else {
                break;
            }
        }
        path.reverse();
        path
    }
    pub fn find_node_by_path(&self, path: &QualifiedName) -> Option<usize> {
        if path.is_empty() { return None; }
        
        let mut curr = self.root;
        // The root itself might be named "root", or maybe the path starts with the child of root
        // If path[0] is in root's children, start there.
        let mut parts = path.iter();
        
        // This logic simulates absolute path matching against the global scope tree.
        // It's a simplification. A real absolute match would start at the true root.
        while let Some(part) = parts.next() {
            if let Some(&child) = self.arena[curr].children.get(part) {
                curr = child;
            } else {
                return None;
            }
        }
        Some(curr)
    }
}

// ==================== CONTEXT PER RISOLUZIONE NOMI ====================
#[derive(Debug)]
pub struct ResolutionContext<'a> {
    pub current_prefix: QualifiedName,
    pub tree: &'a ScopeTree,
    pub current_scope: usize,
    pub cache: &'a RefCell<HashMap<(usize, QualifiedName), ResolutionResult>>,
}

/// Resolves type references across all modules by matching them against the global symbol table.
pub fn resolve_type_refs(modules: Vec<Module>) -> Vec<Module> {
    let tree = ScopeTree::build(&modules);
    let cache = RefCell::new(HashMap::new());
    let ctx = ResolutionContext {
        current_prefix: vec![],
        tree: &tree,
        current_scope: tree.root,
        cache: &cache,
    };
    modules
        .into_iter()
        .map(|m| resolve_module_in_context(&ctx, m))
        .collect()
}

#[derive(Debug, Clone)]
pub enum ResolutionResult {
    Local(QualifiedName),
    External(QualifiedName),
}

// Regole di risoluzione: Memoization -> Lexical Climb (Locale -> Imports) -> Ascensione
fn resolve_name_in_context(
    ctx: &ResolutionContext,
    name: &QualifiedName,
) -> Option<ResolutionResult> {
    if name.is_empty() { return None; }
    let first_part = name.first().unwrap();

    // 0. Cache (Memoization)
    let cache_key = (ctx.current_scope, name.clone());
    if let Some(res) = ctx.cache.borrow().get(&cache_key) {
        return Some(res.clone());
    }

    let validate_import = |candidate: &QualifiedName| -> ResolutionResult {
        if ctx.tree.find_node_by_path(candidate).is_some() {
            ResolutionResult::Local(candidate.clone())
        } else {
            // Fallback: try prepending "root"
            let mut root_cand = vec!["root".to_string()];
            root_cand.extend(candidate.clone());
            if ctx.tree.find_node_by_path(&root_cand).is_some() {
                ResolutionResult::Local(root_cand)
            } else {
                // Fallback: replace "crate" with "root"
                if candidate.first() == Some(&"crate".to_string()) {
                    let mut crate_cand = vec!["root".to_string()];
                    crate_cand.extend(candidate.iter().skip(1).cloned());
                    if ctx.tree.find_node_by_path(&crate_cand).is_some() {
                        ResolutionResult::Local(crate_cand)
                    } else {
                        ResolutionResult::External(candidate.clone())
                    }
                } else {
                    ResolutionResult::External(candidate.clone())
                }
            }
        }
    };

    let mut curr_idx = ctx.current_scope;
    
    // Lexical Climb
    loop {
        let node = &ctx.tree.arena[curr_idx];
        
        // 1. Ricerca Locale (Relative to this scope)
        if node.symbols.contains_key(first_part) {
            // Found base! Need to resolve the rest of the path by traversing children
            if let Some(&child_idx) = node.children.get(first_part) {
                 let mut resolved_idx = child_idx;
                 let mut valid = true;
                 for part in name.iter().skip(1) {
                     if let Some(&next_idx) = ctx.tree.arena[resolved_idx].children.get(part) {
                         resolved_idx = next_idx;
                     } else {
                         valid = false;
                         break;
                     }
                 }
                 if valid {
                     let abs_path = ctx.tree.get_path_for_node(resolved_idx);
                     let res = ResolutionResult::Local(abs_path);
                     ctx.cache.borrow_mut().insert(cache_key.clone(), res.clone());
                     return Some(res);
                 }
            } else if name.len() == 1 {
                // It's a leaf (like a function) with no children
                let mut abs_path = ctx.tree.get_path_for_node(curr_idx);
                abs_path.push(first_part.clone());
                let res = ResolutionResult::Local(abs_path);
                ctx.cache.borrow_mut().insert(cache_key.clone(), res.clone());
                return Some(res);
            }
        }

        // 2. Jump Points (Imports)
        for imp in &node.imports {
            if let Some(alias) = &imp.alias {
                if alias == first_part {
                    let mut candidate = imp.path.clone();
                    candidate.extend(name.iter().skip(1).cloned());
                    
                    let res = validate_import(&candidate);
                    ctx.cache.borrow_mut().insert(cache_key.clone(), res.clone());
                    return Some(res);
                }
            } else if !imp.is_wildcard {
                if let Some(last_part) = imp.path.last() {
                    if last_part == first_part {
                        let mut candidate = imp.path.clone();
                        // Se l'import è `use std::vec::Vec`, e usiamo `Vec::new`, 
                        // vogliamo che parta da dopo `Vec`.
                        candidate.extend(name.iter().skip(1).cloned());
                        
                        let res = validate_import(&candidate);
                        ctx.cache.borrow_mut().insert(cache_key.clone(), res.clone());
                        return Some(res);
                    }
                }
            }
        }
        
        // 2b. Wildcard Jump Points
        for imp in &node.imports {
            if imp.is_wildcard {
                let mut candidate = imp.path.clone();
                candidate.extend(name.clone());
                let res = validate_import(&candidate);
                // Non registriamo sempre external da wildcard perché causerebbe falsi positivi massicci
                if let ResolutionResult::Local(_) = res {
                    ctx.cache.borrow_mut().insert(cache_key.clone(), res.clone());
                    return Some(res);
                }
            }
        }

        // 3. Ascensione (Scope Bubbling)
        if let Some(parent) = node.parent {
            curr_idx = parent;
        } else {
            break;
        }
    }

    // Ultimo fallback assoluto se si passa il percorso dalla radice o se i path sono slegati
    // Nel caso di "A::InnerA", proviamo a vedere se esiste a partire dalla root globale.
    if let Some(idx) = ctx.tree.find_node_by_path(name) {
        let abs_path = ctx.tree.get_path_for_node(idx);
        let res = ResolutionResult::Local(abs_path);
        ctx.cache.borrow_mut().insert(cache_key.clone(), res.clone());
        return Some(res);
    }
    
    // Tentativo super globale: aggiungi root
    let mut root_cand = vec!["root".to_string()];
    root_cand.extend(name.clone());
    if let Some(idx) = ctx.tree.find_node_by_path(&root_cand) {
        let abs_path = ctx.tree.get_path_for_node(idx);
        let res = ResolutionResult::Local(abs_path);
        ctx.cache.borrow_mut().insert(cache_key.clone(), res.clone());
        return Some(res);
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

    let new_scope = ctx.tree.find_node_by_path(&new_prefix).unwrap_or(ctx.current_scope);

    module.name = new_prefix.clone();
    let new_ctx = ResolutionContext {
        current_prefix: new_prefix.clone(),
        tree: ctx.tree,
        current_scope: new_scope,
        cache: ctx.cache,
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

    let mut unfused_impls = vec![];
    for ib in resolved_impls {
        let mut fused = false;
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
                fused = true;
            }
        }
        if !fused {
            unfused_impls.push(ib);
        }
    }
    module.impl_blocks = unfused_impls; // Keep unfused ones

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
    
    let new_scope = ctx.tree.find_node_by_path(&new_prefix).unwrap_or(ctx.current_scope);

    let new_ctx = ResolutionContext {
        current_prefix: new_prefix.clone(),
        tree: ctx.tree,
        current_scope: new_scope,
        cache: ctx.cache,
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

    // Try to determine the scope based on the impl block's target
    let target_name = match &i.impl_for {
        TypeRef::Unresolved(qn) | TypeRef::Resolved(qn) => qn.last().cloned().unwrap_or_default(),
        _ => "".to_string(),
    };
    
    let new_scope = if !target_name.is_empty() {
        if let Some(&child_idx) = ctx.tree.arena[ctx.current_scope].children.get(&target_name) {
            child_idx
        } else {
            ctx.tree.find_node_by_path(&new_prefix).unwrap_or(ctx.current_scope)
        }
    } else {
        ctx.tree.find_node_by_path(&new_prefix).unwrap_or(ctx.current_scope)
    };

    let new_ctx = ResolutionContext {
        current_prefix: new_prefix.clone(),
        tree: ctx.tree,
        current_scope: new_scope,
        cache: ctx.cache,
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
