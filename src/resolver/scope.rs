use crate::config::AnalyzerConfig;
use crate::model::{Block, Function, ImplBlock, Module, StructuredType, TypeRef};
use std::collections::HashMap;

/// Unique identifier for a scope node within the [`ScopeTree`] arena.
pub type ScopeId = usize;

/// Represents the classification and payload of a symbol bound within a lexical scope.
#[derive(Debug, Clone)]
pub enum Symbol {
    /// A submodule pointing to its nested [`ScopeId`].
    Module(ScopeId),
    /// A structured type (class, struct, interface) pointing to its internal type [`ScopeId`].
    Type(ScopeId),
    /// A type alias pointing to its target [`TypeRef`].
    TypeAlias(TypeRef),
    /// A concrete term: local variable, parameter, field, or function.
    /// The [`TypeRef`] specifies the value's type (or return type for functions).
    Value(TypeRef),
}

/// A single lexical environment node (Scope) within the hierarchy.
#[derive(Debug, Clone)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub name: String,
    pub symbols: HashMap<String, Symbol>,
    pub imports: Vec<crate::model::Import>,
    pub super_types: Vec<TypeRef>,
    pub is_module: bool,
    pub language: Option<String>,
    pub type_parameters: Vec<String>,
    pub is_phantom: bool,
    pub children: Vec<ScopeId>,
    pub children_by_name: HashMap<String, Vec<ScopeId>>,
}

/// Hierarchical tree of lexical scopes ($\mathcal{E}$), implemented via an arena allocator.
///
/// Models the global symbol environment $\mathcal{E}$ formalised in Chapter 7, enabling
/// scope climbing, inheritance traversing, and global query resolution.
#[derive(Debug, Clone)]
pub struct ScopeTree {
    pub arena: Vec<Scope>,
    pub root: ScopeId,
    pub pending_impl_blocks: Vec<(crate::model::ImplBlock, ScopeId, String)>,
    pub type_scopes_by_name: HashMap<String, Vec<ScopeId>>,
}

impl ScopeTree {
    /// Constructs the global Scope Tree $\mathcal{E}$ from extracted IR modules $\mathcal{D}$.
    pub fn build(modules: &[Module], config: &AnalyzerConfig) -> Self {
        let mut tree = ScopeTree {
            arena: vec![Scope {
                id: 0,
                parent: None,
                name: "root".to_string(),
                symbols: HashMap::new(),
                imports: vec![],
                super_types: vec![],
                is_module: true,
                language: None,
                type_parameters: vec![],
                is_phantom: false,
                children: vec![],
                children_by_name: HashMap::new(),
            }],
            root: 0,
            pending_impl_blocks: vec![],
            type_scopes_by_name: HashMap::new(),
        };

        for m in modules {
            tree.register_module(m, 0, config);
        }

        // Deferred registration of impl blocks across modules
        let pending = std::mem::take(&mut tree.pending_impl_blocks);
        for (ib, parent_id, lang) in pending {
            tree.register_impl_block(&ib, parent_id, config, &lang);
        }

        tree
    }

    /// Creates a new child scope in the arena.
    pub fn new_scope(&mut self, parent: ScopeId, name: String) -> ScopeId {
        let id = self.arena.len();
        self.arena.push(Scope {
            id,
            parent: Some(parent),
            name: name.clone(),
            symbols: HashMap::new(),
            imports: vec![],
            super_types: vec![],
            is_module: false,
            language: None,
            type_parameters: vec![],
            is_phantom: false,
            children: vec![],
            children_by_name: HashMap::new(),
        });
        self.arena[parent].children.push(id);
        self.arena[parent]
            .children_by_name
            .entry(name)
            .or_default()
            .push(id);
        id
    }

    /// Binds a symbol within the specified scope.
    pub fn define_symbol(&mut self, scope_id: ScopeId, name: String, symbol: Symbol) {
        self.arena[scope_id].symbols.insert(name, symbol);
    }

    fn register_module(&mut self, m: &Module, mut parent_id: ScopeId, config: &AnalyzerConfig) {
        // Create nested scopes for the full module path
        for part in &m.name {
            if part == "root" {
                continue;
            }

            // Check if this part already exists as a child of parent_id
            let found = self.arena[parent_id]
                .children_by_name
                .get(part)
                .and_then(|ids| ids.first().copied());

            parent_id = if let Some(existing_id) = found {
                existing_id
            } else {
                let id = self.new_scope(parent_id, part.clone());
                self.arena[id].is_module = true;
                self.arena[id].language = m.language.clone();
                self.define_symbol(parent_id, part.clone(), Symbol::Module(id));
                id
            };
        }

        let scope_id = parent_id;

        // Update language if not set
        if self.arena[scope_id].language.is_none() {
            self.arena[scope_id].language = m.language.clone();
        }

        // Add imports
        for imp in &m.imports {
            if !self.arena[scope_id].imports.contains(imp) {
                self.arena[scope_id].imports.push(imp.clone());
            }
        }

        // Type Aliases
        for ta in &m.type_aliases {
            let name = ta.name.last().cloned().unwrap_or_default();
            self.define_symbol(scope_id, name, Symbol::TypeAlias(ta.target.clone()));
        }

        // Structured types
        for st in &m.structured_types {
            self.register_structured_type(
                st,
                scope_id,
                config,
                m.language.as_deref().unwrap_or(""),
            );
        }

        // Impl blocks
        for ib in &m.impl_blocks {
            self.pending_impl_blocks.push((
                ib.clone(),
                scope_id,
                m.language.as_deref().unwrap_or("").to_string(),
            ));
        }

        // Free functions
        for ff in &m.free_functions {
            let name = ff.name.last().cloned().unwrap_or_default();
            self.define_symbol(
                scope_id,
                name.clone(),
                Symbol::Value(ff.signature.return_type.clone()),
            );
            self.register_function(
                ff,
                scope_id,
                config,
                m.language.as_deref().unwrap_or(""),
                None,
            );
        }

        // Free variables
        for fv in &m.free_variables {
            self.define_symbol(scope_id, fv.name.clone(), Symbol::Value(fv.ty.clone()));
        }

        // Submodules
        for sub in &m.sub_modules {
            self.register_module(sub, scope_id, config);
        }
    }

    fn register_structured_type(
        &mut self,
        st: &StructuredType,
        parent_id: ScopeId,
        config: &AnalyzerConfig,
        lang: &str,
    ) {
        let name = st.name.last().cloned().unwrap_or_default();
        let class_scope = self.new_scope(parent_id, name.clone());
        if !name.is_empty() {
            self.type_scopes_by_name
                .entry(name.clone())
                .or_default()
                .push(class_scope);
        }

        self.arena[class_scope].super_types = st.super_types.clone();

        self.define_symbol(parent_id, name.clone(), Symbol::Type(class_scope));

        for tp in &st.type_parameters {
            self.arena[class_scope]
                .type_parameters
                .push(tp.name.clone());
            let phantom_scope = self.new_scope(class_scope, tp.name.clone());
            self.arena[phantom_scope].is_phantom = true;
            self.arena[phantom_scope].super_types = tp.bounds.clone();
            self.define_symbol(class_scope, tp.name.clone(), Symbol::Type(phantom_scope));
        }

        for field in &st.fields {
            self.define_symbol(
                class_scope,
                field.name.clone(),
                Symbol::Value(field.ty.clone()),
            );
        }

        let mut path = vec![];
        let mut curr = Some(class_scope);
        while let Some(id) = curr {
            if !self.arena[id].name.starts_with("block") && self.arena[id].name != "root" {
                path.push(self.arena[id].name.clone());
            }
            curr = self.arena[id].parent;
        }
        path.reverse();
        let type_ref = TypeRef::Resolved(path);

        for method in &st.methods {
            let m_name = method.name.last().cloned().unwrap_or_default();
            if !method.is_constructor && m_name != name && !m_name.starts_with('~') {
                self.define_symbol(
                    class_scope,
                    m_name,
                    Symbol::Value(method.signature.return_type.clone()),
                );
            }
            self.register_function(method, class_scope, config, lang, Some(type_ref.clone()));
        }

        for import in &st.imports {
            self.arena[class_scope].imports.push(import.clone());
        }

        for nested in &st.nested_types {
            self.register_structured_type(nested, class_scope, config, lang);
        }
    }

    fn register_impl_block(
        &mut self,
        ib: &ImplBlock,
        parent_id: ScopeId,
        config: &AnalyzerConfig,
        lang: &str,
    ) {
        // Find if the target class scope already exists in parent
        let target_name = match &ib.impl_for {
            TypeRef::Resolved(qn) | TypeRef::External(qn) | TypeRef::Unresolved(qn) => {
                qn.last().cloned().unwrap_or_default()
            }
            TypeRef::ResolutionQuery(q) => crate::resolver::executor::extract_base_name(q),
            TypeRef::Generic { base, .. } => match &**base {
                TypeRef::Resolved(qn) | TypeRef::External(qn) | TypeRef::Unresolved(qn) => {
                    qn.last().cloned().unwrap_or_default()
                }
                TypeRef::ResolutionQuery(q) => crate::resolver::executor::extract_base_name(q),
                _ => "".to_string(),
            },
            _ => "".to_string(),
        };

        if target_name.is_empty() {
            return;
        }

        // Lookup target scope
        let mut target_scope_id = None;
        if let Some(Symbol::Type(id)) = self.arena[parent_id].symbols.get(&target_name) {
            target_scope_id = Some(*id);
        }

        // If not found in parent, try to find the class globally (common for out-of-line C++ methods)
        if target_scope_id.is_none() {
            if let Some(ids) = self.type_scopes_by_name.get(&target_name) {
                target_scope_id = ids.first().copied();
            }
        }

        log::trace!(
            "register_impl_block target_name: {} resolved to: {:?}",
            target_name,
            target_scope_id
        );
        let class_scope = if let Some(id) = target_scope_id {
            id
        } else {
            let id = self.new_scope(parent_id, target_name.clone());
            self.define_symbol(parent_id, target_name.clone(), Symbol::Type(id));
            self.type_scopes_by_name
                .entry(target_name.clone())
                .or_default()
                .push(id);
            id
        };

        if let Some(kw) = &config.get_for(lang).self_type_keyword {
            self.define_symbol(
                class_scope,
                kw.clone(),
                Symbol::TypeAlias(ib.impl_for.clone()),
            );
        }

        for method in &ib.methods {
            let m_name = method.name.last().cloned().unwrap_or_default();
            if !method.is_constructor && m_name != target_name && !m_name.starts_with('~') {
                self.define_symbol(
                    class_scope,
                    m_name,
                    Symbol::Value(method.signature.return_type.clone()),
                );
            }
            self.register_function(method, class_scope, config, lang, Some(ib.impl_for.clone()));
        }

        for nested in &ib.nested_types {
            self.register_structured_type(nested, class_scope, config, lang);
        }

        for ta in &ib.type_aliases {
            let ta_name = ta.name.last().cloned().unwrap_or_default();
            self.define_symbol(
                class_scope,
                ta_name.clone(),
                Symbol::TypeAlias(ta.target.clone()),
            );

            // Magic support for Rust's `Deref` trait which implies inheritance
            if ta_name == "Target"
                && let Some(trait_ref) = &ib.implements_trait
            {
                let is_deref = match trait_ref {
                    TypeRef::Resolved(qn) | TypeRef::Unresolved(qn) | TypeRef::External(qn) => {
                        qn.last().map(|s| s.as_str()) == Some("Deref")
                    }
                    TypeRef::ResolutionQuery(q) => {
                        crate::resolver::executor::extract_base_name(q) == "Deref"
                    }
                    _ => false,
                };
                if is_deref {
                    self.arena[class_scope].super_types.push(ta.target.clone());
                }
            }
        }
    }

    fn register_function(
        &mut self,
        func: &Function,
        parent_id: ScopeId,
        config: &AnalyzerConfig,
        lang: &str,
        parent_class_type: Option<TypeRef>,
    ) {
        let name = func.name.last().cloned().unwrap_or_default();
        let func_scope = self.new_scope(parent_id, name);
        let lang_config = config.get_for(lang);

        for tp in &func.type_parameters {
            self.arena[func_scope].type_parameters.push(tp.name.clone());
            let phantom_scope = self.new_scope(func_scope, tp.name.clone());
            self.arena[phantom_scope].is_phantom = true;
            self.arena[phantom_scope].super_types = tp.bounds.clone();
            self.define_symbol(func_scope, tp.name.clone(), Symbol::Type(phantom_scope));
        }

        let mut params = func.signature.parameters.iter();

        if lang_config.implicit_first_param_as_self
            && let Some(class_type) = &parent_class_type
            && let Some(first_param) = params.next()
        {
            let p_name = first_param
                .name
                .clone()
                .unwrap_or_else(|| "self".to_string());
            self.define_symbol(func_scope, p_name, Symbol::Value(class_type.clone()));
        }

        for param in params {
            if let Some(p_name) = &param.name {
                self.define_symbol(func_scope, p_name.clone(), Symbol::Value(param.ty.clone()));
            }
        }

        if let Some(ref class_type) = parent_class_type
            && let Some(ref kw) = lang_config.self_keyword
        {
            self.define_symbol(func_scope, kw.clone(), Symbol::Value(class_type.clone()));
        }

        if let Some(block) = &func.body {
            self.register_block(block, func_scope, 0);
        }
    }

    fn register_block(&mut self, block: &Block, parent_id: ScopeId, index: usize) {
        let block_scope = self.new_scope(parent_id, format!("block_{}", index));

        for decl in &block.declarations {
            self.define_symbol(
                block_scope,
                decl.name.clone(),
                Symbol::Value(decl.ty.clone()),
            );
        }

        for (i, sub) in block.sub_blocks.iter().enumerate() {
            self.register_block(sub, block_scope, i);
        }
    }
}
