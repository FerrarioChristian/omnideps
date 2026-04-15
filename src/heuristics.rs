use crate::ir::*;
use tree_sitter::Node;

// ==================== DISPATCHER CENTRALE ====================
pub fn dispatch_node(node: Node, source: &str) -> Option<Component> {
    if let Some(st) = try_parse_structured_type(node, source) {
        return Some(Component::StructuredType(st));
    }
    if let Some(ff) = try_parse_free_function(node, source) {
        return Some(Component::FreeFunction(ff));
    }
    if let Some(implb) = try_parse_impl_block(node, source) {
        return Some(Component::ImplBlock(implb));
    }
    None
}

// ==================== FUNZIONI GENERICHE DI PARSING ====================

fn try_parse_structured_type(node: Node, source: &str) -> Option<StructuredType> {
    let kind = node.kind();
    let text = node_text(node, source);

    let is_structured = matches!(
        kind,
        "struct_item"
            | "class_declaration"
            | "interface_declaration"
            | "struct_specifier"
            | "trait_item"
            | "enum_item"
            | "union_item"
    ) || text.contains("struct")
        || text.contains("class")
        || text.contains("interface")
        || text.contains("trait")
        || text.contains("enum");

    if !is_structured {
        return None;
    }

    let name = extract_identifier(node, source).unwrap_or_else(|| "Unnamed".to_string());
    let fields = extract_fields(node, source);
    let methods = extract_methods(node, source);
    let super_types = extract_super_types(node, source);
    let nested_types = extract_nested_types(node, source);

    Some(StructuredType {
        name: vec![name],
        kind: determine_structured_kind(kind, &text),
        fields,
        methods,
        super_types,
        nested_types,
    })
}

fn try_parse_free_function(node: Node, source: &str) -> Option<FreeFunction> {
    let kind = node.kind();
    let text = node_text(node, source);

    let is_function = matches!(
        kind,
        "function_item"
            | "fn_item"
            | "function_declaration"
            | "def_statement"
            | "method_definition"
    ) || text.contains("fn ")
        || text.contains("def ")
        || text.contains("func ");

    if !is_function {
        return None;
    }

    let name = extract_identifier(node, source).unwrap_or_else(|| "unnamed_function".to_string());
    let parameters = extract_parameters(node, source);
    let return_type = extract_return_type(node, source);

    Some(FreeFunction {
        name,
        parameters,
        return_type,
    })
}

fn try_parse_impl_block(node: Node, source: &str) -> Option<ImplBlock> {
    let kind = node.kind();
    let text = node_text(node, source);
    if kind != "impl_item" && !text.contains("impl ") {
        return None;
    }

    let name = extract_identifier(node, source).unwrap_or_else(|| "unnamed_impl".to_string());
    let methods = extract_methods(node, source);
    let impl_for = extract_impl_for(node, source);
    let implements_trait = extract_implements_trait(node, source);

    Some(ImplBlock {
        name: vec![name],
        methods,
        impl_for,
        implements_trait,
    })
}

// ==================== HELPER GENERICI ====================

fn node_text(node: Node, source: &str) -> String {
    node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
}

fn extract_identifier(node: Node, source: &str) -> Option<String> {
    node.child_by_field_name("name")
        .or_else(|| node.child_by_field_name("type_identifier"))
        .or_else(|| node.child_by_field_name("identifier"))
        .and_then(|n| {
            let text = node_text(n, source).trim().to_string();
            if !text.is_empty() { Some(text) } else { None }
        })
}

fn extract_qualified_name(node: Node, source: &str) -> Option<QualifiedName> {
    // Prova con field Tree-sitter (più preciso)
    if let Some(n) = node
        .child_by_field_name("name")
        .or_else(|| node.child_by_field_name("type_identifier"))
        .or_else(|| node.child_by_field_name("identifier"))
    {
        let text = &node_text(n, source);
        return Some(split_qualified_name(text));
    }

    // Fallback: cerca sequenza di identifier separati da :: o .
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

/*
è stato suddiviso in due funzioni più specifiche per evitare confusione tra "qualified name"
(es. std::vector) e "identifier semplice" (nomi di variabili ecc.)
*/

/*
fn extract_name(node: Node, source: &str) -> Option<QualifiedName> {
    // 1. Prova con field Tree-sitter (più preciso)
    if let Some(name_node) = node
        .child_by_field_name("name")
        .or_else(|| node.child_by_field_name("type_identifier"))
        .or_else(|| node.child_by_field_name("identifier"))
    {
        let text = name_node.utf8_text(source.as_bytes()).unwrap_or("");
        return Some(split_qualified_name(text));
    }

    // 2. Fallback: cerca sequenza di identifier separati da :: o .
    let mut cursor = node.walk();
    let mut parts = vec![];
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "identifier" | "type_identifier" | "name") {
            let txt = child.utf8_text(source.as_bytes()).unwrap_or("");
            if !txt.is_empty() {
                parts.push(txt.to_string());
            }
        }
    }
    if !parts.is_empty() { Some(parts) } else { None }
}
*/

// Helper per trasformare "std::vector" o "Outer.Inner" in Vec
fn split_qualified_name(text: &str) -> QualifiedName {
    text.split(&[':', '.', ' '][..])
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect()
}

fn determine_structured_kind(kind: &str, text: &str) -> StructuredTypeKind {
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

fn extract_fields(node: Node, source: &str) -> Vec<Field> {
    let mut fields = vec![];
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if matches!(
            child.kind(),
            "field_declaration" | "property_declaration" | "field"
        ) && let Some(name) = extract_identifier(child, source)
        {
            fields.push(Field {
                name,
                ty: TypeRef::Unknown,
            });
        }
    }
    fields
}

fn extract_methods(node: Node, source: &str) -> Vec<Method> {
    let mut methods = vec![];
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(ff) = try_parse_free_function(child, source) {
            methods.push(Method {
                name: ff.name,
                parameters: ff.parameters,
                return_type: ff.return_type,
            });
        }
    }
    methods
}

fn extract_nested_types(node: Node, source: &str) -> Vec<StructuredType> {
    let mut nested = vec![];
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(st) = try_parse_structured_type(child, source) {
            nested.push(st);
        }
    }
    nested
}

fn extract_parameters(node: Node, source: &str) -> Vec<Parameter> {
    let mut params = vec![];
    if let Some(params_node) = node.child_by_field_name("parameters") {
        let mut cursor = params_node.walk();
        for p in params_node.children(&mut cursor) {
            if p.kind().contains("parameter") {
                params.push(Parameter {
                    name: extract_identifier(p, source),
                    ty: TypeRef::Unknown,
                    is_variadic: node_text(p, source).contains("...")
                        || node_text(p, source).contains("*args"),
                });
            }
        }
    }
    params
}

fn extract_return_type(node: Node, source: &str) -> TypeRef {
    node.child_by_field_name("return_type")
        .map(|n| TypeRef::UserDefined(vec![node_text(n, source)]))
        .unwrap_or(TypeRef::Unknown)
}

// ==================== SUPER TYPES ====================

fn extract_super_types(node: Node, source: &str) -> Vec<TypeRef> {
    let mut supers = vec![];
    let text = node_text(node, source);

    // Euristica ibrida: cerca field Tree-sitter + keyword testuali
    let super_nodes = node
        .child_by_field_name("super_type")
        .or_else(|| node.child_by_field_name("extends"))
        .or_else(|| node.child_by_field_name("implements"))
        .or_else(|| node.child_by_field_name("base_clause"));

    if let Some(super_node) = super_nodes {
        let mut cursor = super_node.walk();
        for child in super_node.children(&mut cursor) {
            if let Some(name) = extract_qualified_name(child, source) {
                supers.push(TypeRef::UserDefined(name));
            }
        }
    } else if text.contains("extends") || text.contains("implements") || text.contains("for ") {
        // Fallback testuale per linguaggi che non hanno field precisi
        let parts: Vec<&str> = text.split(&[':', '.', ' '][..]).collect();
        for p in parts {
            if p.contains("extends") || p.contains("implements") || p.contains("for") {
                continue;
            }
            let trimmed = p.trim();
            if !trimmed.is_empty()
                && trimmed
                    .chars()
                    .next()
                    .map(|c| c.is_alphabetic())
                    .unwrap_or(false)
            {
                supers.push(TypeRef::UserDefined(vec![trimmed.to_string()]));
            }
        }
    }
    supers
}

fn extract_impl_for(node: Node, source: &str) -> TypeRef {
    // Per Rust: "impl Type for ..." o "impl Type"
    let text = node_text(node, source);
    if let Some(for_pos) = text.find("for ") {
        let after_for = text[for_pos + 4..].trim();
        if let Some(name) = extract_name_from_text(after_for) {
            return TypeRef::UserDefined(name);
        }
    }
    // Caso "impl Type { ... }"
    if let Some(name) = extract_qualified_name(node, source) {
        TypeRef::UserDefined(name)
    } else {
        TypeRef::Unknown
    }
}

fn extract_implements_trait(node: Node, source: &str) -> Option<TypeRef> {
    let text = node_text(node, source);
    if let Some(for_pos) = text.find("for ") {
        let before_for = text[..for_pos].trim();
        if before_for.contains("impl") && !before_for.ends_with("impl") {
            // Es. "impl Trait for Type"
            if let Some(name) = extract_name_from_text(before_for) {
                return Some(TypeRef::UserDefined(name));
            }
        }
    }
    None
}

// Helper piccolo per estrarre nome da stringa testuale
fn extract_name_from_text(text: &str) -> Option<QualifiedName> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(split_qualified_name(trimmed))
}
