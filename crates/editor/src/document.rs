use ropey::Rope;
use crate::selection::LineCol;

#[derive(Debug, Clone)]
pub struct Document {
    rope: Rope,
    version: u64,
}

#[derive(Debug, Clone)]
pub struct DocumentSnapshot {
    pub(crate) rope: Rope,
    pub(crate) version: u64,
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

    pub fn line_text(&self, line_idx: usize) -> String {
        if line_idx >= self.rope.len_lines() {
            return String::new();
        }
        let mut s = self.rope.line(line_idx).to_string();
        if s.ends_with('\n') {
            s.pop();
        }
        s
    }

    pub fn line_start_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx.min(self.rope.len_lines()))
    }

    pub fn line_end_char(&self, line_idx: usize) -> usize {
        let next_line = (line_idx + 1).min(self.rope.len_lines());
        self.rope.line_to_char(next_line)
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
