use ropey::Rope;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Cursor {
    pub char_idx: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Selection {
    pub anchor: usize,
    pub head: usize,
}

impl Selection {
    pub fn is_caret(&self) -> bool {
        self.anchor == self.head
    }

    pub fn range(&self) -> (usize, usize) {
        if self.anchor <= self.head {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSet {
    pub primary: Selection,
    pub secondary: Vec<Selection>,
}

impl Default for SelectionSet {
    fn default() -> Self {
        Self {
            primary: Selection { anchor: 0, head: 0 },
            secondary: Vec::new(),
        }
    }
}

impl SelectionSet {
    pub fn is_single_caret(&self) -> bool {
        self.secondary.is_empty() && self.primary.is_caret()
    }

    pub fn all_including_primary(&self) -> Vec<Selection> {
        let mut out = Vec::with_capacity(1 + self.secondary.len());
        out.push(self.primary);
        out.extend(self.secondary.iter().copied());
        out
    }

    pub fn set_single_caret(&mut self, char_idx: usize) {
        self.primary = Selection {
            anchor: char_idx,
            head: char_idx,
        };
        self.secondary.clear();
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    rope: Rope,
    version: u64,
}

#[derive(Debug, Clone)]
pub struct DocumentSnapshot {
    rope: Rope,
    version: u64,
}

impl Document {
    pub fn new(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            version: 0,
        }
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn to_string(&self) -> String {
        self.rope.to_string()
    }

    pub fn snapshot(&self) -> DocumentSnapshot {
        DocumentSnapshot {
            rope: self.rope.clone(),
            version: self.version,
        }
    }

    pub fn restore(&mut self, snapshot: DocumentSnapshot) {
        self.rope = snapshot.rope;
        self.version = snapshot.version;
    }

    pub fn slice_to_string(&self, start_char: usize, end_char: usize) -> String {
        let start = start_char.min(self.rope.len_chars());
        let end = end_char.min(self.rope.len_chars());
        if start >= end {
            return String::new();
        }
        self.rope.slice(start..end).to_string()
    }

    pub fn line_to_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx)
    }

    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.char_to_line(char_idx)
    }

    pub fn char_to_line_col(&self, char_idx: usize) -> LineCol {
        let line = self.rope.char_to_line(char_idx);
        let line_start = self.rope.line_to_char(line);
        LineCol {
            line,
            col: char_idx.saturating_sub(line_start),
        }
    }

    pub fn line_col_to_char(&self, line: usize, col: usize) -> usize {
        let line_start = self.rope.line_to_char(line);
        let line_end = self.rope.line_to_char((line + 1).min(self.rope.len_lines()));
        (line_start + col).min(line_end)
    }

    pub fn insert(&mut self, char_idx: usize, text: &str) {
        self.rope.insert(char_idx, text);
        self.version = self.version.wrapping_add(1);
    }

    pub fn delete_range(&mut self, start_char: usize, end_char: usize) {
        if start_char >= end_char {
            return;
        }
        self.rope.remove(start_char..end_char);
        self.version = self.version.wrapping_add(1);
    }

    pub fn replace_range(&mut self, start_char: usize, end_char: usize, inserted: &str) {
        let start = start_char.min(self.rope.len_chars());
        let end = end_char.min(self.rope.len_chars());

        if start < end {
            self.rope.remove(start..end);
        }
        if !inserted.is_empty() {
            self.rope.insert(start, inserted);
        }
        self.version = self.version.wrapping_add(1);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub start_char: usize,
    pub deleted: String,
    pub inserted: String,
}

impl Edit {
    pub fn inserted_len_chars(&self) -> usize {
        self.inserted.chars().count()
    }

    pub fn deleted_len_chars(&self) -> usize {
        self.deleted.chars().count()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TransactionKind {
    Insert,
    Delete,
    Replace,
    Other,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub kind: TransactionKind,
    pub edits: Vec<Edit>,
}

#[derive(Debug, Default, Clone)]
pub struct History {
    pub undo: Vec<Transaction>,
    pub redo: Vec<Transaction>,
}

impl History {
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn clear_redo(&mut self) {
        self.redo.clear();
    }

    pub fn push(&mut self, tx: Transaction, allow_coalesce_insert: bool) {
        if allow_coalesce_insert {
            if tx.kind == TransactionKind::Insert {
                if let Some(prev) = self.undo.last_mut() {
                    if prev.kind == TransactionKind::Insert {
                        if prev.edits.len() == 1 && tx.edits.len() == 1 {
                            let prev_edit = &mut prev.edits[0];
                            let new_edit = &tx.edits[0];
                            if prev_edit.deleted.is_empty()
                                && new_edit.deleted.is_empty()
                                && prev_edit.start_char + prev_edit.inserted_len_chars()
                                    == new_edit.start_char
                            {
                                prev_edit.inserted.push_str(&new_edit.inserted);
                                self.redo.clear();
                                return;
                            }
                        }
                    }
                }
            }
        }

        self.undo.push(tx);
        self.redo.clear();
    }
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub doc: Document,
    pub selections: SelectionSet,
    pub history: History,
}

impl Buffer {
    pub fn new(text: &str) -> Self {
        Self {
            doc: Document::new(text),
            selections: SelectionSet::default(),
            history: History::default(),
        }
    }

    pub fn snapshot(&self) -> DocumentSnapshot {
        self.doc.snapshot()
    }

    pub fn restore(&mut self, snapshot: DocumentSnapshot) {
        self.doc.restore(snapshot);
        self.history = History::default();
        self.selections.set_single_caret(0);
    }

    pub fn apply_text_to_selections(&mut self, inserted: &str) {
        let selections = self.selections.all_including_primary();
        let mut edits: Vec<Edit> = selections
            .iter()
            .map(|s| {
                let (start, end) = s.range();
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
        let allow_coalesce = kind == TransactionKind::Insert && inserted.chars().count() == 1 && self.selections.is_single_caret();
        self.history.push(tx, allow_coalesce);
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
        true
    }
}
