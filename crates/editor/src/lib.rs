mod buffer;
mod document;
mod engine;
mod history;
mod keymap;
mod layout;
mod search;
mod selection;
mod text_shaping;

pub use buffer::{Buffer, EditImpact, ReplaceRange};
pub use document::{Document, DocumentSnapshot};
pub use engine::EditorEngine;
pub use history::{Edit, History, Transaction, TransactionKind};
pub use keymap::{KeyAction, KeyChord, KeyCode, KeyModifiers, Keymap, Movement};
pub use layout::{
    EditorViewModel, FontMetrics, LayoutConfig, SelectionSpan, VisualLine, Viewport,
    WhitespaceConfig,
};
pub use search::{SearchDirection, SearchMatch, SearchQuery};
pub use selection::{Cursor, LineCol, Selection, SelectionSet};
pub use text_shaping::{ShapedGlyph, ShapedLine, TextShaper};

pub use syntax::{HighlightSpan, LanguageRegistry, SyntaxHighlighter, TokenType};
