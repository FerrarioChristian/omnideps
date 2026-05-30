use crate::ir::{Function, ImplBlock, Import, Module, StructuredType};
use tree_sitter::Node;

use super::classifiers::*;
use super::extractors::*;

pub fn try_parse_module_node(node: Node, source: &str) -> Option<Module> {
    if !is_module(node) {
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

pub fn try_parse_structured_type(node: Node, source: &str) -> Option<StructuredType> {
    if !is_structured_type(node) {
        return None;
    }

    let kind_text = node.kind();
    let text = node_text(node, source);
    let name = extract_qualified_name(node, source).unwrap_or_else(|| vec!["unnamed_type".to_string()]);
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

    let body = if let Some(body_node) = node.child_by_field_name("body") {
        Some(extract_block(body_node, source))
    } else {
        None
    };

    Some(Function {
        name: vec![name],
        signature: crate::ir::Signature {
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
        let alias_part: String = after_as.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        if !alias_part.is_empty() {
            alias = Some(alias_part);
        }
    }

    // Try to extract path using tree-sitter fields, fallback to regex-like
    let path = if let Some(p_node) = node.child_by_field_name("argument")
        .or_else(|| node.child_by_field_name("name"))
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
