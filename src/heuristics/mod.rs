pub mod classifiers;
pub mod text_parsing;
pub mod type_extraction;
pub mod structural_extraction;
pub mod body_extraction;
pub mod parsers;
pub mod annotation_extraction;
use crate::model::{Component, ImplBlock, Import};
use tree_sitter::Node;

pub enum ParsedItem {
    Component(Component),
    ImplBlock(ImplBlock),
    Imports(Vec<Import>),
    Attribute(Vec<crate::model::TypeRef>),
}

/// Attempts to identify and parse the given Tree-sitter `Node` into an Intermediate Representation (IR) Component.
pub fn dispatch_node(node: Node, source: &str, lang_name: &str, config: &crate::config::AnalyzerConfig) -> Option<ParsedItem> {
    if let Some(imports) = parsers::try_parse_imports(node, source) {
        return Some(ParsedItem::Imports(imports));
    }
    if node.kind() == "attribute_item" {
        let attrs = annotation_extraction::extract_annotations(node, source);
        if !attrs.is_empty() {
            return Some(ParsedItem::Attribute(attrs));
        }
    }
    let lang_modules = &config.get_for(lang_name).modules;
    if (lang_modules.inline_mod_based || lang_modules.namespace_based)
        && let Some(m) = parsers::try_parse_module_node(node, source, lang_name) {
            return Some(ParsedItem::Component(Component::Module(m)));
        }
    if let Some(st) = parsers::try_parse_structured_type(node, source, lang_name, config) {
        return Some(ParsedItem::Component(Component::StructuredType(st)));
    }
    if let Some(ff) = parsers::try_parse_function(node, source) {
        return Some(ParsedItem::Component(Component::Function(ff)));
    }
    if config.get_for(lang_name).support_impl_blocks
        && let Some(implb) = parsers::try_parse_impl_block(node, source, lang_name, config) {
            return Some(ParsedItem::ImplBlock(implb));
        }
    if let Some(ta) = parsers::try_parse_type_alias(node, source) {
        return Some(ParsedItem::Component(Component::TypeAlias(ta)));
    }
    if let Some(fv) = parsers::try_parse_free_variable(node, source) {
        return Some(ParsedItem::Component(Component::Field(vec![fv.name], fv.ty)));
    }
    None
}
