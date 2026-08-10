//! Responsible for deeply analyzing the internal behavior of methods and functions.
//!
//! This module delves into function bodies, parsing `{}` blocks recursively to extract
//! local variable declarations, instantiated types, and method calls. It powers the
//! Behavioral Lexical Scoping engine.

use crate::model::{Field, TypeRef};
use tree_sitter::Node;

use super::text_parsing::{extract_identifier, node_text, split_qualified_name};
use super::type_extraction::extract_type_ref;

/// Recursively traverses a compound statement or block node to abstract its hierarchical structure.
///
/// Captures all local variable declarations (reusing the `Field` struct to map `name` to `type`),
/// discovers behavioral dependencies (calls and instantiations), and recursively extracts
/// sub-blocks (e.g., within `if` or `while` statements) to preserve exact scope boundaries.
///
/// # Arguments
/// * `node` - The block-like AST node to parse.
/// * `source` - The complete source code string.
pub fn extract_block(node: Node, source: &str) -> crate::model::Block {
    let mut declarations = vec![];
    let mut calls = vec![];
    let mut instantiates = vec![];
    let mut accesses = vec![];
    let mut type_casts = vec![];
    let mut sub_blocks = vec![];

    let mut cursor = node.walk();
    for mut child in node.children(&mut cursor) {
        let mut kind = child.kind();

        if kind == "expression_statement"
            && let Some(inner) = child.child(0)
        {
            child = inner;
            kind = child.kind();
        }

        let mut is_declaration = matches!(
            kind,
            "field_declaration"
                | "property_declaration"
                | "field"
                | "member_declaration"
                | "variable_declarator"
                | "attribute"
                | "local_variable_declaration"
                | "lexical_declaration"
                | "variable_declaration"
                | "let_declaration"
                | "assignment"
                | "declaration"
                | "for_range_loop"
                | "for_in_statement"
                | "catch_clause"
        );

        if kind == "assignment" {
            if let Some(left) = child.child_by_field_name("left") {
                if matches!(left.kind(), "attribute" | "field_expression" | "subscript_expression" | "member_expression") {
                    is_declaration = false;
                }
            }
        }

        // 1. Variable Declarations
        if is_declaration {
            let name_opt;
            if let Some(decl_node) = child
                .child_by_field_name("declarator")
                .or_else(|| child.child_by_field_name("left"))
            {
                if let Some(name_node) = decl_node.child_by_field_name("name") {
                    name_opt = extract_identifier(name_node, source);
                } else {
                    name_opt = extract_identifier(decl_node, source);
                }
            } else if let Some(name_node) = child.child_by_field_name("name") {
                name_opt = extract_identifier(name_node, source);
            } else if let Some(left_node) = child.child_by_field_name("left") {
                name_opt = extract_identifier(left_node, source);
            } else {
                name_opt = extract_identifier(child, source);
            }

            if let Some(name) = name_opt {
                // For declarations, check if there is an explicit type.
                // Using extract_type_ref on the whole declaration node can falsely extract the variable name as type.
                let mut ty = if let Some(type_node) = child.child_by_field_name("type") {
                    extract_type_ref(type_node, source)
                } else {
                    infer_variable_type(child, source)
                };

                if matches!(ty, TypeRef::Failed(_)) {
                    // Fallback to old behavior if everything fails
                    let extracted = extract_type_ref(child, source);
                    if !matches!(extracted, TypeRef::Failed(_)) {
                        // Check if the extracted "type" is just the variable name itself,
                        // if so it's a false positive from the identifier fallback
                        if extracted != TypeRef::Unresolved(vec![name.clone()]) {
                            ty = extracted;
                        }
                    }
                }
                let annotations = super::annotation_extraction::extract_annotations(child, source);
                declarations.push(Field {
                    name,
                    ty,
                    annotations,
                });
            }

            // WE MUST ALSO check the right-hand side (initializer/value) for behavioral deps!
            let mut inner_calls = vec![];
            let mut inner_inst = vec![];
            let mut inner_accesses = vec![];
            let mut inner_casts = vec![];
            if let Some(val) = child
                .child_by_field_name("value")
                .or_else(|| child.child_by_field_name("right"))
                .or_else(|| child.child_by_field_name("declarator"))
            {
                find_behavioral_deps(
                    val,
                    source,
                    &mut inner_calls,
                    &mut inner_inst,
                    &mut inner_accesses,
                    &mut inner_casts,
                );
            } else {
                // If there's no clear value field, scan the whole declaration just in case (excluding the type/name to avoid false positives)
                // Actually, find_behavioral_deps is safe to run on the whole node because it looks for call_expression / new_expression
                find_behavioral_deps(
                    child,
                    source,
                    &mut inner_calls,
                    &mut inner_inst,
                    &mut inner_accesses,
                    &mut inner_casts,
                );
            }
            calls.extend(inner_calls);
            instantiates.extend(inner_inst);
            accesses.extend(inner_accesses);
            type_casts.extend(inner_casts);

            // Recurse into body/block if it's a loop or catch clause
            if child.kind() == "for_range_loop"
                || child.kind() == "for_in_statement"
                || child.kind() == "catch_clause"
            {
                let mut gcursor = child.walk();
                for grandchild in child.children(&mut gcursor) {
                    if grandchild.kind().contains("body")
                        || grandchild.kind().contains("block")
                        || grandchild.kind() == "compound_statement"
                    {
                        sub_blocks.push(extract_block(grandchild, source));
                    }
                }
            }
        }
        // 2. Nested Blocks
        else if kind.contains("body") || kind.contains("block") || kind == "compound_statement" {
            sub_blocks.push(extract_block(child, source));
        }
        // 3. Behavioral Deps (recursive search within current level, avoiding deep blocks)
        else {
            let mut inner_calls = vec![];
            let mut inner_inst = vec![];
            let mut inner_accesses = vec![];
            let mut inner_casts = vec![];
            find_behavioral_deps(
                child,
                source,
                &mut inner_calls,
                &mut inner_inst,
                &mut inner_accesses,
                &mut inner_casts,
            );
            calls.extend(inner_calls);
            instantiates.extend(inner_inst);
            accesses.extend(inner_accesses);
            type_casts.extend(inner_casts);
        }
    }

    crate::model::Block {
        declarations,
        calls,
        instantiates,
        accesses,
        type_casts,
        sub_blocks,
    }
}

/// Iterates over statements within a block to intercept any behavioral actions
/// (e.g. `object_creation_expression` for instantiations, `call_expression` for method calls).
/// It skips nested blocks, deferring their parsing to recursive `extract_block` calls.
fn find_behavioral_deps(
    node: Node,
    source: &str,
    calls: &mut Vec<TypeRef>,
    instantiates: &mut Vec<TypeRef>,
    accesses: &mut Vec<TypeRef>,
    type_casts: &mut Vec<TypeRef>,
) {
    let kind = node.kind();

    // Skip nested blocks to avoid double counting (they are handled by extract_block)
    if kind.contains("body") || kind.contains("block") || kind == "compound_statement" {
        return;
    }

    // --- Instantiations ---
    if matches!(kind, "object_creation_expression" | "new_expression") {
        if let Some(t_node) = node.child_by_field_name("type") {
            instantiates.push(extract_type_ref(t_node, source));
        }
    } else if kind == "struct_expression"
        && let Some(name_node) = node.child_by_field_name("name")
    {
        instantiates.push(extract_type_ref(name_node, source));
    }

    // --- Calls ---
    if matches!(kind, "call_expression" | "call") {
        extract_call_dependency(node, source, calls, accesses);
    } else if kind == "method_invocation" {
        // Java
        let mut parts = vec![];
        if let Some(obj) = node.child_by_field_name("object")
            && let TypeRef::Unresolved(qn) = extract_type_ref(obj, source)
        {
            parts.extend(qn);
        }
        if let Some(name) = node.child_by_field_name("name")
            && let TypeRef::Unresolved(qn) = extract_type_ref(name, source)
        {
            parts.extend(qn);
        }
        if !parts.is_empty() {
            calls.push(TypeRef::Unresolved(parts));
        }
    }

    // --- Accesses ---
    if matches!(
        kind,
        "field_access"
            | "member_expression"
            | "property_identifier"
            | "member_access"
            | "identifier"
            | "scoped_identifier"
            | "qualified_identifier"
            | "field_expression"
            | "attribute"
    ) {
        accesses.push(extract_type_ref(node, source));
    }

    // --- Type Casts ---
    if matches!(kind, "cast_expression" | "type_cast_expression")
        && let Some(type_node) = node.child_by_field_name("type")
    {
        let ty = extract_type_ref(type_node, source);
        type_casts.push(ty);
    }

    // --- Token Tree Coalescing (e.g. for Rust macros or generic unparsed blocks) ---
    if kind == "token_tree" {
        parse_token_tree_macro(node, source, calls, instantiates, accesses, type_casts);
        return;
    }

    // Do not recurse into compound identifiers or types to avoid spurious accesses for their parts
    if matches!(
        kind,
        "scoped_identifier" | "qualified_identifier" | "field_access" | "member_expression"
    ) {
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        // Skip recursing into the 'function' part of a call, because we already extracted it as a Call.
        if matches!(kind, "call_expression" | "call" | "method_invocation") {
            if let Some(f_node) = node
                .child_by_field_name("function")
                .or_else(|| node.child_by_field_name("name"))
            {
                if child.id() == f_node.id() {
                    continue;
                }
            }
        }

        find_behavioral_deps(child, source, calls, instantiates, accesses, type_casts);
    }
}

/// Provides a "best-effort" type inference for implicitly typed local variables (like `let x = ...` or `auto y = ...`).
///
/// It inspects the right-hand side of an assignment (`value` field or direct children). If the
/// assignment stems from an explicit instantiation (e.g. `new_expression`), it extracts the target
/// class and deduces the variable's type.
pub fn infer_variable_type(node: Node, source: &str) -> TypeRef {
    // 1. If it has a explicit "value" or "right" field (like Rust let_declaration or Python assignment)
    if let Some(val) = node
        .child_by_field_name("value")
        .or_else(|| node.child_by_field_name("right"))
    {
        if matches!(val.kind(), "object_creation_expression" | "new_expression") {
            if let Some(t_node) = val.child_by_field_name("type") {
                return extract_type_ref(t_node, source);
            }
        } else if val.kind() == "struct_expression" {
            if let Some(name_node) = val.child_by_field_name("name") {
                return extract_type_ref(name_node, source);
            }
        } else if val.kind() == "call" {
            // In languages like Python, object creation is just a call node (e.g. `Admin(...)`)
            if let Some(f_node) = val.child_by_field_name("function") {
                let extracted = extract_type_ref(f_node, source);
                if let crate::model::TypeRef::Unresolved(path) = &extracted {
                    if !path.is_empty() {
                        let mut curr = crate::model::Query::Find(path[0].clone());
                        for part in &path[1..] {
                            curr = crate::model::Query::Extract(Box::new(curr), part.clone());
                        }
                        let query = crate::model::Query::Call(Box::new(curr));
                        return crate::model::TypeRef::ResolutionQuery(query);
                    }
                }
                return extracted;
            }
        }
        // It could just be an identifier (e.g. let x = Factory;)
        let text = node_text(val, source);
        if !text.is_empty()
            && text
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == ':')
        {
            return TypeRef::Unresolved(split_qualified_name(&text));
        }
    }

    // 2. Fallback: Search children for initializers
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        if matches!(kind, "object_creation_expression" | "new_expression") {
            if let Some(t_node) = child.child_by_field_name("type") {
                return extract_type_ref(t_node, source);
            }
        } else if kind == "struct_expression" {
            if let Some(name_node) = child.child_by_field_name("name") {
                return extract_type_ref(name_node, source);
            }
        } else if kind == "call" {
            if let Some(f_node) = child.child_by_field_name("function") {
                let extracted = extract_type_ref(f_node, source);
                if let crate::model::TypeRef::Unresolved(path) = &extracted {
                    if !path.is_empty() {
                        let mut curr = crate::model::Query::Find(path[0].clone());
                        for part in &path[1..] {
                            curr = crate::model::Query::Extract(Box::new(curr), part.clone());
                        }
                        let query = crate::model::Query::Call(Box::new(curr));
                        return crate::model::TypeRef::ResolutionQuery(query);
                    }
                }
                return extracted;
            }
        }
    }
    TypeRef::Failed(vec![])
}

/// Extracts a method or function call dependency from a call expression node.
///
/// It correctly handles `qualified_identifier` and `scoped_identifier` nodes by
/// extracting the full path to the function being called, ensuring that static
/// method calls (e.g., `StructA::static_method`) are correctly resolved instead of just their scope.
fn extract_call_dependency(
    node: Node,
    source: &str,
    calls: &mut Vec<TypeRef>,
    accesses: &mut Vec<TypeRef>,
) {
    if let Some(f) = node.child_by_field_name("function") {
        let f_kind = f.kind();
        if matches!(
            f_kind,
            "qualified_identifier"
                | "scoped_identifier"
                | "field_expression"
                | "attribute"
                | "identifier"
                | "type_identifier"
        ) {
            calls.push(extract_type_ref(f, source));
            
            // Also extract the base object of the method call as an access (e.g. `self.permissions` in `self.permissions.append()`)
            if let Some(obj) = f.child_by_field_name("object")
                .or_else(|| f.child_by_field_name("value"))
                .or_else(|| f.child_by_field_name("left"))
                .or_else(|| f.child_by_field_name("argument")) // C++ field_expression uses "argument"
            {
                accesses.push(extract_type_ref(obj, source));
            }
        }
    }
}

/// Extracts behavioral dependencies (calls and accesses) from a `token_tree` node.
///
/// In Tree-sitter (especially for Rust macros like `println!`), `token_tree` nodes
/// contain a flat list of tokens without semantic grouping. This function implements
/// a state machine (Token Coalescing) to reconstruct qualified paths
/// (e.g., `StructA::method().x`) and correctly categorize them as method calls or field accesses.
fn parse_token_tree_macro(
    node: Node,
    source: &str,
    calls: &mut Vec<TypeRef>,
    instantiates: &mut Vec<TypeRef>,
    accesses: &mut Vec<TypeRef>,
    type_casts: &mut Vec<TypeRef>,
) {
    let mut current_path: Vec<String> = vec![];
    let mut expect_ident = true;

    let mut i = 0;
    let count = node.child_count();
    while i < count {
        let child = node.child(i as u32).unwrap();
        let c_kind = child.kind();

        if expect_ident
            && (c_kind == "identifier"
                || c_kind == "type_identifier"
                || c_kind == "scoped_identifier")
        {
            let text = node_text(child, source);
            if c_kind == "scoped_identifier" {
                current_path.extend(split_qualified_name(&text));
            } else {
                current_path.push(text);
            }
            expect_ident = false;
        } else if !expect_ident && (c_kind == "." || c_kind == "::") {
            expect_ident = true;
        } else if !expect_ident && c_kind == "token_tree" {
            calls.push(TypeRef::Unresolved(current_path.clone()));
            // Path does not break on method calls if followed by .
        } else {
            if current_path.len() > 1 {
                accesses.push(TypeRef::Unresolved(current_path.clone()));
            } else if current_path.len() == 1 {
                accesses.push(TypeRef::Unresolved(current_path.clone()));
            }

            current_path.clear();
            expect_ident = true;

            // Recurse into this child
            find_behavioral_deps(child, source, calls, instantiates, accesses, type_casts);
        }
        i += 1;
    }

    if !current_path.is_empty() {
        accesses.push(TypeRef::Unresolved(current_path));
    }
}
