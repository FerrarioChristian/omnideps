use std::cell::RefCell;
use crate::model::*;
use super::stack::SymbolStack;

/// Context for the Query Building phase (Lexical Substitution).
pub struct BuilderContext<'a> {
    pub stack: &'a RefCell<SymbolStack>,
}

/// Transforms Unresolved references into mathematical Queries using Lexical Substitution.
pub fn build_queries(modules: Vec<Module>) -> Vec<Module> {
    let stack = RefCell::new(SymbolStack::new());
    
    // We push the GLOBAL ROOT frame
    stack.borrow_mut().push_scope();

    let ctx = BuilderContext {
        stack: &stack,
    };

    let processed_modules = modules
        .into_iter()
        .map(|m| build_module_queries(&ctx, m))
        .collect();

    stack.borrow_mut().pop_scope();
    processed_modules
}

fn build_module_queries(ctx: &BuilderContext, mut module: Module) -> Module {
    ctx.stack.borrow_mut().push_scope();

    // 1. Hoist imports as substitutions?
    // Wait, the professor said "find" ascends the tree. 
    // If we handle imports in the Executor phase, the Builder doesn't need them!
    // But local variables MUST be registered.
    // What about `self` and `this`? They are just local symbols pointing to the current class.
    
    module.structured_types = module.structured_types.into_iter().map(|st| build_structured_type_queries(ctx, st)).collect();
    module.free_functions = module.free_functions.into_iter().map(|ff| build_function_queries(ctx, ff)).collect();
    module.impl_blocks = module.impl_blocks.into_iter().map(|ib| build_impl_block_queries(ctx, ib)).collect();
    module.sub_modules = module.sub_modules.into_iter().map(|sub| build_module_queries(ctx, sub)).collect();

    ctx.stack.borrow_mut().pop_scope();
    module
}

fn build_structured_type_queries(ctx: &BuilderContext, mut st: StructuredType) -> StructuredType {
    ctx.stack.borrow_mut().push_scope();
    
    // Inject self and this
    let self_query = Query::Find(st.name.last().unwrap_or(&"".to_string()).clone());
    ctx.stack.borrow_mut().define_symbol("self".to_string(), self_query.clone());
    ctx.stack.borrow_mut().define_symbol("this".to_string(), self_query);

    st.super_types = st.super_types.into_iter().map(|t| substitute_type(ctx, t, false)).collect();
    st.fields = st.fields.into_iter().map(|mut f| {
        f.ty = substitute_type(ctx, f.ty, false);
        f
    }).collect();
    st.methods = st.methods.into_iter().map(|m| build_function_queries(ctx, m)).collect();
    st.nested_types = st.nested_types.into_iter().map(|n| build_structured_type_queries(ctx, n)).collect();

    ctx.stack.borrow_mut().pop_scope();
    st
}

fn build_function_queries(ctx: &BuilderContext, mut ff: Function) -> Function {
    ctx.stack.borrow_mut().push_scope();

    ff.signature.parameters = ff.signature.parameters.into_iter().map(|mut p| {
        p.ty = substitute_type(ctx, p.ty, false);
        if let Some(name) = &p.name {
            if let TypeRef::ResolutionQuery(ref q) = p.ty {
                ctx.stack.borrow_mut().define_symbol(name.clone(), q.clone());
            }
        }
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

    block.calls = block.calls.into_iter().map(|c| substitute_type(ctx, c, true)).collect();
    block.instantiates = block.instantiates.into_iter().map(|i| substitute_type(ctx, i, false)).collect();
    block.accesses = block.accesses.into_iter().map(|a| substitute_type(ctx, a, false)).collect();
    
    block.sub_blocks = block.sub_blocks.into_iter().map(|b| build_block_queries(ctx, b)).collect();

    ctx.stack.borrow_mut().pop_scope();
    block
}

fn build_impl_block_queries(ctx: &BuilderContext, mut ib: ImplBlock) -> ImplBlock {
    ctx.stack.borrow_mut().push_scope();

    ib.impl_for = substitute_type(ctx, ib.impl_for, false);
    if let TypeRef::ResolutionQuery(ref q) = ib.impl_for {
        ctx.stack.borrow_mut().define_symbol("self".to_string(), q.clone());
        ctx.stack.borrow_mut().define_symbol("this".to_string(), q.clone());
    }
    ib.implements_trait = ib.implements_trait.map(|t| substitute_type(ctx, t, false));
    ib.methods = ib.methods.into_iter().map(|m| build_function_queries(ctx, m)).collect();
    ib.nested_types = ib.nested_types.into_iter().map(|n| build_structured_type_queries(ctx, n)).collect();

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