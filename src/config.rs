use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModuleStrategy {
    DirectoryBased,
    PackageBased,
    SingleRoot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub module_strategy: ModuleStrategy,
    pub transitive_imports: bool,
    pub cyclic_mro_check: bool,
    pub support_impl_blocks: bool,
    pub self_keyword: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerConfig {
    pub default_config: LanguageConfig,
    pub languages: HashMap<String, LanguageConfig>,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self::default_strategies()
    }
}

impl AnalyzerConfig {
    /// Creates the default architectural strategies based on language semantics.
    pub fn default_strategies() -> Self {
        let mut languages = HashMap::new();
        
        // Python: Directory Based, transitive imports allowed
        languages.insert("python".to_string(), LanguageConfig {
            module_strategy: ModuleStrategy::DirectoryBased,
            transitive_imports: true,
            cyclic_mro_check: true,
            support_impl_blocks: false,
            self_keyword: "self".to_string(),
        });

        // Rust: Directory Based, no transitive imports
        languages.insert("rust".to_string(), LanguageConfig {
            module_strategy: ModuleStrategy::DirectoryBased,
            transitive_imports: false,
            cyclic_mro_check: true,
            support_impl_blocks: true,
            self_keyword: "self".to_string(),
        });

        // Java: Package Based
        languages.insert("java".to_string(), LanguageConfig {
            module_strategy: ModuleStrategy::PackageBased,
            transitive_imports: false,
            cyclic_mro_check: true,
            support_impl_blocks: false,
            self_keyword: "this".to_string(),
        });

        // C++ / C: Package/Namespace Based
        languages.insert("cpp".to_string(), LanguageConfig {
            module_strategy: ModuleStrategy::PackageBased,
            transitive_imports: false,
            cyclic_mro_check: true,
            support_impl_blocks: true, // C++ can implement methods outside class declaration
            self_keyword: "this".to_string(),
        });
        languages.insert("c".to_string(), LanguageConfig {
            module_strategy: ModuleStrategy::PackageBased,
            transitive_imports: false,
            cyclic_mro_check: true,
            support_impl_blocks: false,
            self_keyword: "this".to_string(),
        });

        Self {
            default_config: LanguageConfig {
                module_strategy: ModuleStrategy::SingleRoot,
                transitive_imports: false,
                cyclic_mro_check: true,
                support_impl_blocks: false,
                self_keyword: "this".to_string(),
            },
            languages,
        }
    }

    /// Retrieves the configuration specific to a language, or the default one.
    pub fn get_for(&self, lang: &str) -> &LanguageConfig {
        self.languages.get(lang).unwrap_or(&self.default_config)
    }
}
