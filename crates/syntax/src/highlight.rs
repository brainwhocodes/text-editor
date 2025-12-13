use crate::language::{LanguageConfig, TokenType};
use crate::parser::IncrementalParser;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub token_type: TokenType,
}

#[derive(Debug, Clone)]
pub struct LineHighlights {
    pub line_idx: usize,
    pub spans: Vec<HighlightSpan>,
}

pub struct SyntaxHighlighter {
    parser: IncrementalParser,
    highlighter: Highlighter,
    current_config: Option<HighlightConfiguration>,
    highlight_names: Vec<String>,
}

impl Clone for SyntaxHighlighter {
    fn clone(&self) -> Self {
        Self {
            parser: IncrementalParser::new(),
            highlighter: Highlighter::new(),
            current_config: None,
            highlight_names: self.highlight_names.clone(),
        }
    }
}

impl std::fmt::Debug for SyntaxHighlighter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyntaxHighlighter")
            .field("has_config", &self.current_config.is_some())
            .finish()
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let highlight_names = vec![
            "keyword".to_string(),
            "function".to_string(),
            "type".to_string(),
            "string".to_string(),
            "comment".to_string(),
            "number".to_string(),
            "operator".to_string(),
            "variable".to_string(),
            "punctuation".to_string(),
            "property".to_string(),
            "constant".to_string(),
        ];
        Self {
            parser: IncrementalParser::new(),
            highlighter: Highlighter::new(),
            current_config: None,
            highlight_names,
        }
    }

    pub fn set_language(&mut self, config: &LanguageConfig) -> Result<(), String> {
        self.parser.set_language(config.language.clone())?;
        let mut highlight_config = HighlightConfiguration::new(
            config.language.clone(),
            config.name,
            config.highlight_query,
            "",
            "",
        )
        .map_err(|e| format!("Failed to create highlight config: {}", e))?;
        highlight_config.configure(&self.highlight_names);
        self.current_config = Some(highlight_config);
        Ok(())
    }

    pub fn parse(&mut self, text: &str) -> Option<()> {
        self.parser.parse(text)?;
        Some(())
    }

    pub fn highlight_text(&mut self, text: &str) -> Result<Vec<HighlightSpan>, String> {
        let config = self
            .current_config
            .as_ref()
            .ok_or("No language configured")?;
        let highlights = self
            .highlighter
            .highlight(config, text.as_bytes(), None, |_| None)
            .map_err(|e| format!("Highlight error: {}", e))?;
        let mut spans = Vec::new();
        let mut current_pos = 0usize;
        let highlight_names = &self.highlight_names;
        for event in highlights {
            match event.map_err(|e| format!("Event error: {}", e))? {
                HighlightEvent::Source { start: _, end } => {
                    current_pos = end;
                }
                HighlightEvent::HighlightStart(idx) => {
                    if let Some(token_type) = Self::map_index_to_token_type(highlight_names, idx.0) {
                        let start = current_pos;
                        spans.push(HighlightSpan {
                            start_byte: start,
                            end_byte: start,
                            token_type,
                        });
                    }
                }
                HighlightEvent::HighlightEnd => {
                    if let Some(last) = spans.last_mut() {
                        last.end_byte = current_pos;
                    }
                }
            }
        }
        Ok(spans)
    }

    pub fn highlight_lines(
        &mut self,
        text: &str,
        line_range: std::ops::Range<usize>,
    ) -> Result<Vec<LineHighlights>, String> {
        let all_spans = self.highlight_text(text)?;
        let lines: Vec<&str> = text.lines().collect();
        let mut result = Vec::new();
        let mut byte_offset = 0usize;
        for (line_idx, line_text) in lines.iter().enumerate() {
            if line_idx >= line_range.start && line_idx < line_range.end {
                let line_start = byte_offset;
                let line_end = byte_offset + line_text.len();
                let line_spans: Vec<HighlightSpan> = all_spans
                    .iter()
                    .filter(|span| span.start_byte < line_end && span.end_byte > line_start)
                    .map(|span| HighlightSpan {
                        start_byte: span.start_byte.saturating_sub(line_start),
                        end_byte: (span.end_byte.saturating_sub(line_start)).min(line_text.len()),
                        token_type: span.token_type,
                    })
                    .collect();
                result.push(LineHighlights {
                    line_idx,
                    spans: line_spans,
                });
            }
            byte_offset += line_text.len() + 1;
        }
        Ok(result)
    }

    fn map_index_to_token_type(highlight_names: &[String], idx: usize) -> Option<TokenType> {
        let name = highlight_names.get(idx)?;
        match name.as_str() {
            "keyword" => Some(TokenType::Keyword),
            "function" => Some(TokenType::Function),
            "type" => Some(TokenType::Type),
            "string" => Some(TokenType::String),
            "comment" => Some(TokenType::Comment),
            "number" => Some(TokenType::Number),
            "operator" => Some(TokenType::Operator),
            "variable" => Some(TokenType::Variable),
            "punctuation" => Some(TokenType::Punctuation),
            "property" => Some(TokenType::Property),
            "constant" => Some(TokenType::Constant),
            _ => Some(TokenType::None),
        }
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}
