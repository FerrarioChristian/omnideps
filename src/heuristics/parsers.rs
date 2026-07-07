use crate::model::{Function, ImplBlock, Import, Module, StructuredType};
use tree_sitter::Node;

use super::body_extraction::*;
use super::classifiers::*;
use super::structural_extraction::*;
use super::text_parsing::*;
use super::type_extraction::*;

pub fn try_parse_module_node(node: Node, source: &str) -> Option<Module> {
    if !is_module(node) {
        return None;
    }

    let name = extract_identifier(node, source).unwrap_or_else(|| "unnamed_module".to_string());
    Some(Module {
        name: vec![name],
        language: None,
        file_path: None,
        imports: vec![],
        sub_modules: vec![],
        structured_types: vec![],
        free_functions: vec![],
        impl_blocks: vec![],
    })
}

/// Attempts to parse a file-level package declaration and returns its qualified name path.
pub fn try_parse_package_declaration(node: Node, source: &str) -> Option<Vec<String>> {
    if !is_package_declaration(node) {
        return None;
    }

    // The package_declaration contains the package name as its first named child
    // (e.g. `identifier` or `scoped_identifier` in Java, or `package_clause` in Go)
    if let Some(child) = node.named_child(0) {
        let text = super::text_parsing::node_text(child, source);
        let parts = super::text_parsing::split_qualified_name(&text);
        if !parts.is_empty() {
            return Some(parts);
        }
    }

    None
}

pub fn try_parse_structured_type(node: Node, source: &str) -> Option<StructuredType> {
    if !is_structured_type(node) {
        return None;
    }

    let kind_text = node.kind();
    let text = node_text(node, source);
    let name =
        extract_qualified_name(node, source).unwrap_or_else(|| vec!["unnamed_type".to_string()]);
    let fields = extract_fields(node, source);
    let methods = extract_methods(node, source);
    let super_types = extract_super_types(node, source);
    let nested_types = extract_nested_types(node, source);

    Some(StructuredType {
        name,
        kind: determine_structured_kind(kind_text, &text),
        fields,
        methods,
        super_types,
        nested_types,
    })
}

pub fn try_parse_function(node: Node, source: &str) -> Option<Function> {
    if !is_function(node) {
        return None;
    }

    let name = extract_identifier(node, source).unwrap_or_else(|| "unnamed_function".to_string());
    let parameters = extract_parameters(node, source);
    let return_type = extract_return_type(node, source);

    let body_node = node.child_by_field_name("body");
    let body = if let Some(b) = body_node {
        Some(extract_block(b, source))
    } else {
        // Fallback: search for block or constructor_body children
        let mut cursor = node.walk();
        let mut found_body = None;
        for child in node.children(&mut cursor) {
            if matches!(
                child.kind(),
                "block" | "constructor_body" | "statement_block"
            ) {
                found_body = Some(extract_block(child, source));
                break;
            }
        }
        found_body
    };

    Some(Function {
        name: vec![name],
        signature: crate::model::Signature {
            parameters,
            return_type,
        },
        body,
    })
}

pub fn try_parse_impl_block(node: Node, source: &str) -> Option<ImplBlock> {
    if !is_impl_block(node) {
        return None;
    }

    let name =
        extract_qualified_name(node, source).unwrap_or_else(|| vec!["unnamed_impl".to_string()]);
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

pub fn try_parse_import(node: Node, source: &str) -> Option<Import> {
    if !is_import(node) {
        return None;
    }

    let text = node_text(node, source);
    let is_wildcard = text.contains('*') || text.contains(".*") || text.contains("::*");

    // Attempt to find alias
    let mut alias = None;
    if let Some(as_pos) = text.find(" as ") {
        let after_as = text[as_pos + 4..].trim();
        let alias_part: String = after_as
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !alias_part.is_empty() {
            alias = Some(alias_part);
        }
    }

    // Try to extract path using tree-sitter fields, fallback to regex-like
    let path = if let Some(p_node) = node
        .child_by_field_name("argument")
        .or_else(|| node.child_by_field_name("name"))
        .or_else(|| node.child_by_field_name("path"))
        .or_else(|| node.child_by_field_name("module_name"))
    {
        let mut p_text = node_text(p_node, source);
        if p_text.starts_with("crate::") {
            p_text = p_text.replace("crate::", "");
        }
        split_qualified_name(&p_text)
    } else {
        // Fallback for preproc_include or generic imports
        let mut p = vec![];
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let c_kind = child.kind();
            if matches!(
                c_kind,
                "scoped_identifier"
                    | "identifier"
                    | "dotted_name"
                    | "system_lib_string"
                    | "string_literal"
            ) {
                let txt = node_text(child, source)
                    .replace("\"", "")
                    .replace("<", "")
                    .replace(">", "");
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

#[cfg(test)]
mod parse_tests {
    // Tests vuoti per ora, quelli temporanei di debug sono stati rimossi.
}
