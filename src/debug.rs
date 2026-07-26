//! Provides debugging utilities to inspect the final state of the Name Resolution phase.
//!
//! This module contains a recursive visitor that traverses the resolved Intermediate Representation (IR)
//! and prints a clear, human-readable report of every `TypeRef` encountered (e.g. `Resolved`, `Failed`, `External`),
//! alongside its architectural context (e.g. `SuperType`, `Param`, `LocalVar`).

use crate::model::*;

/// Traverses a list of parsed and resolved modules, printing a formatted report
/// of all type references found within their nested components.
///
/// # Arguments
/// * `modules` - A slice of resolved `Module` components to inspect.
pub fn print_references(modules: &[Module]) {
    println!("\n=== ELENCO RIFERIMENTI ===");
    for m in modules {
        visit_module(m);
    }
}

/// Recursively visits a module's contents (sub-modules, structured types, and free functions).
fn visit_module(m: &Module) {
    for st in &m.structured_types {
        visit_structured_type(st);
    }
    for ff in &m.free_functions {
        visit_function(ff, &m.name.join("::"));
    }
    for sub in &m.sub_modules {
        visit_module(sub);
    }
}

/// Visits a structured type (Class, Struct, Interface, Trait), printing references
/// for its super-types, fields, and recursively visiting its methods and nested types.
fn visit_structured_type(st: &StructuredType) {
    let context = st.name.join("::");
    for sup in &st.super_types {
        print_ref("SuperType", &context, sup);
    }
    for f in &st.fields {
        print_ref(&format!("Field '{}'", f.name), &context, &f.ty);
    }
    for method in &st.methods {
        visit_function(method, &context);
    }
    for nested in &st.nested_types {
        visit_structured_type(nested);
    }
}

/// Visits a function or method, printing references for its parameters and return type,
/// and recursively visiting its behavioral body block.
fn visit_function(f: &Function, context: &str) {
    let fn_context = format!("{}::{}", context, f.name.last().unwrap_or(&"".to_string()));
    for p in &f.signature.parameters {
        print_ref(
            &format!("Param '{}'", p.name.as_deref().unwrap_or("?")),
            &fn_context,
            &p.ty,
        );
    }
    print_ref("Return", &fn_context, &f.signature.return_type);

    if let Some(body) = &f.body {
        visit_block(body, &fn_context);
    }
}

/// Recursively visits a block's statements, printing references for local variable
/// declarations, method calls, instantiations, and exploring nested sub-blocks.
fn visit_block(b: &Block, context: &str) {
    for decl in &b.declarations {
        print_ref(&format!("LocalVar '{}'", decl.name), context, &decl.ty);
    }
    for call in &b.calls {
        print_ref("Call", context, call);
    }
    for inst in &b.instantiates {
        print_ref("Instantiates", context, inst);
    }
    for sub in &b.sub_blocks {
        visit_block(sub, context);
    }
}

/// Helper function that formats and prints a single `TypeRef` state to the standard output.
fn print_ref(kind: &str, context: &str, tr: &TypeRef) {
    let (state, text) = match tr {
        TypeRef::Primitive(p) => ("PRIMITIVE", format!("{:?}", p)),
        TypeRef::Unresolved(q) => ("⚠️ UNRESOLVED", q.join("::")),
        TypeRef::ResolutionQuery(q) => ("🔍 QUERY", format!("{:?}", q)),
        TypeRef::Resolved(q) => ("✅ RESOLVED", q.join("::")),
        TypeRef::External(q) => ("🌐 EXTERNAL", q.join("::")),
        TypeRef::Failed(q) => ("❌ FAILED", q.join("::")),
        TypeRef::Union(types) => ("🔀 UNION", format!("{} variants", types.len())),
    };
    println!("[{:^12}] {:<30} | {} ({})", state, kind, text, context);
}
