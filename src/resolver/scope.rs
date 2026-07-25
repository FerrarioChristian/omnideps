use crate::model::{Module, QualifiedName, StructuredType, TypeRef, Function, ImplBlock, Block};
use std::collections::HashMap;
use crate::config::AnalyzerConfig;

/// Un identificatore univoco per uno Scope all'interno dell'Arena.
pub type ScopeId = usize;

/// Rappresenta il tipo di un simbolo all'interno di uno Scope.
#[derive(Debug, Clone)]
pub enum Symbol {
    /// Un sottomodulo. Punta al suo Scope.
    Module(ScopeId),
    /// Un tipo strutturato (Classe, Struct). Punta al suo Scope.
    Type(ScopeId),
    /// Un Type Alias. Punta al tipo bersaglio.
    TypeAlias(TypeRef),
    /// Un valore concreto: Variabile locale, parametro, campo o funzione.
    /// Il TypeRef indica il tipo del valore (o il tipo di ritorno per le funzioni).
    Value(TypeRef),
}

/// Un singolo Environment Lexicale (Scope).
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
}

/// L'albero gerarchico degli Scope Lexicali, implementato tramite Arena Pattern.
#[derive(Debug, Clone)]
pub struct ScopeTree {
    pub arena: Vec<Scope>,
    pub root: ScopeId,
}

impl ScopeTree {
    /// Costruisce l'albero degli scope a partire dai moduli estratti.
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
            }],
            root: 0,
        };

        for m in modules {
            tree.register_module(m, 0, config);
        }

        tree
    }

    /// Crea un nuovo scope figlio.
    pub fn new_scope(&mut self, parent: ScopeId, name: String) -> ScopeId {
        let id = self.arena.len();
        self.arena.push(Scope {
            id,
            parent: Some(parent),
            name,
            symbols: HashMap::new(),
            imports: vec![],
            super_types: vec![],
            is_module: false,
            language: None,
        });
        id
    }

    /// Inserisce un simbolo all'interno di uno scope.
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
            let mut found = None;
            for child in &self.arena {
                if child.parent == Some(parent_id) && child.name == *part {
                    found = Some(child.id);
                    break;
                }
            }
            
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

        // Aggiungi import
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

        // Tipi strutturati
        for st in &m.structured_types {
            self.register_structured_type(st, scope_id, config, m.language.as_deref().unwrap_or(""));
        }

        // Impl blocks
        for ib in &m.impl_blocks {
            self.register_impl_block(ib, scope_id, config, m.language.as_deref().unwrap_or(""));
        }

        // Funzioni libere
        for ff in &m.free_functions {
            let name = ff.name.last().cloned().unwrap_or_default();
            self.define_symbol(scope_id, name.clone(), Symbol::Value(ff.signature.return_type.clone()));
            self.register_function(ff, scope_id, config, m.language.as_deref().unwrap_or(""), None);
        }

        // Variabili libere
        for fv in &m.free_variables {
            self.define_symbol(scope_id, fv.name.clone(), Symbol::Value(fv.ty.clone()));
        }

        // Sottomoduli
        for sub in &m.sub_modules {
            self.register_module(sub, scope_id, config);
        }
    }

    fn register_structured_type(&mut self, st: &StructuredType, parent_id: ScopeId, config: &AnalyzerConfig, lang: &str) {
        let name = st.name.last().cloned().unwrap_or_default();
        let class_scope = self.new_scope(parent_id, name.clone());
        
        self.arena[class_scope].super_types = st.super_types.clone();

        self.define_symbol(parent_id, name.clone(), Symbol::Type(class_scope));

        for field in &st.fields {
            self.define_symbol(class_scope, field.name.clone(), Symbol::Value(field.ty.clone()));
        }

        let type_ref = TypeRef::Resolved(st.name.clone());
        for method in &st.methods {
            let m_name = method.name.last().cloned().unwrap_or_default();
            self.define_symbol(class_scope, m_name, Symbol::Value(method.signature.return_type.clone()));
            self.register_function(method, class_scope, config, lang, Some(type_ref.clone()));
        }

        for nested in &st.nested_types {
            self.register_structured_type(nested, class_scope, config, lang);
        }
    }

    fn register_impl_block(&mut self, ib: &ImplBlock, parent_id: ScopeId, config: &AnalyzerConfig, lang: &str) {
        // Find if the target class scope already exists in parent
        let target_name = match &ib.impl_for {
            TypeRef::Resolved(qn) | TypeRef::External(qn) => qn.last().cloned().unwrap_or_default(),
            TypeRef::ResolutionQuery(q) => crate::resolver::executor::extract_base_name(q),
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

        let class_scope = if let Some(id) = target_scope_id {
            id
        } else {
            let id = self.new_scope(parent_id, target_name.clone());
            self.define_symbol(parent_id, target_name.clone(), Symbol::Type(id));
            id
        };

        for method in &ib.methods {
            let m_name = method.name.last().cloned().unwrap_or_default();
            self.define_symbol(class_scope, m_name, Symbol::Value(method.signature.return_type.clone()));
            self.register_function(method, class_scope, config, lang, Some(ib.impl_for.clone()));
        }

        for nested in &ib.nested_types {
            self.register_structured_type(nested, class_scope, config, lang);
        }
    }

    fn register_function(&mut self, func: &Function, parent_id: ScopeId, config: &AnalyzerConfig, lang: &str, parent_class_type: Option<TypeRef>) {
        let name = func.name.last().cloned().unwrap_or_default();
        let func_scope = self.new_scope(parent_id, name);
        let lang_config = config.get_for(lang);

        let mut params = func.signature.parameters.iter();
        
        if lang_config.implicit_first_param_as_self {
            if let Some(class_type) = &parent_class_type {
                if let Some(first_param) = params.next() {
                    let p_name = first_param.name.clone().unwrap_or_else(|| "self".to_string());
                    self.define_symbol(func_scope, p_name, Symbol::Value(class_type.clone()));
                }
            }
        }

        for param in params {
            if let Some(p_name) = &param.name {
                self.define_symbol(func_scope, p_name.clone(), Symbol::Value(param.ty.clone()));
            }
        }

        if let Some(ref class_type) = parent_class_type {
            if let Some(ref kw) = lang_config.self_keyword {
                self.define_symbol(func_scope, kw.clone(), Symbol::Value(class_type.clone()));
            }
        }

        if let Some(block) = &func.body {
            self.register_block(block, func_scope, 0);
        }
    }

    fn register_block(&mut self, block: &Block, parent_id: ScopeId, index: usize) {
        let block_scope = self.new_scope(parent_id, format!("block_{}", index));

        for decl in &block.declarations {
            self.define_symbol(block_scope, decl.name.clone(), Symbol::Value(decl.ty.clone()));
        }

        for (i, sub) in block.sub_blocks.iter().enumerate() {
            self.register_block(sub, block_scope, i);
        }
    }
}
