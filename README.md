# AI-Powered Code Editor

A cross-platform AI-powered code editor built with Rust, featuring a high-performance custom editor engine, Tree-sitter syntax highlighting, and OpenRouter LLM integration.

## Features

### ✅ Completed
- **High-End Editor Engine**
  - Rope-based text buffer (ropey) for efficient editing
  - Multi-cursor support with primary + secondary selections
  - Undo/redo with transaction coalescing
  - Incremental line layout cache with soft-wrap support
  - Text shaping with cosmic-text for accurate rendering
  - Viewport-based rendering for performance

- **Syntax Highlighting**
  - Tree-sitter integration for accurate parsing
  - Incremental re-parsing on edits
  - Language detection by file extension
  - Built-in support for Rust and JavaScript
  - Per-line token streams for efficient rendering

- **Editing Commands**
  - Insert/delete text with multi-cursor support
  - Word and line-based navigation
  - Copy/cut/paste
  - Indent/outdent
  - Duplicate line
  - Comment toggling

- **Search & Replace**
  - Incremental search (forward/backward)
  - Replace single occurrence
  - Replace all

### 🚧 In Progress
- Slint UI integration
- OpenRouter AI service
- Workspace management
- Diff view

## Architecture

```
crates/
├── app/          # Slint UI shell + wiring
├── core/         # Domain models, event bus, commands
├── editor/       # Buffer/layout/render/input engine
├── syntax/       # Tree-sitter integration
├── ai/           # OpenRouter client + streaming
├── workspace/    # Project tree, file watching
├── diff/         # Diff view model + patch operations
└── plugins/      # Plugin API + host
```

## How to Run

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Windows, macOS, or Linux

### Build and Run

```bash
# Clone the repository
git clone <repository-url>
cd text-editor

# Build the project
cargo build --release

# Run the application
cargo run --release
```

### Development

```bash
# Run in debug mode with faster compilation
cargo run

# Run tests
cargo test

# Check for errors without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Project Status

See [TODO.md](TODO.md) for detailed implementation progress and roadmap.

### Completed Milestones
- ✅ Section 1.1: Document model + buffer
- ✅ Section 1.2: Layout + rendering pipeline
- ✅ Section 1.3: Input & editing commands
- ✅ Section 1.4: Search & replace
- ✅ Section 2.1: Tree-sitter integration

## Technology Stack

- **Language**: Rust
- **UI Framework**: Slint
- **Text Buffer**: ropey
- **Text Shaping**: cosmic-text
- **Syntax Parsing**: tree-sitter
- **AI Integration**: OpenRouter API
- **Async Runtime**: tokio

## License

[Add your license here]
