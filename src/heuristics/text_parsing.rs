//! Provides foundational utilities for raw text extraction and manipulation from Tree-sitter nodes.
//!
//! This module contains helper functions designed to isolate the low-level string
//! parsing (e.g., splitting qualified names, extracting raw text) from the higher-level
//! structural and semantic extraction phases.

use crate::model::QualifiedName;
use tree_sitter::Node;

/// Extracts the raw UTF-8 text from a Tree-sitter node safely.
///
/// # Arguments
/// * `node` - The Tree-sitter AST node to inspect.
/// * `source` - The complete source code string.
pub fn node_text(node: Node, source: &str) -> String {
    node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
}

/// Splits a textual representation of a qualified name into a structured vector.
///
/// Strips generic type parameters (e.g., `<T>`) to ensure clean name resolution,
/// and splits the string using standard delimiters (`.` or `::`).
///
/// # Arguments
/// * `text` - The raw string representation (e.g., "`std::vector<int>`" or "Outer.Inner").
pub fn split_qualified_name(text: &str) -> QualifiedName {
    // Remove generic parameters for basic name resolution
    let text = text.split('<').next().unwrap_or(text);
    
    // Normalize C/C++ pointer access to dot notation for uniform splitting
    let text_norm = text.replace("->", ".");

    text_norm.split(&[':', '.'][..])
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect()
}

/// Helper to extract a name from a textual string.
pub fn extract_name_from_text(text: &str) -> Option<QualifiedName> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(split_qualified_name(trimmed))
}

/// Helper to extract an identifier from a declarator node, unwrapping nested declarators (typical of C/C++).
fn extract_identifier_from_declarator(mut decl: Node, source: &str) -> Option<String> {
    while let Some(next) = decl.child_by_field_name("declarator") {
        decl = next;
    }

    if decl.kind() == "scoped_identifier" || decl.kind() == "qualified_identifier" {
        let text = node_text(decl, source).trim().to_string();
        if !text.is_empty() {
            return Some(text);
        }
    }

    if let Some(n) = decl.child_by_field_name("name") {
        let text = node_text(n, source).trim().to_string();
        if !text.is_empty() {
            return Some(text);
        }
    }

    let text = node_text(decl, source).trim().to_string();
    if decl.kind() == "identifier" || decl.kind() == "type_identifier" || decl.kind() == "pattern" || decl.kind() == "field_identifier" || decl.kind() == "destructor_name" {
        if !text.is_empty() {
            return Some(text);
        }
    }

    // Take just the name before '(' or '='
    let name_part = text
        .split('(')
        .next()
        .unwrap_or(text.as_str())
        .split('=')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if !name_part.is_empty() {
        return Some(name_part);
    }
    None
}

/// Tries to extract a simple identifier (string) from common child field names.
pub fn extract_identifier(node: Node, source: &str) -> Option<String> {
    if matches!(
        node.kind(),
        "identifier" | "type_identifier" | "pattern" | "name" | "scoped_identifier"
    ) {
        let text = node_text(node, source).trim().to_string();
        if !text.is_empty() {
            return Some(text);
        }
    }

    // For C/C++ style declarations with nested declarators
    if let Some(decl) = node.child_by_field_name("declarator") {
        if let Some(ident) = extract_identifier_from_declarator(decl, source) {
            return Some(ident);
        }
    }

    if let Some(n) = node
        .child_by_field_name("name")
        .or_else(|| node.child_by_field_name("type_identifier"))
        .or_else(|| node.child_by_field_name("identifier"))
        .or_else(|| node.child_by_field_name("pattern"))
        .or_else(|| node.child_by_field_name("left"))
    {
        let text = node_text(n, source).trim().to_string();
        if !text.is_empty() {
            return Some(text);
        }
    }

    // Fallback: look for an identifier child
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            let text = node_text(child, source).trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
    }

    None
}

/// Extracts a qualified name (e.g. A::B::C) by traversing identifiers.
pub fn extract_qualified_name(node: Node, source: &str) -> Option<QualifiedName> {
    if matches!(
        node.kind(),
        "identifier" | "type_identifier" | "name" | "scoped_identifier"
    ) {
        return Some(split_qualified_name(&node_text(node, source)));
    }

    // Try with Tree-sitter fields first (more precise)
    if let Some(n) = node
        .child_by_field_name("name")
        .or_else(|| node.child_by_field_name("type_identifier"))
        .or_else(|| node.child_by_field_name("identifier"))
        .or_else(|| node.child_by_field_name("pattern"))
    {
        let text = &node_text(n, source);
        return Some(split_qualified_name(text));
    }

    // Fallback: search for a sequence of identifiers separated by :: or .
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
