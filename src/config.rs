use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleConfig {
    pub file_based: bool,
    pub directory_based: bool,
    pub package_decl_based: bool,
    pub namespace_based: bool,
    pub inline_mod_based: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub modules: ModuleConfig,
    pub transitive_imports: bool,
    pub support_impl_blocks: bool,
    pub forward_declarations: bool,
    pub self_keyword: Option<String>,
    pub implicit_first_param_as_self: bool,
    pub extract_dynamic_fields: bool,
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

        // Python: Directory Based and File Based
        languages.insert(
            "python".to_string(),
            LanguageConfig {
                modules: ModuleConfig {
                    file_based: true,
                    directory_based: true,
                    package_decl_based: false,
                    namespace_based: false,
                    inline_mod_based: false,
                },
                transitive_imports: true,
                support_impl_blocks: false,
                forward_declarations: false,
                self_keyword: None,
                implicit_first_param_as_self: true,
                extract_dynamic_fields: true,
            },
        );

        // Rust: File, Directory and Inline Mod Based
        languages.insert(
            "rust".to_string(),
            LanguageConfig {
                modules: ModuleConfig {
                    file_based: true,
                    directory_based: true,
                    package_decl_based: false,
                    namespace_based: false,
                    inline_mod_based: true,
                },
                transitive_imports: false,
                support_impl_blocks: true,
                forward_declarations: false,
                self_keyword: Some("self".to_string()),
                implicit_first_param_as_self: false,
                extract_dynamic_fields: false,
            },
        );

        // Java: Package Decl Based
        languages.insert(
            "java".to_string(),
            LanguageConfig {
                modules: ModuleConfig {
                    file_based: false,
                    directory_based: false,
                    package_decl_based: true,
                    namespace_based: false,
                    inline_mod_based: false,
                },
                transitive_imports: false,
                support_impl_blocks: false,
                forward_declarations: false,
                self_keyword: Some("this".to_string()),
                implicit_first_param_as_self: false,
                extract_dynamic_fields: false,
            },
        );

        // C++: Namespace Based
        languages.insert(
            "cpp".to_string(),
            LanguageConfig {
                modules: ModuleConfig {
                    file_based: false,
                    directory_based: false,
                    package_decl_based: false,
                    namespace_based: true,
                    inline_mod_based: false,
                },
                transitive_imports: false,
                support_impl_blocks: true,
                forward_declarations: true,
                self_keyword: Some("this".to_string()),
                implicit_first_param_as_self: false,
                extract_dynamic_fields: false,
            },
        );

        // C: None
        languages.insert(
            "c".to_string(),
            LanguageConfig {
                modules: ModuleConfig {
                    file_based: false,
                    directory_based: false,
                    package_decl_based: false,
                    namespace_based: false,
                    inline_mod_based: false,
                },
                transitive_imports: false,
                support_impl_blocks: false,
                forward_declarations: true,
                self_keyword: None,
                implicit_first_param_as_self: false,
                extract_dynamic_fields: false,
            },
        );

        Self {
            default_config: LanguageConfig {
                modules: ModuleConfig {
                    file_based: false,
                    directory_based: false,
                    package_decl_based: false,
                    namespace_based: false,
                    inline_mod_based: false,
                },
                transitive_imports: false,
                support_impl_blocks: false,
                forward_declarations: false,
                self_keyword: Some("this".to_string()),
                implicit_first_param_as_self: false,
                extract_dynamic_fields: false,
            },
            languages,
        }
    }

    /// Retrieves the configuration specific to a language, or the default one.
    pub fn get_for(&self, lang: &str) -> &LanguageConfig {
        self.languages.get(lang).unwrap_or(&self.default_config)
    }
}
