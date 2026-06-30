use tree_sitter::Node;

/// Heuristically determines if a node is a module or namespace definition.
pub fn is_module(node: Node) -> bool {
    if !node.is_named() {
        return false;
    }
    let kind = node.kind();
    kind.contains("mod_item") || kind.contains("module") || kind.contains("namespace")
}

/// Heuristically determines if a node is a structured type definition (Struct, Class, Interface, etc.).
pub fn is_structured_type(node: Node) -> bool {
    if !node.is_named() {
        return false;
    }
    let kind = node.kind();
    (kind.contains("struct")
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
        && !kind.contains("variant")
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
        || kind.contains("func"))
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
