#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FontMetrics {
    pub char_width_px: f32,
    pub line_height_px: f32,
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            char_width_px: 8.0,
            line_height_px: 16.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutConfig {
    pub soft_wrap: bool,
    pub whitespace: WhitespaceConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhitespaceConfig {
    pub show_spaces: bool,
    pub show_tabs: bool,
    pub show_newlines: bool,
}

impl Default for WhitespaceConfig {
    fn default() -> Self {
        Self {
            show_spaces: false,
            show_tabs: false,
            show_newlines: false,
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            soft_wrap: false,
            whitespace: WhitespaceConfig::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Viewport {
    pub first_line: usize,
    pub max_lines: usize,
    pub width_cols: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSpan {
    pub start_col: usize,
    pub end_col: usize,
}

#[derive(Debug, Clone)]
pub struct VisualLine {
    pub line_idx: usize,
    pub y_px: f32,
    pub wrap_col_offset: usize,
    pub text: String,
    pub selections: Vec<SelectionSpan>,
    pub cursors: Vec<usize>,
    pub is_current_line: bool,
    pub shaped: Option<crate::text_shaping::ShapedLine>,
    pub highlights: Vec<syntax::HighlightSpan>,
}

#[derive(Debug, Clone)]
pub struct EditorViewModel {
    pub lines: Vec<VisualLine>,
    pub gutter_width_cols: usize,
}

pub fn split_by_cols(text: &str, max_cols: usize) -> Vec<String> {
    if max_cols == 0 {
        return vec![text.to_string()];
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_cols {
        return vec![text.to_string()];
    }
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        let end = (i + max_cols).min(chars.len());
        out.push(chars[i..end].iter().collect());
        i = end;
    }
    out
}
