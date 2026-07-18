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
    let mut sub_blocks = vec![];

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();

        // 1. Variable Declarations
        if matches!(
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
        ) {
            let name_opt;
            if let Some(decl_node) = child.child_by_field_name("declarator") {
                if let Some(name_node) = decl_node.child_by_field_name("name") {
                    name_opt = extract_identifier(name_node, source);
                } else {
                    name_opt = extract_identifier(decl_node, source);
                }
            } else if let Some(name_node) = child.child_by_field_name("name") {
                name_opt = extract_identifier(name_node, source);
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
                declarations.push(Field { name, ty });
            }
            
            // WE MUST ALSO check the right-hand side (initializer/value) for behavioral deps!
            let mut inner_calls = vec![];
            let mut inner_inst = vec![];
            let mut inner_accesses = vec![];
            if let Some(val) = child.child_by_field_name("value").or_else(|| child.child_by_field_name("declarator")) {
                find_behavioral_deps(val, source, &mut inner_calls, &mut inner_inst, &mut inner_accesses);
            } else {
                // If there's no clear value field, scan the whole declaration just in case (excluding the type/name to avoid false positives)
                // Actually, find_behavioral_deps is safe to run on the whole node because it looks for call_expression / new_expression
                find_behavioral_deps(child, source, &mut inner_calls, &mut inner_inst, &mut inner_accesses);
            }
            calls.extend(inner_calls);
            instantiates.extend(inner_inst);
            accesses.extend(inner_accesses);
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
            find_behavioral_deps(child, source, &mut inner_calls, &mut inner_inst, &mut inner_accesses);
            calls.extend(inner_calls);
            instantiates.extend(inner_inst);
            accesses.extend(inner_accesses);
        }
    }

    crate::model::Block {
        declarations,
        calls,
        instantiates,
        accesses,
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
    } else if kind == "struct_expression" {
        if let Some(name_node) = node.child_by_field_name("name") {
            instantiates.push(extract_type_ref(name_node, source));
        }
    }

    // --- Calls ---
    if matches!(kind, "call_expression" | "call") {
        extract_call_dependency(node, source, calls);
    } else if kind == "method_invocation" {
        // Java
        let mut parts = vec![];
        if let Some(obj) = node.child_by_field_name("object") {
            if let TypeRef::Unresolved(qn) = extract_type_ref(obj, source) {
                parts.extend(qn);
            }
        }
        if let Some(name) = node.child_by_field_name("name") {
            if let TypeRef::Unresolved(qn) = extract_type_ref(name, source) {
                parts.extend(qn);
            }
        }
        if !parts.is_empty() {
            calls.push(TypeRef::Unresolved(parts));
        }
    }

    // --- Accesses ---
    if matches!(kind, "field_access" | "member_expression" | "property_identifier" | "member_access" | "identifier" | "field_expression") {
        accesses.push(extract_type_ref(node, source));
    }

    // --- Token Tree Coalescing (e.g. for Rust macros or generic unparsed blocks) ---
    if kind == "token_tree" {
        parse_token_tree_macro(node, source, calls, instantiates, accesses);
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_behavioral_deps(child, source, calls, instantiates, accesses);
    }
}

/// Provides a "best-effort" type inference for implicitly typed local variables (like `let x = ...` or `auto y = ...`).
///
/// It inspects the right-hand side of an assignment (`value` field or direct children). If the 
/// assignment stems from an explicit instantiation (e.g. `new_expression`), it extracts the target 
/// class and deduces the variable's type.
pub fn infer_variable_type(node: Node, source: &str) -> TypeRef {
    // 1. If it has a explicit "value" field (like Rust let_declaration)
    if let Some(val) = node.child_by_field_name("value") {
        if matches!(val.kind(), "object_creation_expression" | "new_expression") {
            if let Some(t_node) = val.child_by_field_name("type") {
                return extract_type_ref(t_node, source);
            }
        } else if val.kind() == "struct_expression" {
            if let Some(name_node) = val.child_by_field_name("name") {
                return extract_type_ref(name_node, source);
            }
        }
        // It could just be an identifier (e.g. let x = Factory;)
        let text = node_text(val, source);
        if !text.is_empty() && text.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ':') {
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
) {
    if let Some(f) = node.child_by_field_name("function") {
        let f_kind = f.kind();
        if matches!(
            f_kind,
            "qualified_identifier" | "scoped_identifier" | "field_expression" | "attribute" | "identifier" | "type_identifier"
        ) {
            calls.push(extract_type_ref(f, source));
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
) {
    let mut current_path: Vec<String> = vec![];
    let mut expect_ident = true;

    let mut i = 0;
    let count = node.child_count();
    while i < count {
        let child = node.child(i as u32).unwrap();
        let c_kind = child.kind();
        
        if expect_ident && (c_kind == "identifier" || c_kind == "type_identifier" || c_kind == "scoped_identifier") {
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
            find_behavioral_deps(child, source, calls, instantiates, accesses);
        }
        i += 1;
    }
    
    if !current_path.is_empty() {
        accesses.push(TypeRef::Unresolved(current_path));
    }
}