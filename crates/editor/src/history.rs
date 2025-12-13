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
