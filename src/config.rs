use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    pub implicit_file_modules: bool,
    pub file_level_declarations: bool,
    pub inline_module_blocks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub modules: ModuleConfig,
    pub transitive_imports: bool,
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
            modules: ModuleConfig {
                implicit_file_modules: true,
                file_level_declarations: false,
                inline_module_blocks: false,
            },
            transitive_imports: true,
            support_impl_blocks: false,
            self_keyword: "self".to_string(),
        });

        // Rust: Directory Based, no transitive imports
        languages.insert("rust".to_string(), LanguageConfig {
            modules: ModuleConfig {
                implicit_file_modules: true,
                file_level_declarations: false,
                inline_module_blocks: true,
            },
            transitive_imports: false,
            support_impl_blocks: true,
            self_keyword: "self".to_string(),
        });

        // Java: Package Based
        languages.insert("java".to_string(), LanguageConfig {
            modules: ModuleConfig {
                implicit_file_modules: false,
                file_level_declarations: true,
                inline_module_blocks: false,
            },
            transitive_imports: false,
            support_impl_blocks: false,
            self_keyword: "this".to_string(),
        });

        // C++ / C: Package/Namespace Based
        languages.insert("cpp".to_string(), LanguageConfig {
            modules: ModuleConfig {
                implicit_file_modules: false,
                file_level_declarations: false,
                inline_module_blocks: true,
            },
            transitive_imports: false,
            support_impl_blocks: true, // C++ can implement methods outside class declaration
            self_keyword: "this".to_string(),
        });
        languages.insert("c".to_string(), LanguageConfig {
            modules: ModuleConfig {
                implicit_file_modules: false,
                file_level_declarations: false,
                inline_module_blocks: false,
            },
            transitive_imports: false,
            support_impl_blocks: false,
            self_keyword: "this".to_string(),
        });

        Self {
            default_config: LanguageConfig {
                modules: ModuleConfig {
                    implicit_file_modules: false,
                    file_level_declarations: false,
                    inline_module_blocks: false,
                },
                transitive_imports: false,
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
