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
    
    text.split(&[':', '.'][..])
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

/// Tries to extract a simple identifier (string) from common child field names.
pub fn extract_identifier(node: Node, source: &str) -> Option<String> {
    if matches!(node.kind(), "identifier" | "type_identifier" | "pattern" | "name" | "scoped_identifier") {
        let text = node_text(node, source).trim().to_string();
        if !text.is_empty() { return Some(text); }
    }

    if let Some(n) = node
        .child_by_field_name("name")
        .or_else(|| node.child_by_field_name("type_identifier"))
        .or_else(|| node.child_by_field_name("identifier"))
        .or_else(|| node.child_by_field_name("pattern"))
        .or_else(|| node.child_by_field_name("left"))
    {
        let text = node_text(n, source).trim().to_string();
        if !text.is_empty() { return Some(text); }
    }

    if let Some(decl) = node.child_by_field_name("declarator") {
        if let Some(n) = decl.child_by_field_name("declarator") {
            let text = node_text(n, source).trim().to_string();
            if !text.is_empty() { return Some(text); }
        }
        
        if let Some(n) = decl.child_by_field_name("name") {
            let text = node_text(n, source).trim().to_string();
            if !text.is_empty() { return Some(text); }
        }

        let text = node_text(decl, source).trim().to_string();
        // Take just the name before '(' or '='
        let name_part = text.split('(').next().unwrap_or(text.as_str()).split('=').next().unwrap_or("").trim().to_string();
        if !name_part.is_empty() { return Some(name_part); }
    }

    None
}

/// Extracts a qualified name (e.g. A::B::C) by traversing identifiers.
pub fn extract_qualified_name(node: Node, source: &str) -> Option<QualifiedName> {
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