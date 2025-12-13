use std::collections::HashMap;
use crate::buffer::{Buffer, ReplaceRange};
use crate::history::TransactionKind;
use crate::keymap::{KeyAction, Keymap, Movement};
use crate::layout::{
    EditorViewModel, FontMetrics, LayoutConfig, SelectionSpan, VisualLine, Viewport, split_by_cols,
};
use crate::search::{SearchDirection, SearchMatch, SearchQuery, byte_to_char_idx, char_to_byte_idx};
use crate::selection::{Selection, SelectionSet};
use crate::text_shaping::{ShapedLine, TextShaper};
use syntax::{LanguageRegistry, SyntaxHighlighter};

#[derive(Debug, Clone)]
struct CachedLine {
    text: String,
    shaped: Option<ShapedLine>,
}

#[derive(Debug, Clone)]
pub struct EditorEngine {
    pub buffer: Buffer,
    pub metrics: FontMetrics,
    pub layout: LayoutConfig,
    pub viewport: Viewport,
    pub keymap: Keymap,
    line_cache: HashMap<usize, CachedLine>,
    cached_doc_version: u64,
    cached_line_count: usize,
    shaper: TextShaper,
    highlighter: Option<SyntaxHighlighter>,
    language_registry: LanguageRegistry,
    current_filename: Option<String>,
}

impl EditorEngine {
    pub fn new(text: &str) -> Self {
        let shaper = TextShaper::new(14.0);
        let metrics_from_shaper = shaper.metrics();
        let metrics = FontMetrics {
            char_width_px: metrics_from_shaper.avg_char_width,
            line_height_px: metrics_from_shaper.line_height,
        };
        Self {
            buffer: Buffer::new(text),
            metrics,
            layout: LayoutConfig::default(),
            viewport: Viewport { first_line: 0, max_lines: 64, width_cols: 120 },
            keymap: Keymap::with_defaults(),
            line_cache: HashMap::new(),
            cached_doc_version: 0,
            cached_line_count: 0,
            shaper,
            highlighter: None,
            language_registry: LanguageRegistry::new(),
            current_filename: None,
        }
    }

    pub fn set_filename(&mut self, filename: &str) {
        self.current_filename = Some(filename.to_string());
        if let Some(lang_config) = self.language_registry.detect_language(filename) {
            let mut highlighter = SyntaxHighlighter::new();
            if highlighter.set_language(lang_config).is_ok() {
                let _ = highlighter.parse(&self.buffer.doc.to_string());
                self.highlighter = Some(highlighter);
            }
        } else {
            self.highlighter = None;
        }
    }

    pub fn apply_key_action(&mut self, action: KeyAction, clipboard_text: &mut String) {
        match action {
            KeyAction::Newline => self.buffer.apply_text_to_selections("\n"),
            KeyAction::Backspace => self.backspace(),
            KeyAction::Delete => self.delete_forward(),
            KeyAction::DeleteWordBackward => self.delete_word_backward(),
            KeyAction::DeleteWordForward => self.delete_word_forward(),
            KeyAction::DeleteLine => self.delete_line(),
            KeyAction::Undo => { self.buffer.undo(); }
            KeyAction::Redo => { self.buffer.redo(); }
            KeyAction::Copy => { *clipboard_text = self.copy(); }
            KeyAction::Cut => { *clipboard_text = self.cut(); }
            KeyAction::Paste => {
                let t = clipboard_text.clone();
                self.buffer.apply_text_to_selections(&t);
            }
            KeyAction::Indent => self.indent(),
            KeyAction::Outdent => self.outdent(),
            KeyAction::DuplicateLine => self.duplicate_line(),
            KeyAction::ToggleComment => self.toggle_comment(),
            KeyAction::Move { movement, extend } => self.move_cursors(movement, extend),
        }
    }

    pub fn insert_text(&mut self, text: &str) {
        self.buffer.apply_text_to_selections(text);
    }

    pub fn view_model(&mut self) -> EditorViewModel {
        let doc_version = self.buffer.doc.version();
        let line_count = self.buffer.doc.len_lines();
        if doc_version != self.cached_doc_version {
            if line_count != self.cached_line_count {
                self.line_cache.clear();
            } else if let Some(impact) = self.buffer.last_edit_impact {
                let start = impact.start_line.min(line_count);
                let end = impact.end_line_inclusive.min(line_count.saturating_sub(1));
                for line in start..=end {
                    self.line_cache.remove(&line);
                }
            } else {
                self.line_cache.clear();
            }
            self.cached_doc_version = doc_version;
            self.cached_line_count = line_count;
        }
        let first = self.viewport.first_line.min(line_count);
        let last_exclusive = (first + self.viewport.max_lines).min(line_count);
        let gutter_width_cols = line_count.to_string().len().max(3) + 1;
        let selections = self.buffer.selections.all_including_primary();
        let active_line = self.buffer.doc.char_to_line(self.buffer.selections.primary.head);
        let mut lines = Vec::with_capacity(last_exclusive.saturating_sub(first));
        let mut y_px = 0.0f32;
        for line_idx in first..last_exclusive {
            let (text, shaped) = if let Some(cached) = self.line_cache.get(&line_idx) {
                (cached.text.clone(), cached.shaped.clone())
            } else {
                let t = self.buffer.doc.line_text(line_idx);
                let s = self.shaper.shape_line(&t);
                self.line_cache.insert(line_idx, CachedLine { text: t.clone(), shaped: Some(s.clone()) });
                (t, Some(s))
            };
            let segments = if self.layout.soft_wrap && self.viewport.width_cols > 0 {
                split_by_cols(&text, self.viewport.width_cols)
            } else {
                vec![text.clone()]
            };
            for (segment_idx, segment) in segments.iter().enumerate() {
                let wrap_col_offset = segment_idx * self.viewport.width_cols;
                let mut selection_spans = Vec::new();
                let mut cursors = Vec::new();
                for s in selections.iter() {
                    let (start, end) = s.range();
                    let line_start = self.buffer.doc.line_start_char(line_idx);
                    let line_end = self.buffer.doc.line_end_char(line_idx);
                    let sel_start = start.max(line_start).min(line_end);
                    let sel_end = end.max(line_start).min(line_end);
                    if sel_start < sel_end {
                        let start_col = sel_start.saturating_sub(line_start);
                        let end_col = sel_end.saturating_sub(line_start);
                        let seg_start = wrap_col_offset;
                        let seg_end = wrap_col_offset + segment.chars().count();
                        let inter_start = start_col.max(seg_start).min(seg_end);
                        let inter_end = end_col.max(seg_start).min(seg_end);
                        if inter_start < inter_end {
                            selection_spans.push(SelectionSpan {
                                start_col: inter_start - seg_start,
                                end_col: inter_end - seg_start,
                            });
                        }
                    }
                    if s.is_caret() {
                        let caret = s.head;
                        if caret >= line_start && caret <= line_end {
                            let col = caret.saturating_sub(line_start);
                            let seg_start = wrap_col_offset;
                            let seg_end = wrap_col_offset + segment.chars().count();
                            if col >= seg_start && col <= seg_end {
                                cursors.push(col - seg_start);
                            }
                        }
                    }
                }
                let highlights = if let Some(ref mut highlighter) = self.highlighter {
                    highlighter.highlight_lines(&self.buffer.doc.to_string(), line_idx..line_idx + 1)
                        .ok()
                        .and_then(|mut h| h.pop())
                        .map(|h| h.spans)
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                lines.push(VisualLine {
                    line_idx,
                    y_px,
                    wrap_col_offset,
                    text: segment.clone(),
                    selections: selection_spans,
                    cursors,
                    is_current_line: line_idx == active_line,
                    shaped: shaped.clone(),
                    highlights,
                });
                y_px += self.metrics.line_height_px;
            }
        }
        EditorViewModel { lines, gutter_width_cols }
    }

    pub fn find_next(
        &self,
        query: &SearchQuery,
        from_char: usize,
        direction: SearchDirection,
    ) -> Option<SearchMatch> {
        if query.needle.is_empty() {
            return None;
        }
        let text = self.buffer.doc.to_string();
        let (haystack, needle) = if query.case_sensitive {
            (text.clone(), query.needle.clone())
        } else {
            (text.to_lowercase(), query.needle.to_lowercase())
        };
        match direction {
            SearchDirection::Forward => {
                let start_byte = char_to_byte_idx(&haystack, from_char);
                let slice = &haystack[start_byte..];
                let found = slice.find(&needle)?;
                let global_byte = start_byte + found;
                let start_char_idx = byte_to_char_idx(&haystack, global_byte);
                let end_char_idx = start_char_idx + needle.chars().count();
                Some(SearchMatch { start_char: start_char_idx, end_char: end_char_idx })
            }
            SearchDirection::Backward => {
                let end_byte = char_to_byte_idx(&haystack, from_char.min(haystack.chars().count()));
                let slice = &haystack[..end_byte];
                let found = slice.rfind(&needle)?;
                let start_char_idx = byte_to_char_idx(&haystack, found);
                let end_char_idx = start_char_idx + needle.chars().count();
                Some(SearchMatch { start_char: start_char_idx, end_char: end_char_idx })
            }
        }
    }

    pub fn replace_range(&mut self, range: SearchMatch, replacement: &str) {
        let caret = range.start_char + replacement.chars().count();
        let new_selections = SelectionSet {
            primary: Selection { anchor: caret, head: caret },
            secondary: Vec::new(),
        };
        self.buffer.apply_replace_ranges(
            vec![ReplaceRange {
                start_char: range.start_char,
                end_char: range.end_char,
                inserted: replacement.to_string(),
            }],
            TransactionKind::Replace,
            new_selections,
        );
    }

    pub fn replace_all(&mut self, query: &SearchQuery, replacement: &str) -> usize {
        if query.needle.is_empty() {
            return 0;
        }
        let mut cursor = 0usize;
        let mut matches = Vec::new();
        loop {
            let Some(m) = self.find_next(query, cursor, SearchDirection::Forward) else { break };
            matches.push(m);
            cursor = m.end_char;
            if cursor >= self.buffer.doc.len_chars() {
                break;
            }
        }
        if matches.is_empty() {
            return 0;
        }
        let mut ranges = Vec::with_capacity(matches.len());
        for m in matches.iter() {
            ranges.push(ReplaceRange {
                start_char: m.start_char,
                end_char: m.end_char,
                inserted: replacement.to_string(),
            });
        }
        let caret = ranges.last().map(|r| r.start_char + replacement.chars().count()).unwrap_or(0);
        self.buffer.apply_replace_ranges(
            ranges,
            TransactionKind::Replace,
            SelectionSet {
                primary: Selection { anchor: caret, head: caret },
                secondary: Vec::new(),
            },
        );
        matches.len()
    }

    fn copy(&self) -> String {
        let selections = self.buffer.selections.all_including_primary();
        if selections.iter().all(|s| s.is_caret()) {
            return String::new();
        }
        let mut out = String::new();
        for (i, s) in selections.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            let (start, end) = s.range();
            out.push_str(&self.buffer.doc.slice_to_string(start, end));
        }
        out
    }

    fn cut(&mut self) -> String {
        let text = self.copy();
        if text.is_empty() {
            return text;
        }
        self.buffer.apply_text_to_selections("");
        text
    }

    fn backspace(&mut self) {
        let selections = self.buffer.selections.all_including_primary();
        if selections.iter().any(|s| !s.is_caret()) {
            self.buffer.apply_text_to_selections("");
            return;
        }
        let mut new_set = SelectionSet::default();
        let mut all = Vec::with_capacity(selections.len());
        for s in selections.iter() {
            let caret = s.head.min(self.buffer.doc.len_chars());
            if caret == 0 {
                all.push(Selection { anchor: caret, head: caret });
                continue;
            }
            all.push(Selection { anchor: caret - 1, head: caret });
        }
        if let Some(p) = all.first().copied() {
            new_set.primary = p;
            if all.len() > 1 {
                new_set.secondary = all[1..].to_vec();
            }
        }
        self.buffer.selections = new_set;
        self.buffer.apply_text_to_selections("");
    }

    fn delete_word_backward(&mut self) {
        let selections = self.buffer.selections.all_including_primary();
        if selections.iter().any(|s| !s.is_caret()) {
            self.buffer.apply_text_to_selections("");
            return;
        }
        let text = self.buffer.doc.to_string();
        let mut ranges = Vec::with_capacity(selections.len());
        for s in selections.iter() {
            let caret = s.head;
            let start = find_word_left(&text, caret);
            if start < caret {
                ranges.push(ReplaceRange { start_char: start, end_char: caret, inserted: String::new() });
            }
        }
        let caret = ranges.last().map(|r| r.start_char).unwrap_or(0);
        self.buffer.apply_replace_ranges(
            ranges,
            TransactionKind::Delete,
            SelectionSet { primary: Selection { anchor: caret, head: caret }, secondary: Vec::new() },
        );
    }

    fn delete_word_forward(&mut self) {
        let selections = self.buffer.selections.all_including_primary();
        if selections.iter().any(|s| !s.is_caret()) {
            self.buffer.apply_text_to_selections("");
            return;
        }
        let text = self.buffer.doc.to_string();
        let mut ranges = Vec::with_capacity(selections.len());
        for s in selections.iter() {
            let caret = s.head;
            let end = find_word_right(&text, caret);
            if caret < end {
                ranges.push(ReplaceRange { start_char: caret, end_char: end, inserted: String::new() });
            }
        }
        let caret = ranges.first().map(|r| r.start_char).unwrap_or(0);
        self.buffer.apply_replace_ranges(
            ranges,
            TransactionKind::Delete,
            SelectionSet { primary: Selection { anchor: caret, head: caret }, secondary: Vec::new() },
        );
    }

    fn delete_line(&mut self) {
        let selections = self.buffer.selections.all_including_primary();
        let mut line_idxs = Vec::with_capacity(selections.len());
        for s in selections.iter() {
            line_idxs.push(self.buffer.doc.char_to_line(s.head));
        }
        line_idxs.sort_unstable();
        line_idxs.dedup();
        let mut ranges = Vec::with_capacity(line_idxs.len());
        for line in line_idxs.into_iter().rev() {
            let start = self.buffer.doc.line_start_char(line);
            let end = self.buffer.doc.line_end_char(line);
            if start < end {
                ranges.push(ReplaceRange { start_char: start, end_char: end, inserted: String::new() });
            }
        }
        let caret = ranges.last().map(|r| r.start_char).unwrap_or(0);
        self.buffer.apply_replace_ranges(
            ranges,
            TransactionKind::Delete,
            SelectionSet { primary: Selection { anchor: caret, head: caret }, secondary: Vec::new() },
        );
    }

    fn delete_forward(&mut self) {
        let selections = self.buffer.selections.all_including_primary();
        if selections.iter().any(|s| !s.is_caret()) {
            self.buffer.apply_text_to_selections("");
            return;
        }
        let mut new_set = SelectionSet::default();
        let mut all = Vec::with_capacity(selections.len());
        for s in selections.iter() {
            let caret = s.head.min(self.buffer.doc.len_chars());
            if caret >= self.buffer.doc.len_chars() {
                all.push(Selection { anchor: caret, head: caret });
                continue;
            }
            all.push(Selection { anchor: caret, head: caret + 1 });
        }
        if let Some(p) = all.first().copied() {
            new_set.primary = p;
            if all.len() > 1 {
                new_set.secondary = all[1..].to_vec();
            }
        }
        self.buffer.selections = new_set;
        self.buffer.apply_text_to_selections("");
    }

    fn move_cursors(&mut self, movement: Movement, extend: bool) {
        let doc_len = self.buffer.doc.len_chars();
        let selections = self.buffer.selections.all_including_primary();
        let doc_text = self.buffer.doc.to_string();
        let mut moved = Vec::with_capacity(selections.len());
        for s in selections.iter() {
            let (start, end) = s.range();
            let base = if extend {
                s.head
            } else if matches!(movement, Movement::Left | Movement::Up | Movement::WordLeft | Movement::LineStart) {
                start
            } else {
                end
            };
            let new_head = match movement {
                Movement::Left => base.saturating_sub(1),
                Movement::Right => (base + 1).min(doc_len),
                Movement::LineStart => {
                    let line = self.buffer.doc.char_to_line(base);
                    self.buffer.doc.line_start_char(line)
                }
                Movement::LineEnd => {
                    let line = self.buffer.doc.char_to_line(base);
                    self.buffer.doc.line_end_char(line)
                }
                Movement::WordLeft => find_word_left(&doc_text, base),
                Movement::WordRight => find_word_right(&doc_text, base),
                Movement::Up => {
                    let lc = self.buffer.doc.char_to_line_col(base);
                    if lc.line == 0 { base } else { self.buffer.doc.line_col_to_char(lc.line - 1, lc.col) }
                }
                Movement::Down => {
                    let lc = self.buffer.doc.char_to_line_col(base);
                    if lc.line + 1 >= self.buffer.doc.len_lines() { base } else { self.buffer.doc.line_col_to_char(lc.line + 1, lc.col) }
                }
            };
            if extend {
                moved.push(Selection { anchor: s.anchor, head: new_head });
            } else {
                moved.push(Selection { anchor: new_head, head: new_head });
            }
        }
        let mut new_set = SelectionSet::default();
        if let Some(p) = moved.first().copied() {
            new_set.primary = p;
            if moved.len() > 1 {
                new_set.secondary = moved[1..].to_vec();
            }
        }
        self.buffer.selections = new_set;
    }

    fn indent(&mut self) {
        apply_line_prefix_edit(&mut self.buffer, "    ", false);
    }

    fn outdent(&mut self) {
        apply_line_prefix_edit(&mut self.buffer, "    ", true);
    }

    fn duplicate_line(&mut self) {
        let selections = self.buffer.selections.all_including_primary();
        let mut lines = Vec::new();
        for s in selections.iter() {
            lines.push(self.buffer.doc.char_to_line(s.head));
        }
        lines.sort_unstable();
        lines.dedup();
        let mut ranges = Vec::with_capacity(lines.len());
        for line in lines.into_iter().rev() {
            let start = self.buffer.doc.line_start_char(line);
            let end = self.buffer.doc.line_end_char(line);
            let original = self.buffer.doc.slice_to_string(start, end);
            let (line_text, line_break) = if original.ends_with('\n') {
                (original.trim_end_matches('\n').to_string(), "\n")
            } else {
                (original.clone(), "")
            };
            let inserted = format!("{line_text}\n{line_text}{line_break}");
            ranges.push(ReplaceRange { start_char: start, end_char: end, inserted });
        }
        let caret = self.buffer.selections.primary.head;
        self.buffer.apply_replace_ranges(
            ranges,
            TransactionKind::Other,
            SelectionSet { primary: Selection { anchor: caret, head: caret }, secondary: Vec::new() },
        );
    }

    fn toggle_comment(&mut self) {
        toggle_line_prefix(&mut self.buffer, "//");
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn find_word_left(text: &str, from_char: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut i = from_char.min(chars.len());
    if i == 0 {
        return 0;
    }
    i -= 1;
    while i > 0 && chars[i].is_whitespace() {
        i -= 1;
    }
    while i > 0 && is_word_char(chars[i]) && is_word_char(chars[i - 1]) {
        i -= 1;
    }
    i
}

fn find_word_right(text: &str, from_char: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut i = from_char.min(chars.len());
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }
    while i < chars.len() {
        let c = chars[i];
        if !is_word_char(c) {
            break;
        }
        i += 1;
        if i < chars.len() && !is_word_char(chars[i]) {
            break;
        }
    }
    i
}

fn apply_line_prefix_edit(buffer: &mut Buffer, prefix: &str, remove: bool) {
    let selections = buffer.selections.all_including_primary();
    let mut lines = Vec::new();
    for s in selections.iter() {
        let (start, end) = s.range();
        lines.push(buffer.doc.char_to_line(start));
        lines.push(buffer.doc.char_to_line(end));
    }
    lines.sort_unstable();
    lines.dedup();
    let mut ranges = Vec::new();
    for line in lines.into_iter().rev() {
        let start = buffer.doc.line_start_char(line);
        if remove {
            let current = buffer.doc.slice_to_string(start, (start + prefix.chars().count()).min(buffer.doc.len_chars()));
            if current == prefix {
                ranges.push(ReplaceRange { start_char: start, end_char: start + prefix.chars().count(), inserted: String::new() });
            }
        } else {
            ranges.push(ReplaceRange { start_char: start, end_char: start, inserted: prefix.to_string() });
        }
    }
    if ranges.is_empty() {
        return;
    }
    let caret = buffer.selections.primary.head;
    buffer.apply_replace_ranges(
        ranges,
        TransactionKind::Other,
        SelectionSet { primary: Selection { anchor: caret, head: caret }, secondary: Vec::new() },
    );
}

fn toggle_line_prefix(buffer: &mut Buffer, prefix: &str) {
    let selections = buffer.selections.all_including_primary();
    let mut lines = Vec::new();
    for s in selections.iter() {
        let (start, end) = s.range();
        lines.push(buffer.doc.char_to_line(start));
        lines.push(buffer.doc.char_to_line(end));
    }
    lines.sort_unstable();
    lines.dedup();
    if lines.is_empty() {
        return;
    }
    let mut all_have_prefix = true;
    for line in lines.iter() {
        let start = buffer.doc.line_start_char(*line);
        let check_end = (start + prefix.chars().count()).min(buffer.doc.len_chars());
        let current = buffer.doc.slice_to_string(start, check_end);
        if current != prefix {
            all_have_prefix = false;
            break;
        }
    }
    if all_have_prefix {
        apply_line_prefix_edit(buffer, prefix, true);
        return;
    }
    apply_line_prefix_edit(buffer, prefix, false);
}
