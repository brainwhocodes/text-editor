use tree_sitter::{InputEdit, Parser, Point, Tree};

pub struct IncrementalParser {
    parser: Parser,
    tree: Option<Tree>,
}

impl IncrementalParser {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            tree: None,
        }
    }

    pub fn set_language(&mut self, language: tree_sitter::Language) -> Result<(), String> {
        self.parser
            .set_language(&language)
            .map_err(|e| format!("Failed to set language: {}", e))
    }

    pub fn parse(&mut self, text: &str) -> Option<&Tree> {
        let tree = self.parser.parse(text, self.tree.as_ref())?;
        self.tree = Some(tree);
        self.tree.as_ref()
    }

    pub fn edit(&mut self, edit: &InputEdit) {
        if let Some(tree) = &mut self.tree {
            tree.edit(edit);
        }
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }
}

impl Default for IncrementalParser {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_input_edit(
    start_byte: usize,
    old_end_byte: usize,
    new_end_byte: usize,
    start_position: Point,
    old_end_position: Point,
    new_end_position: Point,
) -> InputEdit {
    InputEdit {
        start_byte,
        old_end_byte,
        new_end_byte,
        start_position,
        old_end_position,
        new_end_position,
    }
}
