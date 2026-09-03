use crate::model::TypeRef;
use tree_sitter::Node;

use super::text_parsing::extract_qualified_name;

/// Extracts annotations/decorators from a given AST node depending on its language structure.
pub fn extract_annotations(node: Node, source: &str) -> Vec<TypeRef> {
    let mut annotations = Vec::new();

    let _kind = node.kind();

    // 1. Python: `decorated_definition` wraps the `decorator` and the actual `definition`.
    if let Some(parent) = node.parent()
        && parent.kind() == "decorated_definition"
    {
        let mut cursor = parent.walk();
        for child in parent.children(&mut cursor) {
            if child.kind() == "decorator"
                && let Some(dec_name) = extract_python_decorator(child, source)
            {
                annotations.push(TypeRef::Unresolved(dec_name));
            }
        }
    }

    // Rust: attributes are often previous siblings
    let mut prev = node.prev_sibling();
    while let Some(sibling) = prev {
        let kind = sibling.kind();
        log::trace!("Checking sibling of {:?}: {:?}", node.kind(), kind);
        if kind == "attribute_item" || kind == "inner_attribute_item" {
            let mut cursor = sibling.walk();
            for attr in sibling.children(&mut cursor) {
                log::trace!("Checking attr kind: {}", attr.kind());
                if attr.kind() == "attribute" || attr.kind() == "meta_item" {
                    // check for `meta_arguments` for things like `#[derive(Debug)]`
                    if let Some(meta) = attr.child_by_field_name("arguments") {
                        let mut m_cursor = meta.walk();
                        for m_arg in meta.children(&mut m_cursor) {
                            if let Some(qname) = extract_qualified_name(m_arg, source) {
                                annotations.push(TypeRef::Unresolved(qname));
                            }
                        }
                    }
                    if let Some(name_node) = attr.child_by_field_name("name") {
                        if let Some(qname) = extract_qualified_name(name_node, source) {
                            annotations.push(TypeRef::Unresolved(qname));
                        }
                    } else if let Some(qname) = extract_qualified_name(attr, source) {
                        // for rust attributes, we only care about the first part (e.g. `serde` in `serde(...)`)
                        if !qname.is_empty() {
                            annotations.push(TypeRef::Unresolved(vec![qname[0].clone()]));
                        }
                    }
                }
            }
        } else if !kind.contains("comment") {
            break;
        }
        prev = sibling.prev_sibling();
    }

    // 2. Java / Rust / C++
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let child_kind = child.kind();
        log::trace!("Checking child kind: {}", child_kind);

        // Java annotations are sometimes inside a `modifiers` node.
        if child_kind == "modifiers" {
            let mut mod_cursor = child.walk();
            for m in child.children(&mut mod_cursor) {
                if m.kind() == "marker_annotation" || m.kind() == "annotation" {
                    if let Some(name_node) = m.child_by_field_name("name") {
                        log::trace!("Found name_node: {}", name_node.kind());
                        if let Some(qname) = extract_qualified_name(name_node, source) {
                            log::trace!("qname extracted: {:?}", qname);
                            annotations.push(TypeRef::Unresolved(qname));
                        }
                    } else if let Some(qname) = extract_qualified_name(m, source) {
                        log::trace!("qname extracted from m: {:?}", qname);
                        // Sometimes tree-sitter-java marker_annotation doesn't have a `name` field but just a direct identifier child
                        let mut name_extracted = qname.clone();
                        // Strip leading '@' if it exists in the first element
                        if let Some(first) = name_extracted.first_mut()
                            && first.starts_with('@')
                        {
                            *first = first[1..].to_string();
                        }
                        annotations.push(TypeRef::Unresolved(name_extracted));
                    } else {
                        log::trace!("extract_qualified_name failed for {:?}", m.kind());
                    }
                }
            }
        }

        // C++ / Rust attributes / Java annotations that are direct children
        if child_kind == "attribute_item"
            || child_kind == "inner_attribute_item"
            || child_kind == "marker_annotation"
            || child_kind == "annotation"
        {
            let name_node = child.child_by_field_name("name");
            if let Some(n) = name_node {
                if let Some(qname) = extract_qualified_name(n, source) {
                    annotations.push(TypeRef::Unresolved(qname));
                }
            } else {
                if let Some(qname) = extract_qualified_name(child, source) {
                    annotations.push(TypeRef::Unresolved(qname));
                }
            }
        }

        // C / C++ attribute_declaration
        if child_kind == "attribute_declaration" {
            let mut attr_cursor = child.walk();
            for attr in child.children(&mut attr_cursor) {
                if attr.kind() == "attribute" {
                    if let Some(name_node) = attr.child_by_field_name("name") {
                        if let Some(qname) = extract_qualified_name(name_node, source) {
                            annotations.push(TypeRef::Unresolved(qname));
                        }
                    } else if let Some(qname) = extract_qualified_name(attr, source) {
                        annotations.push(TypeRef::Unresolved(qname));
                    }
                }
            }
        }
    }

    annotations
}

fn extract_python_decorator(decorator_node: Node, source: &str) -> Option<Vec<String>> {
    // The decorator could have an `identifier`, a `call`, or a `dotted_name`.
    let mut cursor = decorator_node.walk();
    for child in decorator_node.children(&mut cursor) {
        let kind = child.kind();
        if kind == "identifier" || kind == "dotted_name" {
            let txt = super::text_parsing::node_text(child, source);
            return Some(super::text_parsing::split_qualified_name(&txt));
        } else if kind == "call"
            && let Some(func) = child.child_by_field_name("function")
        {
            let txt = super::text_parsing::node_text(func, source);
            return Some(super::text_parsing::split_qualified_name(&txt));
        }
    }
    None
}
