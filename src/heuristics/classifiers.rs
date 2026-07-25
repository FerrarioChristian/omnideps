use tree_sitter::Node;

/// Heuristically determines if a node is a module or namespace definition.
pub fn is_module(node: Node) -> bool {
    if !node.is_named() {
        return false;
    }
    let kind = node.kind();
    kind.contains("mod_item") || kind.contains("module") || kind.contains("namespace")
}

/// Heuristically determines if a node is a file-level package declaration.
pub fn is_package_declaration(node: Node) -> bool {
    if !node.is_named() {
        return false;
    }
    let kind = node.kind();
    kind == "package_declaration" || kind == "package_clause"
}

/// Heuristically determines if a node is a structured type definition (Struct, Class, Interface, etc.).
pub fn is_structured_type(node: Node) -> bool {
    if !node.is_named() {
        return false;
    }
    let kind = node.kind();
    if kind == "type_definition" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let child_kind = child.kind();
            if child_kind.contains("struct") || child_kind.contains("enum") || child_kind.contains("union") {
                return true;
            }
        }
        return false;
    }
    (kind.contains("struct")
        || kind.contains("class")
        || kind.contains("interface")
        || kind.contains("trait")
        || kind.contains("enum")
        || kind.contains("union")
        || kind.contains("annotation_type"))
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
        && !kind.contains("super")
        && !kind.contains("base")
        && !kind.contains("constructor")
}

/// Heuristically identifies free-standing functions or methods.
pub fn is_function(node: Node) -> bool {
    if !node.is_named() {
        return false;
    }
    let kind = node.kind();
    (kind.contains("function")
        || kind.contains("method")
        || kind.contains("fn_item")
        || kind.contains("func")
        || kind.contains("constructor"))
        && !kind.contains("class")
}

/// Identifies implementation blocks commonly found in Rust.
pub fn is_impl_block(node: Node) -> bool {
    if !node.is_named() {
        return false;
    }
    let kind = node.kind();
    kind == "impl_item" || kind == "impl_block"
}

/// Identifies import or use statements.
pub fn is_import(node: Node) -> bool {
    if !node.is_named() {
        return false;
    }
    matches!(
        node.kind(),
        "use_declaration" | "import_declaration" | "import_statement" | "import_from_statement" | "preproc_include"
    )
}

/// Identifies global or static variables.
pub fn is_free_variable(node: Node) -> bool {
    if !node.is_named() {
        return false;
    }
    let kind = node.kind();
    kind == "static_item" || kind == "const_item" || kind == "global_variable_declaration"
}

/// Identifies type aliases (e.g. typedef, using, type = ...).
pub fn is_type_alias(node: Node) -> bool {
    if !node.is_named() {
        return false;
    }
    let kind = node.kind();
    matches!(
        kind,
        "type_alias_declaration" | "alias_declaration" | "type_item" | "type_alias_statement"
    ) || (kind == "type_definition" && !is_structured_type(node)) // C/C++ typedef can be struct or just alias
}
