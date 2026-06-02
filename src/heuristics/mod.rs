pub mod classifiers;
pub mod text_parsing;
pub mod type_extraction;
pub mod structural_extraction;
pub mod body_extraction;
pub mod parsers;

use crate::ir::{Component, ImplBlock, Import};
use tree_sitter::Node;

pub enum ParsedItem {
    Component(Component),
    ImplBlock(ImplBlock),
    Import(Import),
}

/// Attempts to identify and parse the given Tree-sitter `Node` into an Intermediate Representation (IR) Component.
pub fn dispatch_node(node: Node, source: &str) -> Option<ParsedItem> {
    if let Some(import) = parsers::try_parse_import(node, source) {
        return Some(ParsedItem::Import(import));
    }
    if let Some(m) = parsers::try_parse_module_node(node, source) {
        return Some(ParsedItem::Component(Component::Module(m)));
    }
    if let Some(st) = parsers::try_parse_structured_type(node, source) {
        return Some(ParsedItem::Component(Component::StructuredType(st)));
    }
    if let Some(ff) = parsers::try_parse_function(node, source) {
        return Some(ParsedItem::Component(Component::Function(ff)));
    }
    if let Some(implb) = parsers::try_parse_impl_block(node, source) {
        return Some(ParsedItem::ImplBlock(implb));
    }
    None
}
