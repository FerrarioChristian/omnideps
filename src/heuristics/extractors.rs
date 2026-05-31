use crate::ir::{Field, Function, Parameter, QualifiedName, StructuredType, StructuredTypeKind, TypeRef};
use tree_sitter::Node;
use crate::heuristics::parsers::{try_parse_function, try_parse_structured_type};

// ==================== TEXT HELPERS ====================

/// Extracts the raw UTF-8 text from a Tree-sitter node.
pub fn node_text(node: Node, source: &str) -> String {
    node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
}

/// Helper to split "std::vector" or "Outer.Inner" into a QualifiedName vector.
pub fn split_qualified_name(text: &str) -> QualifiedName {
    // Rimuoviamo i generic per la name resolution di base
    let text = text.split('<').next().unwrap_or(text);
    
    text.split(&[':', '.'][..])
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect()
}

/// Helper to extract a name from a textual string.
pub fn extract_name_from_text(text: &str) -> Option<QualifiedName> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(split_qualified_name(trimmed))
}

// ==================== ABSTRACT EXTRACTION HELPERS ====================

/// Tries to extract a simple identifier (string) from common child field names.
pub fn extract_identifier(node: Node, source: &str) -> Option<String> {
    if let Some(n) = node
        .child_by_field_name("name")
        .or_else(|| node.child_by_field_name("type_identifier"))
        .or_else(|| node.child_by_field_name("identifier"))
        .or_else(|| node.child_by_field_name("pattern"))
    {
        let text = node_text(n, source).trim().to_string();
        if !text.is_empty() { return Some(text); }
    }

    if let Some(decl) = node.child_by_field_name("declarator") {
        if let Some(n) = decl.child_by_field_name("declarator") {
            let text = node_text(n, source).trim().to_string();
            if !text.is_empty() { return Some(text); }
        }
        let text = node_text(decl, source).trim().to_string();
        // Take just the name before '('
        let name_part = text.split('(').next().unwrap_or("").trim().to_string();
        if !name_part.is_empty() { return Some(name_part); }
    }

    None
}

/// Extracts a qualified name (e.g. A::B::C) by traversing identifiers.
pub fn extract_qualified_name(node: Node, source: &str) -> Option<QualifiedName> {
    // Try with Tree-sitter fields first (more precise)
    if let Some(n) = node
        .child_by_field_name("name")
        .or_else(|| node.child_by_field_name("type_identifier"))
        .or_else(|| node.child_by_field_name("identifier"))
        .or_else(|| node.child_by_field_name("pattern"))
    {
        let text = &node_text(n, source);
        return Some(split_qualified_name(text));
    }

    // Fallback: search for a sequence of identifiers separated by :: or .
    let mut parts = vec![];
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "identifier" | "type_identifier" | "name") {
            let txt = node_text(child, source);
            if !txt.is_empty() {
                parts.push(txt.to_string());
            }
        }
    }
    if !parts.is_empty() { Some(parts) } else { None }
}

/// Abstracts the extraction of a list of items of a generic type from the AST,
/// recursing into block/body nodes and using a parser function to extract the item.
pub fn extract_list_of<T, F>(node: Node, source: &str, parser: F) -> Vec<T>
where
    F: Fn(Node, &str) -> Option<T> + Copy,
{
    let mut items = vec![];
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(item) = parser(child, source) {
            items.push(item);
        } else if child.kind().contains("body")
            || child.kind().contains("list")
            || child.kind().contains("block")
            || child.kind().contains("declaration")
        {
            items.extend(extract_list_of(child, source, parser));
        }
    }
    items
}

// ==================== SPECIFIC EXTRACTORS ====================

pub fn determine_structured_kind(kind: &str, text: &str) -> StructuredTypeKind {
    if kind.contains("interface") || text.contains("interface") {
        StructuredTypeKind::Interface
    } else if kind.contains("trait") || text.contains("trait") {
        StructuredTypeKind::Trait
    } else if kind.contains("struct") || text.contains("struct") {
        StructuredTypeKind::Struct
    } else {
        StructuredTypeKind::Class
    }
}

pub fn extract_fields(node: Node, source: &str) -> Vec<Field> {
    extract_list_of(node, source, |child, src| {
        if matches!(
            child.kind(),
            "field_declaration" | "property_declaration" | "field" | "member_declaration" | "variable_declarator" | "attribute"
        ) {
            if let Some(name) = extract_identifier(child, src) {
                let ty = extract_type_ref(child, src);
                return Some(Field { name, ty });
            }
        }
        None
    })
}

pub fn extract_methods(node: Node, source: &str) -> Vec<Function> {
    extract_list_of(node, source, try_parse_function)
}

pub fn extract_nested_types(node: Node, source: &str) -> Vec<StructuredType> {
    extract_list_of(node, source, try_parse_structured_type)
}

pub fn extract_parameters(node: Node, source: &str) -> Vec<Parameter> {
    let mut params = vec![];
    if let Some(params_node) = node.child_by_field_name("parameters") {
        let mut cursor = params_node.walk();
        for p in params_node.children(&mut cursor) {
            if p.kind().contains("parameter") {
                let name = extract_identifier(p, source);
                let ty = extract_type_ref(p, source);
                let text = node_text(p, source);
                let is_variadic = text.contains("...") || text.contains("*args");
                params.push(Parameter {
                    name,
                    ty,
                    is_variadic,
                });
            }
        }
    }
    params
}

pub fn extract_return_type(node: Node, source: &str) -> TypeRef {
    if let Some(type_node) = node
        .child_by_field_name("return_type")
        .or_else(|| node.child_by_field_name("type"))
    {
        return extract_type_ref(type_node, source);
    }

    let text = node_text(node, source);
    if let Some(arrow_pos) = text.find("->") {
        let after = text[arrow_pos + 2..].trim();
        let ret_type: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':').collect();
        if !ret_type.is_empty() {
             return TypeRef::Unresolved(split_qualified_name(&ret_type));
        }
    }

    TypeRef::Failed(vec![])
}

pub fn extract_super_types(node: Node, source: &str) -> Vec<TypeRef> {
    let mut supers = vec![];
    let mut super_nodes = vec![];

    if let Some(n) = node.child_by_field_name("super_type") { super_nodes.push(n); }
    if let Some(n) = node.child_by_field_name("extends") { super_nodes.push(n); }
    if let Some(n) = node.child_by_field_name("implements") { super_nodes.push(n); }
    if let Some(n) = node.child_by_field_name("superclass") { super_nodes.push(n); }
    if let Some(n) = node.child_by_field_name("interfaces") { super_nodes.push(n); }
    if let Some(n) = node.child_by_field_name("superclasses") { super_nodes.push(n); }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "base_class_clause" {
            super_nodes.push(child);
        }
    }

    for n in super_nodes {
        let mut cursor = n.walk();
        for child in n.children(&mut cursor) {
            let kind = child.kind();
            if matches!(kind, "type_identifier" | "identifier" | "scoped_type_identifier") {
                supers.push(extract_type_ref(child, source));
            } else if kind == "type_list" {
                let mut c2 = child.walk();
                for c3 in child.children(&mut c2) {
                    if matches!(c3.kind(), "type_identifier" | "identifier" | "scoped_type_identifier") {
                        supers.push(extract_type_ref(c3, source));
                    }
                }
            }
        }
    }
    supers
}

pub fn extract_impl_for(node: Node, source: &str) -> TypeRef {
    extract_type_ref(node, source)
}

pub fn extract_implements_trait(node: Node, source: &str) -> Option<TypeRef> {
    if let Some(trait_node) = node.child_by_field_name("trait") {
        return Some(extract_type_ref(trait_node, source));
    }

    let text = node_text(node, source);
    if let Some(for_pos) = text.find("for ") {
        let before_for = text[..for_pos].trim();
        if before_for.contains("impl") && !before_for.ends_with("impl") {
            let trait_name = before_for.replace("impl", "").trim().to_string();
            if !trait_name.is_empty() {
                return Some(TypeRef::Unresolved(split_qualified_name(&trait_name)));
            }
        }
    }
    None
}

// ==================== REAL TYPES EXTRACTION ====================

pub fn extract_type_ref(node: Node, source: &str) -> TypeRef {
    let kind = node.kind();
    // 0. Handle direct access and identifiers
    if matches!(
        kind,
        "field_access" | "scoped_identifier" | "qualified_identifier" | "identifier" | "type_identifier" | "attribute" | "field_expression"
    ) {
        let text = node_text(node, source);
        if !text.is_empty() {
            return TypeRef::Unresolved(split_qualified_name(&text));
        }
    }

    // 1. Try with common Tree-sitter fields
    if let Some(type_node) = node
        .child_by_field_name("type")
        .or_else(|| node.child_by_field_name("return_type"))
        .or_else(|| node.child_by_field_name("field_type"))
        .or_else(|| node.child_by_field_name("value_type"))
    {
        let text = node_text(type_node, source);
        if !text.is_empty() {
            return TypeRef::Unresolved(split_qualified_name(&text));
        }
    }

    // 2. Fallback: look for generic identifier types
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let child_kind = child.kind();
        if matches!(
            child_kind,
            "type_identifier" | "primitive_type" | "identifier" | "type"
        ) {
            let text = node_text(child, source);
            if !text.is_empty() && !text.contains(" ") {
                return TypeRef::Unresolved(split_qualified_name(&text));
            }
        }
    }

    // 3. Last fallback: text matching after colons or arrows
    let text = node_text(node, source);
    if let Some(colon_pos) = text.find(':') {
        let after = text[colon_pos + 1..].trim();
        if !after.is_empty() {
            return TypeRef::Unresolved(split_qualified_name(after));
        }
    }
    if let Some(arrow_pos) = text.find("->") {
        let after = text[arrow_pos + 2..].trim();
        if !after.is_empty() {
            return TypeRef::Unresolved(split_qualified_name(after));
        }
    }

    TypeRef::Failed(vec![])
}

pub fn extract_block(node: Node, source: &str) -> crate::ir::Block {
    let mut declarations = vec![];
    let mut calls = vec![];
    let mut instantiates = vec![];
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
            if let Some(name) = extract_identifier(child, source) {
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
        }
        // 2. Nested Blocks
        else if kind.contains("body") || kind.contains("block") || kind == "compound_statement" {
            sub_blocks.push(extract_block(child, source));
        }
        // 3. Behavioral Deps (recursive search within current level, avoiding deep blocks)
        else {
            let mut inner_calls = vec![];
            let mut inner_inst = vec![];
            find_behavioral_deps(child, source, &mut inner_calls, &mut inner_inst);
            calls.extend(inner_calls);
            instantiates.extend(inner_inst);
        }
    }

    crate::ir::Block {
        declarations,
        calls,
        instantiates,
        sub_blocks,
    }
}

fn find_behavioral_deps(
    node: Node,
    source: &str,
    calls: &mut Vec<TypeRef>,
    instantiates: &mut Vec<TypeRef>,
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
        if let Some(f) = node.child_by_field_name("function") {
            let f_kind = f.kind();
            if matches!(f_kind, "qualified_identifier" | "scoped_identifier") {
                if let Some(scope) = f
                    .child_by_field_name("scope")
                    .or_else(|| f.child_by_field_name("path"))
                {
                    calls.push(extract_type_ref(scope, source));
                }
            } else if matches!(f_kind, "field_expression" | "attribute") {
                calls.push(extract_type_ref(f, source));
            } else if matches!(f_kind, "identifier" | "type_identifier") {
                calls.push(extract_type_ref(f, source));
            }
        }
    } else if kind == "method_invocation" {
        // Java
        if let Some(obj) = node.child_by_field_name("object") {
            calls.push(extract_type_ref(obj, source));
        } else if let Some(name) = node.child_by_field_name("name") {
            calls.push(extract_type_ref(name, source));
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_behavioral_deps(child, source, calls, instantiates);
    }
}

fn infer_variable_type(node: Node, source: &str) -> TypeRef {
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