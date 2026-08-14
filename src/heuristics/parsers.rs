use crate::model::{Function, ImplBlock, Import, Module, StructuredType};
use tree_sitter::Node;

use super::body_extraction::*;
use super::classifiers::*;
use super::structural_extraction::*;
use super::text_parsing::*;
use super::type_extraction::*;

pub fn try_parse_module_node(node: Node, source: &str, lang_name: &str) -> Option<Module> {
    if !is_module(node) {
        return None;
    }

    let name = extract_identifier(node, source).unwrap_or_else(|| "unnamed_module".to_string());
    Some(Module {
        name: vec![name],
        language: Some(lang_name.to_string()),
        file_path: None,
        imports: vec![],
        sub_modules: vec![],
        structured_types: vec![],
        type_aliases: vec![],
        free_functions: vec![],
        impl_blocks: vec![],
        free_variables: vec![],
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

pub fn try_parse_structured_type(
    node: Node,
    source: &str,
    lang_name: &str,
    config: &crate::config::AnalyzerConfig,
) -> Option<StructuredType> {
    if !is_structured_type(node) {
        return None;
    }

    let kind_text = node.kind();
    let text = node_text(node, source);
    let name =
        extract_qualified_name(node, source).unwrap_or_else(|| vec!["unnamed_type".to_string()]);
    let mut fields = extract_fields(node, source, lang_name, config);

    // Handle C/C++ typedef struct/enum by extracting fields from the inner specifier
    if node.kind() == "type_definition" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "struct_specifier"
                || child.kind() == "enum_specifier"
                || child.kind() == "union_specifier"
            {
                fields.extend(extract_fields(child, source, lang_name, config));
            }
        }
    }

    let methods = extract_methods(node, source);
    let super_types = extract_super_types(node, source);
    let mut nested_types = extract_nested_types(node, source, lang_name, config);

    // Remove unnamed nested types. These are usually the inner specifiers of a typedef
    // (e.g., typedef struct { ... } Name) which we already hoisted the fields for above,
    // or unreferenceable anonymous structs that shouldn't clutter the graph.
    // Also remove the inner specifier if it has the exact same name as the typedef
    // (e.g., typedef struct Point { ... } Point).
    if node.kind() == "type_definition" {
        nested_types.retain(|nt| nt.name != vec!["unnamed_type".to_string()] && nt.name != name);
    } else {
        nested_types.retain(|nt| nt.name != vec!["unnamed_type".to_string()]);
    }

    let annotations = super::annotation_extraction::extract_annotations(node, source);
    println!("Type {:?} annotations: {:?}", name, annotations);
    println!("FIELDS FOR {:?}: {:?}", name, fields);

    Some(StructuredType {
        name,
        kind: determine_structured_kind(kind_text, &text),
        fields,
        methods,
        super_types,
        nested_types,
        annotations,
        imports: vec![],
    })
}

pub fn try_parse_function(mut node: Node, source: &str) -> Option<Function> {
    if !is_function(node) {
        return None;
    }

    if node.kind() == "decorated_definition" {
        if let Some(definition) = node.child_by_field_name("definition") {
            node = definition;
        } else {
            // fallback, find the first function child
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind().contains("function") || child.kind().contains("method") {
                    node = child;
                    break;
                }
            }
        }
    }

    let name = extract_identifier(node, source).unwrap_or_else(|| "unnamed_function".to_string());
    let is_constructor = node.kind().contains("constructor");

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

    let annotations = super::annotation_extraction::extract_annotations(node, source);

    Some(Function {
        name: split_qualified_name(&name),
        signature: crate::model::Signature {
            parameters,
            return_type,
        },
        body,
        is_constructor,
        annotations,
    })
}

pub fn try_parse_impl_block(
    node: Node,
    source: &str,
    lang_name: &str,
    config: &crate::config::AnalyzerConfig,
) -> Option<ImplBlock> {
    if !is_impl_block(node) {
        return None;
    }

    let name =
        extract_qualified_name(node, source).unwrap_or_else(|| vec!["unnamed_impl".to_string()]);
    let methods = extract_methods(node, source);
    let impl_for = extract_impl_for(node, source);
    let implements_trait = extract_implements_trait(node, source);
    let nested_types = extract_nested_types(node, source, lang_name, config);
    let type_aliases = crate::heuristics::structural_extraction::extract_list_of(
        node,
        source,
        false,
        |child, src| try_parse_type_alias(child, src),
    );

    Some(ImplBlock {
        name,
        methods,
        impl_for,
        implements_trait,
        nested_types,
        type_aliases,
    })
}

fn sanitize_import_path(raw_text: &str) -> Vec<String> {
    let mut txt = raw_text.replace("\"", "").replace("<", "").replace(">", "");
    if txt.starts_with("crate::") {
        txt = txt.replace("crate::", "");
    }
    for ext in [".hpp", ".cpp", ".h", ".c", ".ts", ".js"] {
        // Added common JS/TS extensions too as bonus for language agnostic
        if txt.ends_with(ext) {
            txt = txt[..txt.len() - ext.len()].to_string();
            break;
        }
    }
    txt = txt.replace("/", "::");
    split_qualified_name(&txt)
}

pub fn try_parse_imports(node: Node, source: &str) -> Option<Vec<Import>> {
    let kind = node.kind();
    if !kind.contains("import") && !kind.contains("use") && kind != "using_declaration" {
        return None;
    }

    let text = node_text(node, source);
    let is_wildcard = text.contains('*')
        || text.contains(".*")
        || text.contains("::*")
        || text.starts_with("using namespace ");

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

    let mut imports = vec![];

    if let Some(p_node) = node.child_by_field_name("module_name") {
        let base_path = split_qualified_name(&node_text(p_node, source));
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let child_kind = child.kind();
            if matches!(
                child_kind,
                "aliased_import" | "dotted_name" | "name" | "identifier"
            ) && child.id() != p_node.id()
            {
                let txt = node_text(child, source);
                let mut full_path = base_path.clone();
                full_path.extend(split_qualified_name(&txt));
                imports.push(Import {
                    path: full_path,
                    alias: alias.clone(),
                    is_wildcard,
                });
            }
        }
        if imports.is_empty() {
            imports.push(Import {
                path: base_path,
                alias,
                is_wildcard,
            });
        }
    } else {
        let path = if let Some(p_node) = node
            .child_by_field_name("argument")
            .or_else(|| node.child_by_field_name("name"))
            .or_else(|| node.child_by_field_name("path"))
        {
            let p_text = node_text(p_node, source);
            sanitize_import_path(&p_text)
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
                    let txt = node_text(child, source);
                    p = sanitize_import_path(&txt);
                    // For using_declaration we DO want to break here since the identifier is the path
                    break;
                }
            }
            p
        };

        if !path.is_empty() {
            imports.push(Import {
                path,
                alias,
                is_wildcard,
            });
        }
    }

    if imports.is_empty() {
        None
    } else {
        Some(imports)
    }
}

pub fn try_parse_free_variable(node: Node, source: &str) -> Option<crate::model::Field> {
    if !is_free_variable(node) {
        return None;
    }

    if let Some(name) = extract_identifier(node, source) {
        let ty = extract_type_ref(node, source);
        let annotations = super::annotation_extraction::extract_annotations(node, source);
        return Some(crate::model::Field {
            name,
            ty,
            annotations,
        });
    }

    None
}

pub fn try_parse_type_alias(node: Node, source: &str) -> Option<crate::model::TypeAlias> {
    if !is_type_alias(node) {
        return None;
    }

    let name = extract_identifier(node, source);
    println!("TypeAlias: name={:?} from node: {:?}", name, node.kind());
    let name = name?;

    // For type alias, we can typically extract the type ref right from the node
    let target = extract_type_ref(node, source);
    println!("TypeAlias: name={:?} target={:?}", name, target);

    Some(crate::model::TypeAlias {
        name: vec![name],
        target,
    })
}

#[cfg(test)]
mod parse_tests {
    // Tests vuoti per ora, quelli temporanei di debug sono stati rimossi.
}
