use crate::ir::*;
use tree_sitter::Node;

pub enum ParsedItem {
    Component(Component),
    ImplBlock(ImplBlock),
}

// ==================== DISPATCHER CENTRALE ====================
/// Attempts to identify and parse the given Tree-sitter `Node` into an Intermediate Representation (IR) Component.
pub fn dispatch_node(node: Node, source: &str) -> Option<ParsedItem> {
    if let Some(m) = try_parse_module_node(node, source) {
        return Some(ParsedItem::Component(Component::Module(m)));
    }
    if let Some(st) = try_parse_structured_type(node, source) {
        return Some(ParsedItem::Component(Component::StructuredType(st)));
    }
    if let Some(ff) = try_parse_function(node, source) {
        return Some(ParsedItem::Component(Component::Function(ff)));
    }
    if let Some(implb) = try_parse_impl_block(node, source) {
        return Some(ParsedItem::ImplBlock(implb));
    }
    None
}

// ==================== FUNZIONI GENERICHE DI PARSING ====================

/// Heuristically determines if a node is a module or namespace definition.
fn try_parse_module_node(node: Node, source: &str) -> Option<Module> {
    if !node.is_named() {
        return None;
    }
    let kind = node.kind();
    if !kind.contains("mod_item") && !kind.contains("module") && !kind.contains("namespace") {
        return None;
    }
    
    let name = extract_identifier(node, source).unwrap_or_else(|| "unnamed_module".to_string());
    Some(Module {
        name: vec![name],
        sub_modules: vec![],
        structured_types: vec![],
        free_functions: vec![],
        impl_blocks: vec![],
    })
}

/// Heuristically determines if a node is a structured type definition (Struct, Class, Interface, etc.).
fn try_parse_structured_type(node: Node, source: &str) -> Option<StructuredType> {
    if !node.is_named() {
        return None;
    }
    let kind = node.kind();
    let text = node_text(node, source);

    let is_structured = (kind.contains("struct")
        || kind.contains("class")
        || kind.contains("interface")
        || kind.contains("trait")
        || kind.contains("enum")
        || kind.contains("union"))
        && !kind.contains("bound")
        && !kind.contains("clause")
        && !kind.contains("list")
        && !kind.contains("expression")
        && !kind.contains("argument")
        && !kind.contains("call")
        && !kind.contains("identifier")
        && !kind.contains("specifier")
        && !kind.contains("mod");

    if !is_structured {
        return None;
    }

    let name = extract_qualified_name(node, source).unwrap_or_else(|| vec!["unnamed_type".to_string()]);
    let fields = extract_fields(node, source);
    let methods = extract_methods(node, source);
    let super_types = extract_super_types(node, source);
    let nested_types = extract_nested_types(node, source);

    Some(StructuredType {
        name,
        kind: determine_structured_kind(kind, &text),
        fields,
        methods,
        super_types,
        nested_types,
    })
}

/// Heuristically identifies free-standing functions or methods.
fn try_parse_function(node: Node, source: &str) -> Option<Function> {
    if !node.is_named() {
        return None;
    }
    let kind = node.kind();

    let is_function = kind.contains("function")
        || kind.contains("method")
        || kind.contains("fn_item")
        || kind.contains("def")
        || kind.contains("func");

    if !is_function {
        return None;
    }

    let name = extract_identifier(node, source).unwrap_or_else(|| "unnamed_function".to_string());
    let parameters = extract_parameters(node, source);
    let return_type = extract_return_type(node, source);

    Some(Function {
        name,
        signature: crate::ir::Signature {
            parameters,
            return_type,
        },
    })
}

/// Identifies implementation blocks commonly found in Rust.
fn try_parse_impl_block(node: Node, source: &str) -> Option<ImplBlock> {
    if !node.is_named() {
        return None;
    }
    let kind = node.kind();
    if !kind.contains("impl") {
        return None;
    }

    let name = extract_qualified_name(node, source).unwrap_or_else(|| vec!["unnamed_impl".to_string()]);
    let methods = extract_methods(node, source);
    let impl_for = extract_impl_for(node, source);
    let implements_trait = extract_implements_trait(node, source);

    Some(ImplBlock {
        name,
        methods,
        impl_for,
        implements_trait,
    })
}

// ==================== HELPER GENERICI ====================

/// Extracts the raw UTF-8 text from a Tree-sitter node.
fn node_text(node: Node, source: &str) -> String {
    node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
}

/// Tries to extract a simple identifier (string) from common child field names.
fn extract_identifier(node: Node, source: &str) -> Option<String> {
    node.child_by_field_name("name")
        .or_else(|| node.child_by_field_name("type_identifier"))
        .or_else(|| node.child_by_field_name("identifier"))
        .and_then(|n| {
            let text = node_text(n, source).trim().to_string();
            if !text.is_empty() { Some(text) } else { None }
        })
}

/// Extracts a qualified name (e.g. A::B::C) by traversing identifiers.
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
            let ty = extract_type_ref(child, source);
            fields.push(Field { name, ty });
        } else if child.kind().contains("body")
            || child.kind().contains("list")
            || child.kind().contains("block")
        {
            fields.extend(extract_fields(child, source));
        }
    }
    fields
}

fn extract_methods(node: Node, source: &str) -> Vec<Function> {
    let mut methods = vec![];
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(ff) = try_parse_function(child, source) {
            methods.push(Function {
                name: ff.name,
                signature: ff.signature,
            });
        } else if child.kind().contains("body")
            || child.kind().contains("list")
            || child.kind().contains("block")
        {
            methods.extend(extract_methods(child, source));
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
        } else if child.kind().contains("body")
            || child.kind().contains("list")
            || child.kind().contains("block")
        {
            nested.extend(extract_nested_types(child, source));
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
                let name = extract_identifier(p, source);
                let ty = extract_type_ref(p, source);
                let is_variadic =
                    node_text(p, source).contains("...") || node_text(p, source).contains("*args");
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
    extract_type_ref(node, source)
}

// ==================== SUPER TYPES ====================

// fn extract_super_types(node: Node, source: &str) -> Vec<TypeRef> {
//     let mut supers = vec![];
//     let text = node_text(node, source);
//
//     // Euristica ibrida: cerca field Tree-sitter + keyword testuali
//     let super_nodes = node
//         .child_by_field_name("super_type")
//         .or_else(|| node.child_by_field_name("extends"))
//         .or_else(|| node.child_by_field_name("implements"))
//         .or_else(|| node.child_by_field_name("base_clause"));
//
//     if let Some(super_node) = super_nodes {
//         let mut cursor = super_node.walk();
//         for child in super_node.children(&mut cursor) {
//             if let Some(name) = extract_qualified_name(child, source) {
//                 supers.push(TypeRef::Unresolved(name));
//             }
//         }
//     } else if text.contains("extends") || text.contains("implements") || text.contains("for ") {
//         // Fallback testuale per linguaggi che non hanno field precisi
//         let parts: Vec<&str> = text.split(&[':', '.', ' '][..]).collect();
//         for p in parts {
//             if p.contains("extends") || p.contains("implements") || p.contains("for") {
//                 continue;
//             }
//             let trimmed = p.trim();
//             if !trimmed.is_empty()
//                 && trimmed
//                     .chars()
//                     .next()
//                     .map(|c| c.is_alphabetic())
//                     .unwrap_or(false)
//             {
//                 supers.push(TypeRef::Unresolved(vec![trimmed.to_string()]));
//             }
//         }
//     }
//     supers
// }
//
// fn extract_impl_for(node: Node, source: &str) -> TypeRef {
//     // Per Rust: "impl Type for ..." o "impl Type"
//     let text = node_text(node, source);
//     if let Some(for_pos) = text.find("for ") {
//         let after_for = text[for_pos + 4..].trim();
//         if let Some(name) = extract_name_from_text(after_for) {
//             return TypeRef::Unresolved(name);
//         }
//     }
//     // Caso "impl Type { ... }"
//     if let Some(name) = extract_qualified_name(node, source) {
//         TypeRef::Unresolved(name)
//     } else {
//         TypeRef::Failed(vec![])
//     }
// }

fn extract_super_types(node: Node, source: &str) -> Vec<TypeRef> {
    let mut supers = vec![];
    let super_node = node
        .child_by_field_name("super_type")
        .or_else(|| node.child_by_field_name("extends"))
        .or_else(|| node.child_by_field_name("implements"))
        .or_else(|| node.child_by_field_name("base_clause"));

    if let Some(n) = super_node {
        let mut cursor = n.walk();
        for child in n.children(&mut cursor) {
            supers.push(extract_type_ref(child, source));
        }
    }
    supers
}

fn extract_impl_for(node: Node, source: &str) -> TypeRef {
    extract_type_ref(node, source)
}

fn extract_implements_trait(node: Node, source: &str) -> Option<TypeRef> {
    let text = node_text(node, source);
    if let Some(for_pos) = text.find("for ") {
        let before_for = text[..for_pos].trim();
        if before_for.contains("impl") && !before_for.ends_with("impl") {
            // Es. "impl Trait for Type"
            if let Some(name) = extract_name_from_text(before_for) {
                return Some(TypeRef::Unresolved(name));
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

// ==================== ESTRAZIONE TIPI REALI  ====================

fn extract_type_ref(node: Node, source: &str) -> TypeRef {
    // 1. Prova con i field Tree-sitter più comuni (precisi)
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

    // 2. Fallback: cerca qualsiasi nodo di tipo "type_identifier", "primitive_type", ecc.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        if matches!(
            kind,
            "type_identifier" | "primitive_type" | "identifier" | "type"
        ) {
            let text = node_text(child, source);
            if !text.is_empty() && !text.contains(" ") {
                return TypeRef::Unresolved(split_qualified_name(&text));
            }
        }
    }

    // 3. Ultimo fallback: cerca testo dopo ":" o "->" o ":"
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
