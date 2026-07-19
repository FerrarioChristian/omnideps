//! Handles the extraction of macro-level architectural structures from the AST.
//!
//! This module focuses on extracting signatures and boundaries of complex entities
//! like fields, methods, parameters, and super-types. It heavily utilizes
//! recursive traversal patterns to abstract away language-specific syntax trees.

use crate::model::{Field, Function, Parameter, StructuredType, TypeRef};
use tree_sitter::Node;

use super::text_parsing::{extract_identifier, node_text, split_qualified_name};
use super::type_extraction::extract_type_ref;

/// Recursively abstracts the extraction of a list of items of a generic type `T`.
///
/// It scans the children of the provided node. If a child matches the given parsing
/// logic (`parser`), the item is collected. If it encounters a container node (like `body` or `block`),
/// it descends into it recursively.
///
/// # Arguments
/// * `node` - The root AST node to begin the search from.
/// * `source` - The complete source code string.
/// * `parser` - A closure or function pointer that attempts to extract `T` from a Node.
pub fn extract_list_of<T, F>(node: Node, source: &str, parser: F) -> Vec<T>
where
    F: Fn(Node, &str) -> Option<T> + Copy,
{
    let mut items = vec![];
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(item) = parser(child, source) {
            items.push(item);
        } else if !crate::heuristics::classifiers::is_function(child)
            && !crate::heuristics::classifiers::is_structured_type(child)
            && (child.kind().contains("body")
                || child.kind().contains("list")
                || child.kind().contains("block")
                || child.kind().contains("declaration")
                || child.kind().contains("variant"))
        {
            items.extend(extract_list_of(child, source, parser));
        }
    }
    items
}

/// Extracts all field (property) declarations from a given node.
pub fn extract_fields(node: Node, source: &str) -> Vec<Field> {
    let mut fields = extract_list_of(node, source, |child, src| {
        if matches!(
            child.kind(),
            "field_declaration"
                | "property_declaration"
                | "field"
                | "member_declaration"
                | "variable_declarator"
                | "attribute"
        ) {
            if let Some(name) = extract_identifier(child, src) {
                let ty = extract_type_ref(child, src);
                return Some(Field { name, ty });
            }
        }
        None
    });

    // Handle Rust's Tuple Structs / Enum Tuple Variants
    fn extract_tuple_fields(n: Node, src: &str, fds: &mut Vec<Field>) {
        if n.kind() == "ordered_field_declaration_list" {
            let mut index = 0;
            let mut c = n.walk();
            for child in n.children(&mut c) {
                let kind = child.kind();
                if kind == "visibility_modifier"
                    || kind == ","
                    || kind == "("
                    || kind == ")"
                    || kind.contains("attribute")
                {
                    continue;
                }
                fds.push(Field {
                    name: index.to_string(),
                    ty: extract_type_ref(child, src),
                });
                index += 1;
            }
        } else {
            let mut c = n.walk();
            for child in n.children(&mut c) {
                if !crate::heuristics::classifiers::is_function(child) 
                    && !crate::heuristics::classifiers::is_structured_type(child) 
                {
                    extract_tuple_fields(child, src, fds);
                }
            }
        }
    }
    extract_tuple_fields(node, source, &mut fields);

    fields
}

/// Extracts all method declarations from a given node.
pub fn extract_methods(node: Node, source: &str) -> Vec<Function> {
    extract_list_of(node, source, crate::heuristics::parsers::try_parse_function)
}

/// Extracts all nested structured types (classes, structs) declared within a given node.
pub fn extract_nested_types(node: Node, source: &str) -> Vec<StructuredType> {
    extract_list_of(
        node,
        source,
        crate::heuristics::parsers::try_parse_structured_type,
    )
}

/// Extracts formal parameters from a function or method signature.
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

/// Deduces the return type of a function signature.
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
        let ret_type: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':')
            .collect();
        if !ret_type.is_empty() {
            return TypeRef::Unresolved(split_qualified_name(&ret_type));
        }
    }

    TypeRef::Failed(vec![])
}

/// Extracts all inherited or implemented super-types (base classes, interfaces, traits).
pub fn extract_super_types(node: Node, source: &str) -> Vec<TypeRef> {
    let mut supers = vec![];
    let mut super_nodes = vec![];

    if let Some(n) = node.child_by_field_name("super_type") {
        super_nodes.push(n);
    }
    if let Some(n) = node.child_by_field_name("extends") {
        super_nodes.push(n);
    }
    if let Some(n) = node.child_by_field_name("implements") {
        super_nodes.push(n);
    }
    if let Some(n) = node.child_by_field_name("superclass") {
        super_nodes.push(n);
    }
    if let Some(n) = node.child_by_field_name("interfaces") {
        super_nodes.push(n);
    }
    if let Some(n) = node.child_by_field_name("superclasses") {
        super_nodes.push(n);
    }
    if let Some(n) = node.child_by_field_name("bounds") {
        super_nodes.push(n);
    } // trait bounds

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
            if matches!(
                kind,
                "type_identifier" | "identifier" | "scoped_type_identifier"
            ) {
                supers.push(extract_type_ref(child, source));
            } else if kind == "type_list" {
                let mut c2 = child.walk();
                for c3 in child.children(&mut c2) {
                    if matches!(
                        c3.kind(),
                        "type_identifier" | "identifier" | "scoped_type_identifier"
                    ) {
                        supers.push(extract_type_ref(c3, source));
                    }
                }
            }
        }
    }
    supers
}

/// Extracts the target type that an implementation block is extending (e.g. `Target` in `impl Target`).
pub fn extract_impl_for(node: Node, source: &str) -> TypeRef {
    // In Rust, `impl Trait for Target` has `trait` and `type` fields.
    if let Some(type_node) = node.child_by_field_name("type") {
        return extract_type_ref(type_node, source);
    }

    // Fallback: If there's no trait, the first type is the target (impl Target).
    // If there is a trait (impl Trait for Target), the second type is the target.
    if let Some(trait_node) = node.child_by_field_name("trait") {
        // Find the type identifier that comes AFTER the trait node
        let mut cursor = node.walk();
        let mut found_trait = false;
        for child in node.children(&mut cursor) {
            if child.id() == trait_node.id() {
                found_trait = true;
            } else if found_trait
                && matches!(
                    child.kind(),
                    "type_identifier" | "scoped_type_identifier" | "identifier"
                )
            {
                return extract_type_ref(child, source);
            }
        }
    }

    extract_type_ref(node, source)
}

/// Extracts the specific trait or interface being implemented by an implementation block
/// (e.g. `Trait` in `impl Trait for Target`).
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
