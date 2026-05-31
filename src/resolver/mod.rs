pub mod stack;
pub mod cache;
pub mod primitives;

use crate::ir::*;
use std::cell::RefCell;
use std::collections::HashSet;
use stack::SymbolStack;
use cache::ResolutionCache;
use primitives::PrimitiveRegistry;

#[derive(Debug, Clone)]
pub enum ResolutionResult {
    Local(QualifiedName),
    External(QualifiedName),
}

// ==================== GLOBAL REGISTRY ====================
// Pass 1: We scan the whole IR to record every absolute path that exists in the project.
// This is used to validate if a resolved path is Local or External.
pub struct GlobalRegistry {
    pub paths: HashSet<QualifiedName>,
}

impl GlobalRegistry {
    pub fn build(modules: &[Module]) -> Self {
        let mut registry = Self { paths: HashSet::new() };
        for m in modules {
            registry.register_module(m, vec![]);
        }
        registry
    }

    fn register_module(&mut self, m: &Module, mut prefix: QualifiedName) {
        prefix.extend(m.name.clone());
        self.paths.insert(prefix.clone());

        for st in &m.structured_types {
            self.register_structured_type(st, prefix.clone());
        }
        for ff in &m.free_functions {
            let mut ff_prefix = prefix.clone();
            ff_prefix.extend(ff.name.clone());
            self.paths.insert(ff_prefix);
        }
        for ib in &m.impl_blocks {
            // Register ImplBlock methods and nested types
            let target_name = match &ib.impl_for {
                TypeRef::Unresolved(qn) | TypeRef::Resolved(qn) => qn.last().cloned().unwrap_or_default(),
                _ => "".to_string(),
            };
            if !target_name.is_empty() {
                let mut target_prefix = prefix.clone();
                target_prefix.push(target_name);
                for method in &ib.methods {
                    let mut m_prefix = target_prefix.clone();
                    m_prefix.extend(method.name.clone());
                    self.paths.insert(m_prefix);
                }
                for nested in &ib.nested_types {
                    self.register_structured_type(nested, target_prefix.clone());
                }
            }
        }
        for sub in &m.sub_modules {
            self.register_module(sub, prefix.clone());
        }
    }

    fn register_structured_type(&mut self, st: &StructuredType, mut prefix: QualifiedName) {
        prefix.extend(st.name.clone());
        self.paths.insert(prefix.clone());

        for method in &st.methods {
            let mut m_prefix = prefix.clone();
            m_prefix.extend(method.name.clone());
            self.paths.insert(m_prefix);
        }
        for nested in &st.nested_types {
            self.register_structured_type(nested, prefix.clone());
        }
    }

    pub fn exists(&self, path: &QualifiedName) -> bool {
        self.paths.contains(path)
    }
}

// ==================== CONTEXT ====================
pub struct ResolutionContext<'a> {
    pub current_prefix: QualifiedName,
    pub stack: &'a RefCell<SymbolStack>,
    pub cache: &'a RefCell<ResolutionCache>,
    pub registry: &'a GlobalRegistry,
    pub primitives: &'a PrimitiveRegistry,
}

// ==================== CORE ALGORITHM ====================
pub fn resolve_type_refs(modules: Vec<Module>, primitives: &PrimitiveRegistry) -> Vec<Module> {
    let registry = GlobalRegistry::build(&modules);
    let stack = RefCell::new(SymbolStack::new());
    let cache = RefCell::new(ResolutionCache::new());
    
    // We push the GLOBAL ROOT frame
    stack.borrow_mut().push_scope();

    let ctx = ResolutionContext {
        current_prefix: vec![],
        stack: &stack,
        cache: &cache,
        registry: &registry,
        primitives,
    };

    let resolved_modules = modules
        .into_iter()
        .map(|m| resolve_module_in_context(&ctx, m))
        .collect();
        
    stack.borrow_mut().pop_scope();
    
    resolved_modules
}

// ==================== RESOLUTION LOGIC ====================
fn resolve_name_in_context(
    ctx: &ResolutionContext,
    name: &QualifiedName,
) -> Option<ResolutionResult> {
    if name.is_empty() { return None; }
    let first_part = name.first().unwrap();

    // 0. Memoization Cache
    if let Some(res) = ctx.cache.borrow().get(&ctx.current_prefix, name) {
        return Some(res);
    }

    let validate_import = |candidate: &QualifiedName| -> ResolutionResult {
        if ctx.registry.exists(candidate) {
            ResolutionResult::Local(candidate.clone())
        } else if candidate.first() == Some(&"crate".to_string()) {
            // First fallback: replace "crate" with "root"
            let mut crate_cand = vec!["root".to_string()];
            crate_cand.extend(candidate.iter().skip(1).cloned());
            if ctx.registry.exists(&crate_cand) {
                ResolutionResult::Local(crate_cand)
            } else {
                ResolutionResult::External(candidate.clone())
            }
        } else {
            // Second fallback: prepending "root"
            let mut root_cand = vec!["root".to_string()];
            root_cand.extend(candidate.clone());
            if ctx.registry.exists(&root_cand) {
                ResolutionResult::Local(root_cand)
            } else {
                ResolutionResult::External(candidate.clone())
            }
        }
    };

    // 1. Lexical Climb (Symbol Stack Traversal)
    for frame in ctx.stack.borrow().iter_frames_top_down() {
        // A. Local Symbols in this frame
        if let Some(abs_path) = frame.symbols.get(first_part) {
            let mut full_path = abs_path.clone();
            full_path.extend(name.iter().skip(1).cloned());
            
            if ctx.registry.exists(&full_path) {
                let res = ResolutionResult::Local(full_path);
                ctx.cache.borrow_mut().insert(&ctx.current_prefix, name.clone(), res.clone());
                return Some(res);
            }
        }

        // B. Imports (Jump Points) in this frame
        for imp in &frame.imports {
            if let Some(alias) = &imp.alias {
                if alias == first_part {
                    let mut candidate = imp.path.clone();
                    candidate.extend(name.iter().skip(1).cloned());
                    let res = validate_import(&candidate);
                    ctx.cache.borrow_mut().insert(&ctx.current_prefix, name.clone(), res.clone());
                    return Some(res);
                }
            } else if !imp.is_wildcard {
                if let Some(last_part) = imp.path.last() {
                    if last_part == first_part {
                        let mut candidate = imp.path.clone();
                        candidate.extend(name.iter().skip(1).cloned());
                        let res = validate_import(&candidate);
                        ctx.cache.borrow_mut().insert(&ctx.current_prefix, name.clone(), res.clone());
                        return Some(res);
                    }
                }
            } else {
                // Wildcard import
                let mut candidate = imp.path.clone();
                candidate.extend(name.clone());
                let res = validate_import(&candidate);
                if let ResolutionResult::Local(_) = res {
                    ctx.cache.borrow_mut().insert(&ctx.current_prefix, name.clone(), res.clone());
                    return Some(res);
                }
            }
        }
    }

    // 2. Absolute Fallbacks
    if ctx.registry.exists(name) {
        let res = ResolutionResult::Local(name.clone());
        ctx.cache.borrow_mut().insert(&ctx.current_prefix, name.clone(), res.clone());
        return Some(res);
    }
    
    let mut root_cand = vec!["root".to_string()];
    root_cand.extend(name.clone());
    if ctx.registry.exists(&root_cand) {
        let res = ResolutionResult::Local(root_cand);
        ctx.cache.borrow_mut().insert(&ctx.current_prefix, name.clone(), res.clone());
        return Some(res);
    }

    None
}

fn resolve_type_ref(ctx: &ResolutionContext, tr: TypeRef) -> TypeRef {
    match tr {
        TypeRef::Unresolved(qn) => {
            if let Some(res) = resolve_name_in_context(ctx, &qn) {
                match res {
                    ResolutionResult::Local(abs_qn) => TypeRef::Resolved(abs_qn),
                    ResolutionResult::External(ext_qn) => TypeRef::External(ext_qn),
                }
            } else {
                // Check if it's a primitive type
                if qn.len() == 1 && ctx.primitives.is_primitive(&qn[0]) {
                    TypeRef::Primitive(PrimitiveType::Other(qn[0].clone()))
                } else {
                    TypeRef::Failed(qn)
                }
            }
        }
        _ => tr,
    }
}

// ==================== TRAVERSAL ====================

fn register_self_keywords(ctx: &ResolutionContext, class_path: &QualifiedName) {
    ctx.stack.borrow_mut().define_symbol("self".to_string(), class_path.clone());
    ctx.stack.borrow_mut().define_symbol("this".to_string(), class_path.clone());
}

fn resolve_module_in_context(ctx: &ResolutionContext, mut module: Module) -> Module {
    let mut new_prefix = ctx.current_prefix.clone();
    new_prefix.extend(module.name.clone());
    
    // Create new context with updated prefix
    let new_ctx = ResolutionContext {
        current_prefix: new_prefix.clone(),
        stack: ctx.stack,
        cache: ctx.cache,
        registry: ctx.registry,
        primitives: ctx.primitives,
    };

    // PUSH lexical scope for this module
    new_ctx.stack.borrow_mut().push_scope();

    // Hoist Imports
    for imp in &module.imports {
        new_ctx.stack.borrow_mut().add_import(imp.clone());
    }

    // Hoist local components (Structs, Functions, Sub-modules)
    for st in &module.structured_types {
        let mut st_path = new_prefix.clone();
        st_path.extend(st.name.clone());
        new_ctx.stack.borrow_mut().define_symbol(st.name.last().unwrap().clone(), st_path);
    }
    for ff in &module.free_functions {
        let mut ff_path = new_prefix.clone();
        ff_path.extend(ff.name.clone());
        new_ctx.stack.borrow_mut().define_symbol(ff.name.last().unwrap().clone(), ff_path);
    }
    for sub in &module.sub_modules {
        let mut sub_path = new_prefix.clone();
        sub_path.extend(sub.name.clone());
        new_ctx.stack.borrow_mut().define_symbol(sub.name.last().unwrap().clone(), sub_path);
    }

    module.name = new_prefix.clone();

    // Recursively resolve
    module.structured_types = module
        .structured_types
        .into_iter()
        .map(|st| resolve_structured_type(&new_ctx, st))
        .collect();

    module.free_functions = module
        .free_functions
        .into_iter()
        .map(|ff| resolve_function(&new_ctx, ff))
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
    module.impl_blocks = unfused_impls;

    module.sub_modules = module
        .sub_modules
        .into_iter()
        .map(|sub| resolve_module_in_context(&new_ctx, sub))
        .collect();

    // POP lexical scope
    new_ctx.stack.borrow_mut().pop_scope();

    module
}

fn resolve_structured_type(ctx: &ResolutionContext, mut st: StructuredType) -> StructuredType {
    let mut new_prefix = ctx.current_prefix.clone();
    new_prefix.extend(st.name.clone());
    st.name = new_prefix.clone();

    let new_ctx = ResolutionContext {
        current_prefix: new_prefix.clone(),
        stack: ctx.stack,
        cache: ctx.cache,
        registry: ctx.registry,
        primitives: ctx.primitives,
    };

    // PUSH scope for StructuredType
    new_ctx.stack.borrow_mut().push_scope();
    
    // Inject "self" and "this" to refer to the current type
    register_self_keywords(&new_ctx, &new_prefix);

    // Hoist nested components
    for nested in &st.nested_types {
        let mut n_path = new_prefix.clone();
        n_path.extend(nested.name.clone());
        new_ctx.stack.borrow_mut().define_symbol(nested.name.last().unwrap().clone(), n_path);
    }
    for m in &st.methods {
        let mut m_path = new_prefix.clone();
        m_path.extend(m.name.clone());
        new_ctx.stack.borrow_mut().define_symbol(m.name.last().unwrap().clone(), m_path);
    }

    // Resolve structural types
    st.super_types = st
        .super_types
        .into_iter()
        .map(|t| resolve_type_ref(&new_ctx, t))
        .collect();

    st.fields = st
        .fields
        .into_iter()
        .map(|mut f| {
            f.ty = resolve_type_ref(&new_ctx, f.ty);
            f
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

    // POP scope
    new_ctx.stack.borrow_mut().pop_scope();

    st
}

fn resolve_function(ctx: &ResolutionContext, mut ff: Function) -> Function {
    let mut new_prefix = ctx.current_prefix.clone();
    new_prefix.extend(ff.name.clone());
    ff.name = new_prefix.clone();

    // We do push a scope for the function (which includes parameters)
    ctx.stack.borrow_mut().push_scope();

    // Resolve Signature
    ff.signature.parameters = ff
        .signature
        .parameters
        .into_iter()
        .map(|mut p| {
            p.ty = resolve_type_ref(ctx, p.ty);
            // If the parameter has a name, define it in the function scope
            if let Some(name) = &p.name {
                if let TypeRef::Resolved(abs_path) | TypeRef::External(abs_path) = &p.ty {
                    ctx.stack.borrow_mut().define_symbol(name.clone(), abs_path.clone());
                }
            }
            p
        })
        .collect();
    ff.signature.return_type = resolve_type_ref(ctx, ff.signature.return_type);
    
    // Resolve Body
    ff.body = ff.body.map(|b| resolve_block(ctx, b));

    ctx.stack.borrow_mut().pop_scope();

    ff
}

fn resolve_block(ctx: &ResolutionContext, mut block: Block) -> Block {
    // Entering a block creates a new lexical scope
    ctx.stack.borrow_mut().push_scope();

    // 1. Resolve Declarations and Define them in the stack
    block.declarations = block.declarations.into_iter().map(|mut decl| {
        decl.ty = resolve_type_ref(ctx, decl.ty);
        if let TypeRef::Resolved(abs_path) | TypeRef::External(abs_path) = &decl.ty {
            ctx.stack.borrow_mut().define_symbol(decl.name.clone(), abs_path.clone());
        }
        decl
    }).collect();

    // 2. Resolve Behavioral Dependencies
    block.calls = block.calls.into_iter().map(|c| resolve_type_ref(ctx, c)).collect();
    block.instantiates = block.instantiates.into_iter().map(|i| resolve_type_ref(ctx, i)).collect();

    // 3. Recurse into sub-blocks
    block.sub_blocks = block.sub_blocks.into_iter().map(|b| resolve_block(ctx, b)).collect();

    ctx.stack.borrow_mut().pop_scope();
    block
}

fn resolve_impl_block(ctx: &ResolutionContext, mut i: ImplBlock) -> ImplBlock {
    let mut new_prefix = ctx.current_prefix.clone();
    new_prefix.extend(i.name.clone());
    i.name = new_prefix.clone();

    // ImplBlock creates a temporary scope
    ctx.stack.borrow_mut().push_scope();

    i.impl_for = resolve_type_ref(ctx, i.impl_for);
    if let TypeRef::Resolved(abs_path) | TypeRef::External(abs_path) = &i.impl_for {
        register_self_keywords(ctx, abs_path);
    }
    i.implements_trait = i.implements_trait.map(|t| resolve_type_ref(ctx, t));

    i.methods = i
        .methods
        .into_iter()
        .map(|m| resolve_function(ctx, m))
        .collect();

    i.nested_types = i
        .nested_types
        .into_iter()
        .map(|n| resolve_structured_type(ctx, n))
        .collect();

    ctx.stack.borrow_mut().pop_scope();
    
    i
}