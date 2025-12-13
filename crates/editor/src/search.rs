#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    pub needle: String,
    pub case_sensitive: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub start_char: usize,
    pub end_char: usize,
}

pub fn byte_to_char_idx(s: &str, byte_idx: usize) -> usize {
    s[..byte_idx.min(s.len())].chars().count()
}

pub fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
    let mut cur = 0usize;
    for (b, _) in s.char_indices() {
        if cur == char_idx {
            return b;
        }
        cur += 1;
    }
    s.len()
}
