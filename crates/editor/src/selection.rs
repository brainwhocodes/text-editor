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
