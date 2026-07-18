//! Implements the 1st-Pass of the Name Resolution algorithm (Global Hoisting).
//!
//! The `GlobalRegistry` scans the extracted IR prior to lexical scoping and creates a flat index
//! of all absolute paths present in the workspace. This dictionary is crucial during the
//! resolution phase to accurately distinguish between `Local` entities and `External` boundaries.

use crate::model::{Module, QualifiedName, StructuredType, TypeRef};
use std::collections::HashMap;

/// Denotes the type of component registered, used for type inference.
#[derive(Debug, Clone)]
pub enum RegistryEntry {
    Module { imports: Vec<crate::model::Import> },
    StructuredType { super_types: Vec<TypeRef> },
    Function { return_type: TypeRef },
    Field { field_type: TypeRef },
}

/// The central dictionary indexing all absolute paths within the analyzed project.
pub struct GlobalRegistry {
    pub paths: HashMap<QualifiedName, RegistryEntry>,
}

impl GlobalRegistry {
    /// Builds the global registry by recursively iterating through all modules and components.
    pub fn build(modules: &[Module]) -> Self {
        let mut registry = Self { paths: HashMap::new() };
        for m in modules {
            registry.register_module(m, vec![]);
        }
        registry
    }

    /// Recursively registers a Module, its functions, structured types, and Impl blocks.
    fn register_module(&mut self, m: &Module, mut prefix: QualifiedName) {
        prefix.extend(m.name.clone());
        
        let entry = self.paths.entry(prefix.clone()).or_insert_with(|| RegistryEntry::Module { imports: vec![] });
        if let RegistryEntry::Module { imports: existing_imports } = entry {
            for imp in &m.imports {
                if !existing_imports.contains(imp) {
                    existing_imports.push(imp.clone());
                }
            }
        }

        for st in &m.structured_types {
            self.register_structured_type(st, prefix.clone());
        }
        for ff in &m.free_functions {
            let mut ff_prefix = prefix.clone();
            ff_prefix.extend(ff.name.clone());
            self.paths.insert(ff_prefix, RegistryEntry::Function { return_type: ff.signature.return_type.clone() });
        }
        for ib in &m.impl_blocks {
            // Register ImplBlock methods and nested types
            let target_name = match &ib.impl_for {
                TypeRef::Unresolved(qn) | TypeRef::Resolved(qn) => qn.last().cloned().unwrap_or_default(),
                TypeRef::ResolutionQuery(q) => crate::resolver::executor::extract_base_name(q),
                _ => "".to_string(),
            };
            if !target_name.is_empty() {
                let mut target_prefix = prefix.clone();
                target_prefix.push(target_name);
                for method in &ib.methods {
                    let mut m_prefix = target_prefix.clone();
                    m_prefix.extend(method.name.clone());
                    self.paths.insert(m_prefix, RegistryEntry::Function { return_type: method.signature.return_type.clone() });
                }
                for nested in &ib.nested_types {
                    self.register_structured_type(nested, target_prefix.clone());
                }
            }
        }
        for sub in &m.sub_modules {
            self.register_module(sub, prefix.clone());
        }
    }

    /// Recursively registers a StructuredType, its methods, and any nested types.
    fn register_structured_type(&mut self, st: &StructuredType, mut prefix: QualifiedName) {
        prefix.extend(st.name.clone());
        self.paths.insert(prefix.clone(), RegistryEntry::StructuredType { super_types: st.super_types.clone() });

        for field in &st.fields {
            let mut f_prefix = prefix.clone();
            f_prefix.push(field.name.clone());
            self.paths.insert(f_prefix, RegistryEntry::Field { field_type: field.ty.clone() });
        }

        for method in &st.methods {
            let mut m_prefix = prefix.clone();
            m_prefix.extend(method.name.clone());
            self.paths.insert(m_prefix, RegistryEntry::Function { return_type: method.signature.return_type.clone() });
        }
        for nested in &st.nested_types {
            self.register_structured_type(nested, prefix.clone());
        }
    }

    /// Validates whether a given absolute path points to a known Local component.
    pub fn exists(&self, path: &QualifiedName) -> bool {
        self.paths.contains_key(path)
    }

    /// Retrieves the entry for type inference.
    pub fn get(&self, path: &QualifiedName) -> Option<&RegistryEntry> {
        self.paths.get(path)
    }
}