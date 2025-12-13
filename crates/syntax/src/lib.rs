mod highlight;
mod language;
mod parser;

pub use highlight::{HighlightSpan, LineHighlights, SyntaxHighlighter};
pub use language::{LanguageConfig, LanguageRegistry, TokenType};
pub use parser::{create_input_edit, IncrementalParser};
