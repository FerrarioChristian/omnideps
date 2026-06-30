//! A data-driven registry for language-specific primitive types.
//!
//! This module decouples primitive type recognition from the AST extraction logic, allowing
//! the resolver to intercept and mark tokens like `String` or `int` as `PrimitiveType` 
//! instead of logging them as `Failed` resolutions.

use std::collections::{HashMap, HashSet};

/// A registry that loads and stores the language-specific primitive types.
pub struct PrimitiveRegistry {
    lang_primitives: HashSet<String>,
}

impl PrimitiveRegistry {
    /// Loads the primitives from the external JSON file and filters them for the target language.
    pub fn load(lang_name: &str) -> anyhow::Result<Self> {
        let primitives_json = include_str!("../../primitives.json");
        let registry: HashMap<String, Vec<String>> = serde_json::from_str(primitives_json)?;
        
        let lang_primitives = registry
            .get(lang_name)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect();
            
        Ok(Self { lang_primitives })
    }

    /// Creates an empty registry (used as fallback).
    pub fn empty() -> Self {
        Self {
            lang_primitives: HashSet::new(),
        }
    }

    /// Merges another registry into this one (useful for multi-language workspaces).
    pub fn merge(&mut self, other: PrimitiveRegistry) {
        self.lang_primitives.extend(other.lang_primitives);
    }

    /// Checks if a given identifier is a primitive type in the current language context.
    pub fn is_primitive(&self, name: &str) -> bool {
        self.lang_primitives.contains(name)
    }
}
