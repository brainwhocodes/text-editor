use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};

#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    pub glyph_id: u16,
    pub x_offset: f32,
    pub width: f32,
    pub char_start: usize,
    pub char_end: usize,
}

#[derive(Debug, Clone)]
pub struct ShapedLine {
    pub glyphs: Vec<ShapedGlyph>,
    pub width_px: f32,
    pub char_to_x: Vec<f32>,
}

impl ShapedLine {
    pub fn x_for_char(&self, char_idx: usize) -> f32 {
        self.char_to_x.get(char_idx).copied().unwrap_or(self.width_px)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    pub line_height: f32,
    pub avg_char_width: f32,
}

pub struct TextShaper {
    font_system: FontSystem,
    // future use
    #[allow(dead_code)]
    swash_cache: SwashCache,
    font_size: f32,
}

impl Clone for TextShaper {
    fn clone(&self) -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            font_size: self.font_size,
        }
    }
}

impl std::fmt::Debug for TextShaper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextShaper")
            .field("font_size", &self.font_size)
            .finish()
    }
}

impl TextShaper {
    pub fn new(font_size: f32) -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            font_size,
        }
    }

    pub fn metrics(&self) -> FontMetrics {
        let metrics = Metrics::new(self.font_size, self.font_size * 1.2);
        FontMetrics {
            line_height: metrics.line_height,
            avg_char_width: self.font_size * 0.6,
        }
    }

    pub fn shape_line(&mut self, text: &str) -> ShapedLine {
        let metrics = Metrics::new(self.font_size, self.font_size * 1.2);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, None, None);
        buffer.set_text(&mut self.font_system, text, Attrs::new(), Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut glyphs = Vec::new();
        let mut char_to_x = Vec::new();
        let mut current_x = 0.0f32;
        let char_count = text.chars().count();

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let char_start = glyph.start;
                let char_end = glyph.end;
                let x_offset = current_x + glyph.x_offset;
                let width = glyph.w;
                while char_to_x.len() < char_start {
                    char_to_x.push(current_x);
                }
                for _ in char_start..char_end {
                    char_to_x.push(x_offset);
                }
                glyphs.push(ShapedGlyph {
                    glyph_id: glyph.glyph_id,
                    x_offset,
                    width,
                    char_start,
                    char_end,
                });
                current_x = x_offset + width;
            }
        }

        while char_to_x.len() <= char_count {
            char_to_x.push(current_x);
        }

        ShapedLine {
            glyphs,
            width_px: current_x,
            char_to_x,
        }
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }
}
