use std::cell::RefCell;
use crate::model::*;
use super::stack::SymbolStack;

/// Context for the Query Building phase (Lexical Substitution).
pub struct BuilderContext<'a> {
    pub stack: &'a RefCell<SymbolStack>,
    pub config: &'a crate::config::AnalyzerConfig,
}

/// Transforms Unresolved references into mathematical Queries using Lexical Substitution.
pub fn build_queries(modules: Vec<Module>, config: &crate::config::AnalyzerConfig) -> Vec<Module> {
    let stack = RefCell::new(SymbolStack::new());
    
    // We push the GLOBAL ROOT frame
    stack.borrow_mut().push_scope();

    let ctx = BuilderContext {
        stack: &stack,
        config,
    };

    let processed_modules = modules
        .into_iter()
        .map(|m| {
            let lang = m.language.clone();
            build_module_queries(&ctx, lang, m)
        })
        .collect();

    stack.borrow_mut().pop_scope();
    processed_modules
}

fn build_module_queries(ctx: &BuilderContext, current_lang: Option<String>, mut module: Module) -> Module {
    ctx.stack.borrow_mut().push_scope();

    // 1. Hoist imports as substitutions?
    // Wait, the professor said "find" ascends the tree. 
    // If we handle imports in the Executor phase, the Builder doesn't need them!
    // But local variables MUST be registered.
    // What about `self` and `this`? They are just local symbols pointing to the current class.
    
    module.structured_types = module.structured_types.into_iter().map(|st| build_structured_type_queries(ctx, current_lang.clone(), st)).collect();
    module.free_functions = module.free_functions.into_iter().map(|ff| build_function_queries(ctx, ff, None, false)).collect();
    module.impl_blocks = module.impl_blocks.into_iter().map(|ib| build_impl_block_queries(ctx, current_lang.clone(), ib)).collect();
    module.sub_modules = module.sub_modules.into_iter().map(|sub| {
        let lang = sub.language.clone().or(current_lang.clone());
        build_module_queries(ctx, lang, sub)
    }).collect();

    ctx.stack.borrow_mut().pop_scope();
    module
}

fn build_structured_type_queries(ctx: &BuilderContext, current_lang: Option<String>, mut st: StructuredType) -> StructuredType {
    ctx.stack.borrow_mut().push_scope();
    
    // Inject self keyword according to language configuration
    let self_query = Query::Find(st.name.last().unwrap_or(&"".to_string()).clone());
    let lang_config = ctx.config.get_for(current_lang.as_deref().unwrap_or(""));
    if let Some(kw) = &lang_config.self_keyword {
        ctx.stack.borrow_mut().define_symbol(kw.clone(), self_query.clone());
    }

    st.super_types = st.super_types.into_iter().map(|t| substitute_type(ctx, t, false)).collect();
    st.fields = st.fields.into_iter().map(|mut f| {
        f.ty = substitute_type(ctx, f.ty, false);
        f
    }).collect();
    st.methods = st.methods.into_iter().map(|m| build_function_queries(ctx, m, Some(self_query.clone()), lang_config.implicit_first_param_as_self)).collect();
    st.nested_types = st.nested_types.into_iter().map(|n| build_structured_type_queries(ctx, current_lang.clone(), n)).collect();

    ctx.stack.borrow_mut().pop_scope();
    st
}

fn build_function_queries(ctx: &BuilderContext, mut ff: Function, self_query: Option<Query>, implicit_first_param_as_self: bool) -> Function {
    ctx.stack.borrow_mut().push_scope();

    let mut is_first = true;
    ff.signature.parameters = ff.signature.parameters.into_iter().map(|mut p| {
        p.ty = substitute_type(ctx, p.ty, false);
        if let Some(name) = &p.name {
            if is_first && implicit_first_param_as_self {
                if let Some(sq) = &self_query {
                    ctx.stack.borrow_mut().define_symbol(name.clone(), sq.clone());
                }
            } else if let TypeRef::ResolutionQuery(ref q) = p.ty {
                ctx.stack.borrow_mut().define_symbol(name.clone(), q.clone());
            }
        }
        is_first = false;
        p
    }).collect();

    ff.signature.return_type = substitute_type(ctx, ff.signature.return_type, false);

    ff.body = ff.body.map(|b| build_block_queries(ctx, b));

    ctx.stack.borrow_mut().pop_scope();
    ff
}

fn build_block_queries(ctx: &BuilderContext, mut block: Block) -> Block {
    ctx.stack.borrow_mut().push_scope();

    block.declarations = block.declarations.into_iter().map(|mut decl| {
        decl.ty = substitute_type(ctx, decl.ty, false);
        if let TypeRef::ResolutionQuery(ref q) = decl.ty {
            ctx.stack.borrow_mut().define_symbol(decl.name.clone(), q.clone());
        }
        decl
    }).collect();

    block.calls = block.calls.into_iter().map(|c| substitute_type(ctx, c, false)).collect();
    block.instantiates = block.instantiates.into_iter().map(|i| substitute_type(ctx, i, false)).collect();
    block.accesses = block.accesses.into_iter().map(|a| substitute_type(ctx, a, false)).collect();
    
    block.sub_blocks = block.sub_blocks.into_iter().map(|b| build_block_queries(ctx, b)).collect();

    ctx.stack.borrow_mut().pop_scope();
    block
}

fn build_impl_block_queries(ctx: &BuilderContext, current_lang: Option<String>, mut ib: ImplBlock) -> ImplBlock {
    ctx.stack.borrow_mut().push_scope();

    ib.impl_for = substitute_type(ctx, ib.impl_for, false);
    let lang_config = ctx.config.get_for(current_lang.as_deref().unwrap_or(""));
    let mut self_query = None;
    if let TypeRef::ResolutionQuery(ref q) = ib.impl_for {
        self_query = Some(q.clone());
        if let Some(kw) = &lang_config.self_keyword {
            ctx.stack.borrow_mut().define_symbol(kw.clone(), q.clone());
        }
    }
    ib.implements_trait = ib.implements_trait.map(|t| substitute_type(ctx, t, false));
    ib.methods = ib.methods.into_iter().map(|m| build_function_queries(ctx, m, self_query.clone(), lang_config.implicit_first_param_as_self)).collect();
    ib.nested_types = ib.nested_types.into_iter().map(|n| build_structured_type_queries(ctx, current_lang.clone(), n)).collect();

    ctx.stack.borrow_mut().pop_scope();
    ib
}

/// Performs Lexical Substitution.
fn substitute_type(ctx: &BuilderContext, tr: TypeRef, is_call: bool) -> TypeRef {
    match tr {
        TypeRef::Unresolved(qn) if !qn.is_empty() => {
            let first = &qn[0];
            
            // Check if the first part is a local variable in the stack
            let mut base_query = if let Some(local_query) = ctx.stack.borrow().frames.iter().rev().find_map(|f| f.symbols.get(first)) {
                local_query.clone()
            } else {
                Query::Find(first.clone())
            };

            for part in qn.into_iter().skip(1) {
                base_query = Query::Extract(Box::new(base_query), part);
            }

            if is_call {
                base_query = Query::Call(Box::new(base_query));
            }

            TypeRef::ResolutionQuery(base_query)
        }
        _ => tr,
    }
}