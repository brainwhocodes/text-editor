mod buffer;
mod document;
mod engine;
mod history;
mod keymap;
mod layout;
mod search;
mod selection;

pub use buffer::{Buffer, EditImpact, ReplaceRange};
pub use document::{Document, DocumentSnapshot};
pub use engine::EditorEngine;
pub use history::{Edit, History, Transaction, TransactionKind};
pub use keymap::{KeyAction, KeyChord, KeyCode, KeyModifiers, Keymap, Movement};
pub use layout::{EditorViewModel, FontMetrics, LayoutConfig, SelectionSpan, VisualLine, Viewport};
pub use search::{SearchDirection, SearchMatch, SearchQuery};
pub use selection::{Cursor, LineCol, Selection, SelectionSet};
