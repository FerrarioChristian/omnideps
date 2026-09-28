//! Responsible for converting raw AST nodes into formal TypeRef structures.
//!
//! This module attempts to heuristically deduce types from nodes representing
//! variables, fields, or return values, returning an `Unresolved` reference
//! that will later be processed by the Name Resolution engine.

use crate::model::{StructuredTypeKind, TypeRef};
use tree_sitter::Node;

use super::text_parsing::{node_text, split_qualified_name};

/// Determines the structural classification (Class, Struct, Interface, Trait)
/// of a parsed node based on its AST kind or raw textual representation.
///
/// # Arguments
/// * `kind` - The AST kind string (e.g., "class_declaration").
/// * `text` - The raw text of the node, used as a fallback mechanism.
pub fn determine_structured_kind(kind: &str, text: &str) -> StructuredTypeKind {
    if kind.contains("interface") || text.contains("interface") {
        StructuredTypeKind::Interface
    } else if kind.contains("trait") || text.contains("trait") {
        StructuredTypeKind::Trait
    } else if kind.contains("struct") || text.contains("struct") {
        StructuredTypeKind::Struct
    } else if kind.contains("enum_variant") {
        StructuredTypeKind::EnumVariant
    } else if kind.contains("enum") || text.contains("enum") {
        StructuredTypeKind::Enum
    } else {
        StructuredTypeKind::Class
    }
}

/// A core heuristic function that attempts to extract a Type Reference (`TypeRef`)
/// from a given AST node.
///
/// It employs a multi-tier fallback strategy:
/// 1. Checks if the node is inherently a direct identifier or access path.
/// 2. Inspects common Tree-sitter named fields (`type`, `return_type`).
/// 3. Scans child nodes for known type identifiers.
/// 4. Analyzes the raw text for colons (`:`) or arrows (`->`).
///
/// # Arguments
/// * `node` - The AST node potentially containing type information.
/// * `source` - The complete source code string.
pub fn extract_type_ref(node: Node, source: &str) -> TypeRef {
    let kind = node.kind();

    // 0.2. Unwrap wrapper `type` nodes (e.g. in Python)
    if kind == "type"
        && node.child_count() == 1
        && let Some(child) = node.child(0)
    {
        return extract_type_ref(child, source);
    }

    // 0.5. Try union types
    if let Some(union_ref) = try_extract_union(node, source) {
        return union_ref;
    }

    // 0.6. Try generic/template types (AST native)
    if let Some(generic_ref) = try_extract_generic(node, source) {
        return generic_ref;
    }

    // 0. Handle direct access and identifiers
    if matches!(
        node.kind(),
        "type_identifier"
            | "identifier"
            | "scoped_identifier"
            | "qualified_identifier"
            | "scoped_type_identifier"
            | "attribute"
            | "field_expression"
            | "field_access"
            | "primitive_type"
            | "predefined_type"
            | "template_type"
            | "type"
            | "string"
    ) {
        let text = node_text(node, source);
        let text = text.replace(['\'', '"'], "");
        if !text.is_empty() {
            if let Some(generic_ref) = parse_generic_from_text(&text) {
                return generic_ref;
            }
            return TypeRef::Unresolved(split_qualified_name(&text));
        }
    }

    // 1. Try with common Tree-sitter fields
    if let Some(type_ref) = try_extract_from_type_field(node, source) {
        return type_ref;
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
            if !text.is_empty() && !text.contains(' ') {
                if let Some(generic_ref) = parse_generic_from_text(&text) {
                    return generic_ref;
                }
                return TypeRef::Unresolved(split_qualified_name(&text));
            }
        }
    }

    // 3. Last fallback: text matching after colons or arrows
    let text = node_text(node, source);
    if let Some(colon_pos) = text.find(':') {
        let after = text[colon_pos + 1..].trim();
        if !after.is_empty() {
            return parse_type_from_text(after);
        }
    }
    if let Some(arrow_pos) = text.find("->") {
        let after = text[arrow_pos + 2..].trim();
        if !after.is_empty() {
            return parse_type_from_text(after);
        }
    }

    TypeRef::Failed(vec![])
}

/// Attempts to extract a `TypeRef` by inspecting common Tree-sitter type fields.
///
/// Fields like `type`, `return_type`, `field_type`, or `value_type` often contain
/// the actual type node. This function recursively calls `extract_type_ref` on
/// these fields to properly resolve nested types (e.g., unwrapping references like `&StructA`).
fn try_extract_from_type_field(node: Node, source: &str) -> Option<TypeRef> {
    node.child_by_field_name("type")
        .or_else(|| node.child_by_field_name("return_type"))
        .or_else(|| node.child_by_field_name("field_type"))
        .or_else(|| node.child_by_field_name("value_type"))
        .or_else(|| node.child_by_field_name("right"))
        .map(|type_node| extract_type_ref(type_node, source))
}

/// Extracts a generic type application (`TypeRef::Generic`) from CST constructs
/// that parameterize a base type with arguments (e.g. `template_type`, `generic_type`, `subscript`).
pub fn try_extract_generic(node: Node, source: &str) -> Option<TypeRef> {
    let kind = node.kind();
    if !matches!(kind, "template_type" | "generic_type" | "subscript") {
        return None;
    }

    let base_node = node
        .child_by_field_name("type")
        .or_else(|| node.child_by_field_name("name"))
        .or_else(|| node.child_by_field_name("value"))
        .or_else(|| node.named_child(0))?;

    let base_tr = extract_type_ref(base_node, source);
    if matches!(base_tr, TypeRef::Failed(_)) {
        return None;
    }

    // Exclude Union types (e.g. typing.Union[A, B]), handled specifically by try_extract_union
    if let TypeRef::Unresolved(ref qn) = base_tr
        && qn.last().map(|s| s.as_str()) == Some("Union")
    {
        return None;
    }

    let args_container = node
        .child_by_field_name("type_arguments")
        .or_else(|| node.child_by_field_name("arguments"))
        .or_else(|| node.child_by_field_name("subscript"))
        .or_else(|| {
            let mut cursor = node.walk();
            node.children(&mut cursor).find(|c| {
                matches!(
                    c.kind(),
                    "template_argument_list" | "type_arguments" | "type_parameter"
                )
            })
        })?;

    let args = extract_type_argument_list(args_container, source);
    if args.is_empty() {
        return None;
    }

    Some(TypeRef::Generic {
        base: Box::new(base_tr),
        args,
    })
}

/// Extracts a list of type arguments from an argument container node (tuple, type_arguments, template_argument_list, type_parameter)
/// or a single subscript argument node.
fn extract_type_argument_list(container: Node, source: &str) -> Vec<TypeRef> {
    let kind = container.kind();
    if matches!(
        kind,
        "tuple" | "type_arguments" | "template_argument_list" | "type_parameter"
    ) {
        let mut args = Vec::new();
        let mut cursor = container.walk();
        for child in container.children(&mut cursor) {
            let ck = child.kind();
            if !matches!(ck, "<" | ">" | "(" | ")" | "[" | "]" | ",") && !ck.is_empty() {
                let tr = extract_type_ref(child, source);
                if !matches!(tr, TypeRef::Failed(_)) {
                    args.push(tr);
                }
            }
        }
        args
    } else {
        let tr = extract_type_ref(container, source);
        if matches!(tr, TypeRef::Failed(_)) {
            vec![]
        } else {
            vec![tr]
        }
    }
}

/// Parses a generic type from a textual representation containing balanced `<...>` or `[...]`.
pub fn parse_generic_from_text(text: &str) -> Option<TypeRef> {
    let text = text.trim();
    parse_delimited_type(text, '<', '>').or_else(|| parse_delimited_type(text, '[', ']'))
}

/// Helper to parse a delimited type application: Base<Arg1, Arg2> or Base[Arg1, Arg2]
fn parse_delimited_type(text: &str, open: char, close: char) -> Option<TypeRef> {
    if !text.ends_with(close) {
        return None;
    }
    let open_pos = text.find(open)?;
    let base_text = text[..open_pos].trim();
    if base_text.is_empty() {
        return None;
    }
    let inner = &text[open_pos + 1..text.len() - 1];
    let arg_texts = split_generic_arguments(inner);
    if arg_texts.is_empty() {
        return None;
    }

    let parsed_args: Vec<TypeRef> = arg_texts
        .into_iter()
        .map(|a| parse_type_from_text(&a))
        .collect();

    if base_text == "Union" {
        Some(TypeRef::Union(parsed_args))
    } else {
        Some(TypeRef::Generic {
            base: Box::new(parse_type_from_text(base_text)),
            args: parsed_args,
        })
    }
}

/// Fallback text parser that handles nested generics, unions, and simple identifiers.
pub fn parse_type_from_text(text: &str) -> TypeRef {
    let text = text.trim().replace(['&', '*'], "");
    let text = text.trim();
    if let Some(generic_ref) = parse_generic_from_text(text) {
        return generic_ref;
    }
    if text.contains('|') {
        let types: Vec<TypeRef> = text
            .split('|')
            .map(|part| parse_type_from_text(part.trim()))
            .collect();
        return TypeRef::Union(types);
    }
    TypeRef::Unresolved(split_qualified_name(text))
}

fn split_generic_arguments(inner: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth: usize = 0;

    for ch in inner.chars() {
        match ch {
            '<' | '[' | '(' => {
                depth += 1;
                current.push(ch);
            }
            '>' | ']' | ')' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    args.push(trimmed.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        args.push(trimmed.to_string());
    }
    args
}

/// Tries to extract a Union type from typical constructs like `union_type`, `binary_operator` (|), or `Union[...]`
fn try_extract_union(node: Node, source: &str) -> Option<TypeRef> {
    let kind = node.kind();
    if kind == "union_type" {
        let mut types = vec![];
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "|" {
                let ty = extract_type_ref(child, source);
                if !matches!(ty, TypeRef::Failed(_)) {
                    types.push(ty);
                }
            }
        }
        if !types.is_empty() {
            return Some(TypeRef::Union(types));
        }
    }

    if kind == "binary_operator" && node_text(node, source).contains('|') {
        let mut types = vec![];
        if let Some(left) = node.child_by_field_name("left") {
            types.push(extract_type_ref(left, source));
        }
        if let Some(right) = node.child_by_field_name("right") {
            types.push(extract_type_ref(right, source));
        }
        if !types.is_empty() {
            return Some(TypeRef::Union(types));
        }
    }

    if kind == "generic_type"
        && let Some(name_node) = node.child(0)
        && node_text(name_node, source).trim() == "Union"
        && let Some(params) = node.child(1)
    {
        let mut types = vec![];
        let mut cursor = params.walk();
        for child in params.children(&mut cursor) {
            let ckind = child.kind();
            if ckind != "[" && ckind != "]" && ckind != "," {
                let ty = extract_type_ref(child, source);
                if !matches!(ty, TypeRef::Failed(_)) {
                    types.push(ty);
                }
            }
        }
        if !types.is_empty() {
            return Some(TypeRef::Union(types));
        }
    }

    if kind == "subscript"
        && let Some(val_node) = node.child_by_field_name("value")
        && node_text(val_node, source).trim() == "Union"
        && let Some(sub_node) = node.child_by_field_name("subscript")
    {
        let mut types = vec![];
        if sub_node.kind() == "tuple" {
            let mut cursor = sub_node.walk();
            for child in sub_node.children(&mut cursor) {
                let ck = child.kind();
                if ck != "(" && ck != ")" && ck != "," && !ck.is_empty() {
                    let ty = extract_type_ref(child, source);
                    if !matches!(ty, TypeRef::Failed(_)) {
                        types.push(ty);
                    }
                }
            }
        } else {
            let ty = extract_type_ref(sub_node, source);
            if !matches!(ty, TypeRef::Failed(_)) {
                types.push(ty);
            }
        }
        if !types.is_empty() {
            return Some(TypeRef::Union(types));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_generic_simple_angle_brackets() {
        let tr = parse_generic_from_text("Box<Transport::Car>").expect("Should parse generic");
        match tr {
            TypeRef::Generic { base, args } => {
                assert_eq!(*base, TypeRef::Unresolved(vec!["Box".to_string()]));
                assert_eq!(args.len(), 1);
                assert_eq!(
                    args[0],
                    TypeRef::Unresolved(vec!["Transport".to_string(), "Car".to_string()])
                );
            }
            _ => panic!("Expected TypeRef::Generic"),
        }
    }

    #[test]
    fn test_parse_generic_multiple_arguments() {
        let tr =
            parse_generic_from_text("KeyValue<int, Transport::Car>").expect("Should parse generic");
        match tr {
            TypeRef::Generic { base, args } => {
                assert_eq!(*base, TypeRef::Unresolved(vec!["KeyValue".to_string()]));
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected TypeRef::Generic"),
        }
    }

    #[test]
    fn test_parse_generic_nested_square_brackets() {
        let tr = parse_generic_from_text("dict[str, list[User]]").expect("Should parse generic");
        match tr {
            TypeRef::Generic { base, args } => {
                assert_eq!(*base, TypeRef::Unresolved(vec!["dict".to_string()]));
                assert_eq!(args.len(), 2);
                assert!(matches!(args[1], TypeRef::Generic { .. }));
            }
            _ => panic!("Expected TypeRef::Generic"),
        }
    }

    #[test]
    fn test_parse_generic_non_generic() {
        assert!(parse_generic_from_text("Car").is_none());
        assert!(parse_generic_from_text("Transport::Car").is_none());
    }
}
