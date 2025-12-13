use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    Keyword,
    Function,
    Type,
    String,
    Comment,
    Number,
    Operator,
    Variable,
    Punctuation,
    Property,
    Constant,
    None,
}

#[derive(Clone)]
pub struct LanguageConfig {
    pub name: &'static str,
    pub language: tree_sitter::Language,
    pub highlight_query: &'static str,
    pub extensions: &'static [&'static str],
}

#[derive(Clone)]
pub struct LanguageRegistry {
    languages: HashMap<&'static str, LanguageConfig>,
    extension_map: HashMap<&'static str, &'static str>,
}

impl std::fmt::Debug for LanguageRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanguageRegistry")
            .field("languages", &self.languages.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl LanguageRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            languages: HashMap::new(),
            extension_map: HashMap::new(),
        };
        registry.register_builtin_languages();
        registry
    }

    fn register_builtin_languages(&mut self) {
        self.register(LanguageConfig {
            name: "rust",
            language: tree_sitter_rust::language(),
            highlight_query: include_str!("queries/rust.scm"),
            extensions: &["rs"],
        });
        self.register(LanguageConfig {
            name: "javascript",
            language: tree_sitter_javascript::language(),
            highlight_query: include_str!("queries/javascript.scm"),
            extensions: &["js", "jsx", "mjs"],
        });
    }

    pub fn register(&mut self, config: LanguageConfig) {
        for ext in config.extensions {
            self.extension_map.insert(ext, config.name);
        }
        self.languages.insert(config.name, config);
    }

    pub fn detect_language(&self, filename: &str) -> Option<&LanguageConfig> {
        let extension = filename.rsplit('.').next()?;
        let lang_name = self.extension_map.get(extension)?;
        self.languages.get(lang_name)
    }

    pub fn get_language(&self, name: &str) -> Option<&LanguageConfig> {
        self.languages.get(name)
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}
