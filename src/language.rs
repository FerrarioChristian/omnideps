//! Defines the languages supported by the analyzer and their integration with tree-sitter.

use std::path::Path;
use tree_sitter::Language;

/// An enumeration of all the programming languages the analyzer can process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedLanguage {
    Rust,
    Java,
    Python,
    C,
    Cpp,
}

impl SupportedLanguage {
    /// Attempts to identify the language from a file's extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "java" => Some(Self::Java),
            "py" => Some(Self::Python),
            "c" | "h" => Some(Self::C),
            "cpp" | "cxx" | "cc" | "hxx" | "hpp" => Some(Self::Cpp),
            _ => None,
        }
    }

    /// Attempts to identify the language from a file path.
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        Self::from_extension(ext)
    }

    /// Returns the corresponding Tree-sitter language parser.
    pub fn to_tree_sitter_lang(&self) -> Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Java => tree_sitter_java::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::C => tree_sitter_c::LANGUAGE.into(),
            Self::Cpp => tree_sitter_cpp::LANGUAGE.into(),
        }
    }

    /// Returns the string identifier used for configuration and primitive loading.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Java => "java",
            Self::Python => "python",
            Self::C => "c",
            Self::Cpp => "cpp",
        }
    }
}
