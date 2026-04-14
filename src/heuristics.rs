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

    let name = extract_name(node, source).unwrap_or_else(|| "Unnamed".to_string());
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

    let name = extract_name(node, source).unwrap_or_else(|| "unnamed".to_string());
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

    let name = extract_name(node, source).unwrap_or_else(|| "impl".to_string());
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

fn extract_name(node: Node, source: &str) -> Option<String> {
    node.child_by_field_name("name")
        .or_else(|| node.child_by_field_name("type_identifier"))
        .or_else(|| node.child_by_field_name("identifier"))
        .map(|n| n.utf8_text(source.as_bytes()).unwrap_or("").to_string())
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
        ) {
            if let Some(name) = extract_name(child, source) {
                fields.push(Field {
                    name,
                    ty: TypeRef::Unknown,
                });
            }
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

fn extract_super_types(_node: Node, _source: &str) -> Vec<TypeRef> {
    vec![] // TODO futuro: cerca "extends", "implements", "for" ecc.
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
                    name: extract_name(p, source),
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

fn extract_impl_for(_node: Node, _source: &str) -> TypeRef {
    TypeRef::Unknown // TODO futuro
}

fn extract_implements_trait(_node: Node, _source: &str) -> Option<TypeRef> {
    None
}
