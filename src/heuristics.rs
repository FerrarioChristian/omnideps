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

    // Euristica ibrida: node.kind() + keyword testuale
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

    let name = extract_name(node, source)?;
    let fields = extract_fields(node, source);
    let methods = extract_methods(node, source);
    let super_types = extract_super_types(node, source);
    let nested_types = extract_nested_types(node, source);

    let kind_enum = determine_structured_kind(kind, &text);

    Some(StructuredType {
        name: vec![name],
        kind: kind_enum,
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

    let name = extract_name(node, source)?;
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
    // Prova prima con i field name di Tree-sitter
    if let Some(name_node) = node.child_by_field_name("name") {
        return Some(
            name_node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string(),
        );
    }
    // Fallback su qualsiasi nodo "identifier" o "type_identifier"
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "identifier" | "type_identifier" | "name") {
            return Some(child.utf8_text(source.as_bytes()).unwrap_or("").to_string());
        }
    }
    None
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
            "field_declaration" | "field_declaration_list" | "property_declaration"
        ) {
            if let Some(name) = extract_name(child, source) {
                let ty = TypeRef::Unknown; // TODO: estrarre tipo reale
                fields.push(Field { name, ty });
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
            // Converte FreeFunction in Method (stessa struttura)
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
    // TODO: implementazione semplice per ora
    vec![] // per il momento lasciamo vuoto
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
    // Cerchiamo il nodo parameters / parameter_list
    if let Some(params_node) = node.child_by_field_name("parameters") {
        let mut cursor = params_node.walk();
        for param in params_node.children(&mut cursor) {
            if param.kind().contains("parameter") {
                let name = extract_name(param, source);
                let ty = TypeRef::Unknown; // TODO: estrarre tipo
                let is_variadic = node_text(param, source).contains("...")
                    || node_text(param, source).contains("*args");
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

fn extract_return_type(node: Node, source: &str) -> TypeRef {
    // Molto semplice per ora
    if let Some(ret_node) = node.child_by_field_name("return_type") {
        TypeRef::UserDefined(vec![node_text(ret_node, source)])
    } else {
        TypeRef::Unknown
    }
}

fn extract_impl_for(_node: Node, _source: &str) -> TypeRef {
    // Per "impl Type for ..." o "impl Type"
    TypeRef::Unknown // TODO: migliorare
}

fn extract_implements_trait(_node: Node, _source: &str) -> Option<TypeRef> {
    None // TODO: migliorare
}
