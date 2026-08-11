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
            if child_kind.contains("struct")
                || child_kind.contains("enum")
                || child_kind.contains("union")
            {
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

    // In C/C++, function declarations are "declaration" nodes containing a "function_declarator"
    if kind == "declaration" && has_nested_function_declarator(node) {
        return true;
    }

    let is_func_kind = kind.contains("function")
        || kind.contains("method")
        || kind.contains("fn_item")
        || kind.contains("func")
        || kind.contains("constructor")
        || kind == "decorated_definition";

    is_func_kind && !kind.contains("class")
}

/// Helper function to traverse deeply nested declarators (typical of C/C++)
/// and check if any of them is a "function_declarator".
fn has_nested_function_declarator(node: Node) -> bool {
    let mut curr = node.child_by_field_name("declarator");
    while let Some(decl) = curr {
        if decl.kind() == "function_declarator" {
            return true;
        }
        curr = decl.child_by_field_name("declarator");
    }
    false
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
        "use_declaration"
            | "import_declaration"
            | "import_statement"
            | "import_from_statement"
            | "preproc_include"
    )
}

/// Identifies global or static variables.
pub fn is_free_variable(node: Node) -> bool {
    if !node.is_named() {
        return false;
    }
    let kind = node.kind();
    
    let mut is_decl = false;
    if kind == "declaration" || kind == "variable_declaration" {
        // In C/C++, exclude function declarations (they contain a function_declarator)
        if has_nested_function_declarator(node) {
            return false;
        }
        // Exclude local variables by ensuring no parent is a function or method body
        let mut parent = node.parent();
        while let Some(p) = parent {
            let pk = p.kind();
            if pk.contains("function") || pk.contains("method") || pk.contains("body") || pk == "compound_statement" || pk == "block" {
                return false;
            }
            parent = p.parent();
        }
        is_decl = true;
    }

    kind == "static_item" || kind == "const_item" || kind == "global_variable_declaration" || is_decl
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
