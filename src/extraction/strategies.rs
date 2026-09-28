use crate::model::{ImplBlock, Module, Query, TypeRef};
use std::path::Path;

/// Wraps extracted components in a nested module hierarchy matching the file's relative path on disk.
///
/// Common source root prefixes (`src`, `lib`, `include`) are stripped, and the file stem becomes the leaf module name.
pub fn apply_directory_strategy(
    modules: &mut Vec<Module>,
    path: &Path,
    file_path_str: &str,
    lang_name: &str,
) {
    let mut path_components: Vec<String> = path
        .components()
        .filter_map(|c| {
            let s = c.as_os_str().to_string_lossy().to_string();
            if s == "." || s == ".." { None } else { Some(s) }
        })
        .collect();

    // Remove common source roots from the beginning of the path
    if let Some(first) = path_components.first()
        && matches!(first.as_str(), "src" | "lib" | "include")
    {
        path_components.remove(0);
    }

    if let Some(last) = path_components.last_mut()
        && let Some(stem) = Path::new(last).file_stem()
    {
        *last = stem.to_string_lossy().to_string();
    }

    if modules.is_empty() || path_components.is_empty() {
        return;
    }

    let mut current = modules.remove(0);
    current.name = vec![path_components.pop().unwrap()];

    for comp in path_components.into_iter().rev() {
        let outer = Module {
            name: vec![comp],
            language: Some(lang_name.to_string()),
            file_path: Some(file_path_str.to_string()),
            imports: vec![],
            sub_modules: vec![current],
            structured_types: vec![],
            type_aliases: vec![],
            free_functions: vec![],
            impl_blocks: vec![],
            free_variables: vec![],
        };
        current = outer;
    }

    let global_root = Module {
        name: vec!["root".to_string()],
        language: Some(lang_name.to_string()),
        file_path: None,
        imports: vec![],
        sub_modules: vec![current],
        structured_types: vec![],
        type_aliases: vec![],
        free_functions: vec![],
        impl_blocks: vec![],
        free_variables: vec![],
    };
    modules.push(global_root);
}

/// Groups extracted components according to the file's explicit package declaration (e.g., Java `package com.foo.bar;`).
///
/// Wraps the file's components into an outer module hierarchy matching the declared package components.
pub fn apply_package_strategy(
    modules: &mut Vec<Module>,
    package_path: Vec<String>,
    file_path_str: &str,
    lang_name: &str,
) {
    if package_path.is_empty() || modules.is_empty() {
        return;
    }

    // The components are currently inside the "root" module extracted by extract_from_cst
    let content_module = modules.remove(0);

    // We unpack the contents of this module (classes, functions, etc.) directly into the package
    let mut current = Module {
        name: vec![package_path.last().unwrap().clone()],
        language: Some(lang_name.to_string()),
        file_path: Some(file_path_str.to_string()),
        imports: content_module.imports,
        sub_modules: content_module.sub_modules, // If there are any nested modules parsed from file
        structured_types: content_module.structured_types,
        type_aliases: content_module.type_aliases,
        free_functions: content_module.free_functions,
        impl_blocks: content_module.impl_blocks,
        free_variables: content_module.free_variables,
    };

    // Wrap in outer packages
    for comp in package_path.into_iter().rev().skip(1) {
        let outer = Module {
            name: vec![comp],
            language: Some(lang_name.to_string()),
            file_path: Some(file_path_str.to_string()),
            imports: vec![],
            sub_modules: vec![current],
            structured_types: vec![],
            type_aliases: vec![],
            free_functions: vec![],
            impl_blocks: vec![],
            free_variables: vec![],
        };
        current = outer;
    }

    // Put everything back under the "root" universe module to keep parity with DirectoryBased
    let global_root = Module {
        name: vec!["root".to_string()],
        language: Some(lang_name.to_string()),
        file_path: None,
        imports: vec![],
        sub_modules: vec![current],
        structured_types: vec![],
        type_aliases: vec![],
        free_functions: vec![],
        impl_blocks: vec![],
        free_variables: vec![],
    };
    modules.push(global_root);
}

/// Universally reconciles out-of-line method definitions with their declaring structured type.
///
/// In languages such as C++, methods may be defined outside the class body using qualified names
/// (e.g. `ClassName::method`). This pass detects free functions with multi-part identifiers (`name.len() > 1`)
/// and attaches them to their matching [`crate::model::StructuredType`] within the module or generates
/// a synthetic [`crate::model::ImplBlock`] for cross-module definitions.
///
/// For languages without qualified top-level function names (Java, Python, Rust, C), this pass performs zero modifications.
pub fn link_out_of_line_methods(modules: &mut [Module]) {
    for module in modules.iter_mut() {
        let mut methods_to_move = vec![];

        // Extract functions that have qualified names (e.g., MyClass::my_method)
        module.free_functions.retain(|ff| {
            if ff.name.len() > 1 {
                methods_to_move.push(ff.clone());
                false // Remove from free_functions
            } else {
                true // Keep
            }
        });

        // Find the class and append
        for method in methods_to_move {
            let class_name = &method.name[..method.name.len() - 1];
            let method_name = method.name.last().unwrap().clone();

            let mut found = false;
            for st in &mut module.structured_types {
                if st.name == class_name {
                    let mut m = method.clone();
                    m.name = vec![method_name.clone()];
                    st.methods.push(m);
                    found = true;
                    break;
                }
            }

            // If not found in current module, maybe it's cross-module?
            // For simplicity in C++, we assume the definition is in the same namespace block,
            // or we could use the ImplBlock logic. Let's create an ImplBlock!
            if !found {
                module.impl_blocks.push(ImplBlock {
                    name: class_name.to_vec(),
                    impl_for: TypeRef::ResolutionQuery(Query::Find(
                        class_name.last().unwrap().clone(),
                    )),
                    implements_trait: None,
                    methods: vec![{
                        let mut m = method.clone();
                        m.name = vec![method_name];
                        m
                    }],
                    nested_types: vec![],
                    type_aliases: vec![],
                });
            }
        }

        link_out_of_line_methods(&mut module.sub_modules);
    }
}
