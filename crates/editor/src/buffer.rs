use crate::document::{Document, DocumentSnapshot};
use crate::history::{Edit, History, Transaction, TransactionKind};
use crate::selection::{Selection, SelectionSet};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EditImpact {
    pub start_line: usize,
    pub end_line_inclusive: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceRange {
    pub start_char: usize,
    pub end_char: usize,
    pub inserted: String,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub doc: Document,
    pub selections: SelectionSet,
    pub history: History,
    pub last_edit_impact: Option<EditImpact>,
}

impl Buffer {
    pub fn new(text: &str) -> Self {
        Self {
            doc: Document::new(text),
            selections: SelectionSet::default(),
            history: History::default(),
            last_edit_impact: None,
        }
    }

    pub fn snapshot(&self) -> DocumentSnapshot {
        self.doc.snapshot()
    }

    pub fn restore(&mut self, snapshot: DocumentSnapshot) {
        self.doc.restore(snapshot);
        self.history = History::default();
        self.selections.set_single_caret(0);
        self.last_edit_impact = None;
    }

    pub fn apply_text_to_selections(&mut self, inserted: &str) {
        let selections = self.selections.all_including_primary();
        let mut start_line = usize::MAX;
        let mut end_line = 0usize;
        let mut edits: Vec<Edit> = selections
            .iter()
            .map(|s| {
                let (start, end) = s.range();
                start_line = start_line.min(self.doc.char_to_line(start));
                end_line = end_line.max(self.doc.char_to_line(end));
                Edit {
                    start_char: start,
                    deleted: self.doc.slice_to_string(start, end),
                    inserted: inserted.to_string(),
                }
            })
            .collect();
        if edits.iter().all(|e| e.deleted.is_empty() && e.inserted.is_empty()) {
            return;
        }
        edits.sort_by(|a, b| b.start_char.cmp(&a.start_char));
        for e in edits.iter() {
            let delete_end = e.start_char + e.deleted_len_chars();
            self.doc.replace_range(e.start_char, delete_end, &e.inserted);
        }
        let mut new_set = SelectionSet::default();
        let mut collapsed: Vec<Selection> = selections
            .iter()
            .map(|s| {
                let start = s.range().0;
                let caret = start + inserted.chars().count();
                Selection {
                    anchor: caret,
                    head: caret,
                }
            })
            .collect();
        if let Some(p) = collapsed.first().copied() {
            new_set.primary = p;
            if collapsed.len() > 1 {
                new_set.secondary = collapsed.drain(1..).collect();
            }
        }
        self.selections = new_set;
        let kind = if inserted.is_empty() {
            TransactionKind::Delete
        } else if selections.iter().all(|s| s.is_caret()) {
            TransactionKind::Insert
        } else {
            TransactionKind::Replace
        };
        let tx = Transaction { kind, edits };
        let allow_coalesce = kind == TransactionKind::Insert
            && inserted.chars().count() == 1
            && self.selections.is_single_caret();
        self.history.push(tx, allow_coalesce);
        if start_line == usize::MAX {
            self.last_edit_impact = None;
        } else {
            let inserted_newlines = inserted.chars().filter(|c| *c == '\n').count();
            let extra_lines = inserted_newlines + 1;
            self.last_edit_impact = Some(EditImpact {
                start_line,
                end_line_inclusive: end_line.saturating_add(extra_lines),
            });
        }
    }

    pub fn apply_replace_ranges(
        &mut self,
        ranges: Vec<ReplaceRange>,
        kind: TransactionKind,
        new_selections: SelectionSet,
    ) {
        if ranges.is_empty() {
            return;
        }
        let mut start_line = usize::MAX;
        let mut end_line = 0usize;
        let mut edits: Vec<Edit> = ranges
            .into_iter()
            .map(|r| {
                start_line = start_line.min(self.doc.char_to_line(r.start_char));
                end_line = end_line.max(self.doc.char_to_line(r.end_char));
                Edit {
                    start_char: r.start_char,
                    deleted: self.doc.slice_to_string(r.start_char, r.end_char),
                    inserted: r.inserted,
                }
            })
            .collect();
        edits.sort_by(|a, b| b.start_char.cmp(&a.start_char));
        for e in edits.iter() {
            let delete_end = e.start_char + e.deleted_len_chars();
            self.doc.replace_range(e.start_char, delete_end, &e.inserted);
        }
        self.selections = new_selections;
        self.history.push(Transaction { kind, edits }, false);
        if start_line == usize::MAX {
            self.last_edit_impact = None;
        } else {
            self.last_edit_impact = Some(EditImpact {
                start_line,
                end_line_inclusive: end_line.saturating_add(1),
            });
        }
    }

    pub fn undo(&mut self) -> bool {
        let Some(tx) = self.history.undo.pop() else {
            return false;
        };
        let mut inverse = tx.clone();
        inverse.edits.sort_by(|a, b| b.start_char.cmp(&a.start_char));
        for e in inverse.edits.iter() {
            let end = e.start_char + e.inserted_len_chars();
            self.doc.replace_range(e.start_char, end, &e.deleted);
        }
        self.history.redo.push(tx);
        self.last_edit_impact = None;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(tx) = self.history.redo.pop() else {
            return false;
        };
        let mut forward = tx.clone();
        forward.edits.sort_by(|a, b| b.start_char.cmp(&a.start_char));
        for e in forward.edits.iter() {
            let end = e.start_char + e.deleted_len_chars();
            self.doc.replace_range(e.start_char, end, &e.inserted);
        }
        self.history.undo.push(tx);
        self.last_edit_impact = None;
        true
    }
}
