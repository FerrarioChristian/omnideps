use crate::model::QualifiedName;
use super::ResolutionResult;
use std::collections::HashMap;

/// A Cache structured to mimic the lexical tree hierarchy.
/// Instead of complex node-based trees, we use the absolute path of the scope (joined by "::")
/// as a unique key for O(1) memoization isolated per scope.
#[derive(Debug, Clone)]
pub struct ResolutionCache {
    /// Maps Scope Path -> (Name Searched -> Resolution Result)
    store: HashMap<String, HashMap<QualifiedName, ResolutionResult>>,
}

impl ResolutionCache {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    /// Generates a unique string identifier for the current scope hierarchy.
    fn scope_key(prefix: &[String]) -> String {
        if prefix.is_empty() {
            "GLOBAL_ROOT".to_string()
        } else {
            prefix.join("::")
        }
    }

    /// Retrieves a memoized resolution result for the given name in the current scope.
    pub fn get(&self, current_prefix: &[String], name: &QualifiedName) -> Option<ResolutionResult> {
        let key = Self::scope_key(current_prefix);
        self.store.get(&key).and_then(|scope_cache| scope_cache.get(name).cloned())
    }

    /// Stores a resolution result for the given name in the current scope.
    pub fn insert(&mut self, current_prefix: &[String], name: QualifiedName, res: ResolutionResult) {
        let key = Self::scope_key(current_prefix);
        self.store
            .entry(key)
            .or_insert_with(HashMap::new)
            .insert(name, res);
    }
}
