use crate::ir::*;
use tree_sitter::Node;

pub enum ParsedItem {
    Component(Component),
    ImplBlock(ImplBlock),
    Import(Import),
}

// ==================== DISPATCHER CENTRALE ====================
/// Attempts to identify and parse the given Tree-sitter `Node` into an Intermediate Representation (IR) Component.
pub fn dispatch_node(node: Node, source: &str) -> Option<ParsedItem> {
    if let Some(import) = try_parse_import(node, source) {
        return Some(ParsedItem::Import(import));
    }
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
        imports: vec![],
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
        && !kind.contains("reference")
        && !kind.contains("body")
        && !kind.contains("mod")
        && !kind.contains("variant");

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

    let mut calls = vec![];
    let mut instantiates = vec![];
    if let Some(body) = node.child_by_field_name("body") {
        traverse_for_body_deps(body, source, &mut calls, &mut instantiates);
    }

    Some(Function {
        name: vec![name],
        signature: crate::ir::Signature {
            parameters,
            return_type,
        },
        calls,
        instantiates,
    })
    }

    fn traverse_for_body_deps(node: Node, source: &str, calls: &mut Vec<TypeRef>, instantiates: &mut Vec<TypeRef>) {
    let kind = node.kind();

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
                if let Some(scope) = f.child_by_field_name("scope").or_else(|| f.child_by_field_name("path")) {
                    calls.push(extract_type_ref(scope, source));
                }
            } else if matches!(f_kind, "field_expression" | "attribute") {
                if let Some(obj) = f.child_by_field_name("argument").or_else(|| f.child_by_field_name("object")).or_else(|| f.child_by_field_name("value")) {
                    calls.push(extract_type_ref(obj, source));
                }
            } else if matches!(f_kind, "identifier" | "type_identifier") {
                calls.push(extract_type_ref(f, source));
            }
        }
    } else if kind == "method_invocation" { // Java
        if let Some(obj) = node.child_by_field_name("object") {
            calls.push(extract_type_ref(obj, source));
        } else if let Some(name) = node.child_by_field_name("name") {
            calls.push(extract_type_ref(name, source));
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        traverse_for_body_deps(child, source, calls, instantiates);
    }
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
    let nested_types = extract_nested_types(node, source);

    Some(ImplBlock {
        name,
        methods,
        impl_for,
        implements_trait,
        nested_types,
    })
}

fn try_parse_import(node: Node, source: &str) -> Option<Import> {
    if !node.is_named() {
        return None;
    }
    let kind = node.kind();
    if !matches!(
        kind,
        "use_declaration" | "import_declaration" | "import_statement" | "import_from_statement" | "preproc_include"
    ) {
        return None;
    }

    let text = node_text(node, source);
    let is_wildcard = text.contains('*') || text.contains(".*") || text.contains("::*");
    
    // Attempt to find alias
    let mut alias = None;
    if let Some(as_pos) = text.find(" as ") {
        let after_as = text[as_pos + 4..].trim();
        let alias_part: String = after_as.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        if !alias_part.is_empty() {
            alias = Some(alias_part);
        }
    }

    // Try to extract path using tree-sitter fields, fallback to regex-like
    let path = if let Some(p_node) = node.child_by_field_name("name")
        .or_else(|| node.child_by_field_name("path")) 
        .or_else(|| node.child_by_field_name("module_name"))
    {
        split_qualified_name(&node_text(p_node, source))
    } else {
        // Fallback for preproc_include or generic imports
        let mut p = vec![];
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let c_kind = child.kind();
            if matches!(c_kind, "scoped_identifier" | "identifier" | "dotted_name" | "system_lib_string" | "string_literal") {
                let txt = node_text(child, source).replace("\"", "").replace("<", "").replace(">", "");
                p = split_qualified_name(&txt);
                break;
            }
        }
        p
    };

    if path.is_empty() {
        return None;
    }

    Some(Import {
        path,
        alias,
        is_wildcard,
    })
}

// ==================== HELPER GENERICI ====================

/// Extracts the raw UTF-8 text from a Tree-sitter node.
fn node_text(node: Node, source: &str) -> String {
    node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
}

/// Tries to extract a simple identifier (string) from common child field names.
fn extract_identifier(node: Node, source: &str) -> Option<String> {
    if let Some(n) = node
        .child_by_field_name("name")
        .or_else(|| node.child_by_field_name("type_identifier"))
        .or_else(|| node.child_by_field_name("identifier"))
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
                calls: ff.calls,
                instantiates: ff.instantiates,
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
            || child.kind().contains("declaration")
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
    // 0. Handle direct field_access, scoped_identifier, qualified_identifier or identifiers
    let kind = node.kind();
    if matches!(
        kind,
        "field_access" | "scoped_identifier" | "qualified_identifier" | "identifier" | "type_identifier" | "attribute"
    ) {
        let text = node_text(node, source);
        if !text.is_empty() {
            return TypeRef::Unresolved(split_qualified_name(&text));
        }
    }

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
