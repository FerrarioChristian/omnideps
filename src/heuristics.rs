use crate::ir::*;
use tree_sitter::Node;

// ==================== DISPATCHER CENTRALE ====================
pub fn dispatch_node(node: Node, source: &str) -> Option<Component> {
    // Prova in ordine di priorità le varie euristiche
    if let Some(st) = try_parse_structured_type(node, source) {
        return Some(Component::StructuredType(st));
    }
    if let Some(ff) = try_parse_free_function(node, source) {
        return Some(Component::FreeFunction(ff));
    }
    if let Some(implb) = try_parse_impl_block(node, source) {
        return Some(Component::ImplBlock(implb));
    }
    // TODO: aggiungere altri (enum, union, trait puro, ecc.)
    None
}

// ==================== FUNZIONI GENERICHE DI PARSING ====================

fn try_parse_structured_type(node: Node, source: &str) -> Option<StructuredType> {
    let kind = node.kind();
    let text = node.text(source);

    // Euristica 1: node.kind() tipico
    if matches!(
        kind,
        "struct_item" | "class_declaration" | "interface_declaration" | "struct_specifier"
    ) {
        // Euristica 2: keyword testuale di fallback
    } else if text.contains("struct")
        || text.contains("class")
        || text.contains("interface")
        || text.contains("trait")
    {
        // continua
    } else {
        return None;
    }

    // Estrazione nome, fields, methods, superTypes, nested_types...
    let name = extract_name(node, source)?;
    let fields = extract_fields(node, source);
    let methods = extract_methods(node, source);
    let super_types = extract_super_types(node, source);

    Some(StructuredType {
        name: vec![name],
        kind: determine_structured_kind(kind, text),
        fields,
        methods,
        super_types,
        nested_types: vec![], // popolato ricorsivamente
    })
}

fn try_parse_free_function(node: Node, source: &str) -> Option<FreeFunction> {
    let kind = node.kind();
    let text = node.text(source);

    if !matches!(
        kind,
        "function_item" | "fn_item" | "function_declaration" | "def_statement"
    ) && !(text.contains("fn ") || text.contains("def ") || text.contains("func "))
    {
        return None;
    }

    let name = extract_name(node, source)?;
    let params = extract_parameters(node, source);
    let return_type = extract_return_type(node, source);

    Some(FreeFunction {
        name,
        parameters: params,
        return_type,
    })
}

fn try_parse_impl_block(node: Node, source: &str) -> Option<ImplBlock> {
    let kind = node.kind();
    if kind != "impl_item" && !node.text(source).contains("impl ") {
        return None;
    }

    // estrazione di impl_for e implements_trait...
    unimplemented!("impl block parsing not implemented yet");
}

// ==================== HELPER GENERICI (usati da tutte) ====================

fn extract_name(node: Node, source: &str) -> Option<String> {
    // cerca il primo figlio di tipo "identifier", "type_identifier", "name" ecc.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "identifier" | "type_identifier" | "name") {
            return Some(child.text(source).to_string());
        }
    }
    None
}

fn node_text(node: Node, source: &str) -> String {
    node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
}

// TODO: implementare extract_fields, extract_parameters, determine_structured_kind, ecc.
