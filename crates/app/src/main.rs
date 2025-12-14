use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

mod events;
use events::{create_event_bridge, invoke_ui_update, UiEvent};

slint::include_modules!();

/// State for managing open editor tabs and file content.
#[derive(Debug, Default)]
struct EditorState {
    /// Open tabs: path -> content
    tabs: Vec<OpenTab>,
    /// Currently active tab index
    active_index: Option<usize>,
}

#[derive(Debug, Clone)]
struct OpenTab {
    path: PathBuf,
    filename: String,
    content: String,
    dirty: bool,
    language: String,
}

impl EditorState {
    fn new() -> Self {
        Self::default()
    }

    fn open_file(&mut self, path: PathBuf) -> Result<usize, String> {
        // Check if already open
        if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
            self.active_index = Some(idx);
            return Ok(idx);
        }
        // Read file content
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let filename = path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled".to_string());
        let language = detect_language(&path);
        let tab = OpenTab {
            path,
            filename,
            content,
            dirty: false,
            language,
        };
        self.tabs.push(tab);
        let idx = self.tabs.len() - 1;
        self.active_index = Some(idx);
        Ok(idx)
    }

    fn close_tab(&mut self, path: &Path) -> bool {
        if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
            self.tabs.remove(idx);
            if self.tabs.is_empty() {
                self.active_index = None;
            } else if let Some(active) = self.active_index {
                if active >= self.tabs.len() {
                    self.active_index = Some(self.tabs.len() - 1);
                } else if active > idx {
                    self.active_index = Some(active - 1);
                }
            }
            return true;
        }
        false
    }

    fn active_tab(&self) -> Option<&OpenTab> {
        self.active_index.and_then(|i| self.tabs.get(i))
    }

    fn set_active_by_path(&mut self, path: &Path) {
        if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
            self.active_index = Some(idx);
        }
    }
}

fn detect_language(path: &Path) -> String {
    match path.extension().and_then(|s| s.to_str()) {
        Some("rs") => "Rust".to_string(),
        Some("py") => "Python".to_string(),
        Some("js") => "JavaScript".to_string(),
        Some("ts") => "TypeScript".to_string(),
        Some("tsx") => "TypeScript React".to_string(),
        Some("jsx") => "JavaScript React".to_string(),
        Some("html" | "htm") => "HTML".to_string(),
        Some("css") => "CSS".to_string(),
        Some("scss" | "sass") => "SCSS".to_string(),
        Some("json") => "JSON".to_string(),
        Some("toml") => "TOML".to_string(),
        Some("yaml" | "yml") => "YAML".to_string(),
        Some("md") => "Markdown".to_string(),
        Some("go") => "Go".to_string(),
        Some("c") => "C".to_string(),
        Some("cpp" | "cc" | "cxx") => "C++".to_string(),
        Some("h" | "hpp") => "C/C++ Header".to_string(),
        Some("java") => "Java".to_string(),
        Some("kt") => "Kotlin".to_string(),
        Some("rb") => "Ruby".to_string(),
        Some("sh" | "bash") => "Shell".to_string(),
        Some("sql") => "SQL".to_string(),
        Some("xml") => "XML".to_string(),
        Some("slint") => "Slint".to_string(),
        _ => "Plain Text".to_string(),
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let client = ai::OpenRouterClient::new().unwrap();
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "user".to_string());
    let key_store = ai::KeyStore::new("ai-code-editor", username);
    let ai_service = ai::AiService::new(client, key_store.clone());

    let window = AppWindow::new()?;
    let weak = window.as_weak();
    let ai_service = std::sync::Arc::new(ai_service);
    let handle = rt.handle().clone();

    // Initialize workspace with current directory or passed argument
    let workspace_root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let workspace = Arc::new(Mutex::new(
        workspace::WorkspaceService::open(workspace_root).unwrap_or_else(|e| {
            eprintln!("Failed to open workspace: {e}");
            workspace::WorkspaceService::open(std::env::current_dir().unwrap()).unwrap()
        }),
    ));

    // Editor state for managing open tabs
    let editor_state = Arc::new(Mutex::new(EditorState::new()));

    // Build initial file tree and update UI
    {
        let mut ws = workspace.lock().unwrap();
        ws.build_tree();
        window.set_workspace_name(ws.name().into());
        update_file_tree(&window, &ws);
    }

    // Handle file selection - open file in editor
    {
        let editor_clone = Arc::clone(&editor_state);
        let weak_file = weak.clone();
        window.on_file_selected(move |path| {
            let path_str: String = path.into();
            let path = PathBuf::from(&path_str);
            let mut editor = editor_clone.lock().unwrap();
            if let Err(e) = editor.open_file(path) {
                eprintln!("Failed to open file: {e}");
                return;
            }
            if let Some(w) = weak_file.upgrade() {
                update_editor_ui(&w, &editor);
            }
        });
    }

    // Handle tab selection
    {
        let editor_clone = Arc::clone(&editor_state);
        let weak_tab = weak.clone();
        window.on_tab_selected(move |path| {
            let path_str: String = path.into();
            let mut editor = editor_clone.lock().unwrap();
            editor.set_active_by_path(Path::new(&path_str));
            if let Some(w) = weak_tab.upgrade() {
                update_editor_ui(&w, &editor);
            }
        });
    }

    // Handle tab close
    {
        let editor_clone = Arc::clone(&editor_state);
        let weak_close = weak.clone();
        window.on_tab_closed(move |path| {
            let path_str: String = path.into();
            let mut editor = editor_clone.lock().unwrap();
            editor.close_tab(Path::new(&path_str));
            if let Some(w) = weak_close.upgrade() {
                update_editor_ui(&w, &editor);
            }
        });
    }

    // Handle folder toggle
    {
        let workspace_clone = Arc::clone(&workspace);
        let weak_folder = weak.clone();
        window.on_folder_toggled(move |path| {
            let path_str: String = path.into();
            let mut ws = workspace_clone.lock().unwrap();
            ws.toggle_expand(std::path::Path::new(&path_str));
            if let Some(w) = weak_folder.upgrade() {
                update_file_tree(&w, &ws);
            }
        });
    }

    // Create the event bridge for UI-thread communication
    let (event_sender, mut event_receiver) = create_event_bridge(256, None);

    // Spawn the event processor task
    {
        let weak_events = weak.clone();
        handle.spawn(async move {
            while let Some(event) = event_receiver.recv().await {
                let weak = weak_events.clone();
                invoke_ui_update(move || {
                    if let Some(w) = weak.upgrade() {
                        handle_ui_event(&w, event);
                    }
                });
            }
        });
    }

    let initial_model = load_config().model;
    window.set_model_id(initial_model.clone().into());
    window.set_model_status(format!("Model: {initial_model}").into());

    {
        let weak_model = weak.clone();
        let handle_model = handle.clone();
        window.on_save_model(move |model: slint::SharedString| {
            let model: String = model.into();
            let weak_model = weak_model.clone();

            handle_model.spawn(async move {
                let status = tokio::task::spawn_blocking(move || {
                    let cfg = AppConfig { model: model.clone() };
                    match save_config(&cfg) {
                        Ok(()) => Ok(model),
                        Err(e) => Err(e),
                    }
                })
                .await
                .ok();

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = weak_model.upgrade() {
                        match status {
                            Some(Ok(model)) => {
                                w.set_model_id(model.clone().into());
                                w.set_model_status(format!("Model: {model}").into());
                            }
                            Some(Err(e)) => {
                                w.set_model_status(format!("Model: error ({e})").into());
                            }
                            None => {
                                w.set_model_status("Model: error".into());
                            }
                        }
                    }
                });
            });
        });
    }

    window.set_key_status("API key: checking...".into());
    {
        let weak_init = weak.clone();
        let key_store_init = key_store.clone();
        handle.spawn(async move {
            let result = tokio::task::spawn_blocking(move || key_store_init.get_openrouter_key())
                .await
                .ok();

            let status = match result {
                Some(Ok(Some(_))) => "API key: set".to_string(),
                Some(Ok(None)) => "API key: not set".to_string(),
                Some(Err(e)) => format!("API key: error ({e})"),
                None => "API key: error".to_string(),
            };

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(w) = weak_init.upgrade() {
                    w.set_key_status(status.into());
                }
            });
        });
    }

    {
        let weak_save = weak.clone();
        let key_store_save = key_store.clone();
        let handle_save = handle.clone();
        window.on_save_api_key(move |key: slint::SharedString| {
            let key: String = key.into();
            let weak_save = weak_save.clone();
            let key_store_save = key_store_save.clone();

            handle_save.spawn(async move {
                let result = tokio::task::spawn_blocking(move || key_store_save.set_openrouter_key(&key))
                    .await
                    .ok();

                let (status, clear_input) = match result {
                    Some(Ok(())) => ("API key: set".to_string(), true),
                    Some(Err(e)) => (format!("API key: error ({e})"), false),
                    None => ("API key: error".to_string(), false),
                };

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = weak_save.upgrade() {
                        w.set_key_status(status.into());
                        if clear_input {
                            w.set_api_key_input("".into());
                        }
                    }
                });
            });
        });
    }

    {
        let weak_remove = weak.clone();
        let key_store_remove = key_store.clone();
        let handle_remove = handle.clone();
        window.on_remove_api_key(move || {
            let weak_remove = weak_remove.clone();
            let key_store_remove = key_store_remove.clone();

            handle_remove.spawn(async move {
                let result = tokio::task::spawn_blocking(move || key_store_remove.remove_openrouter_key())
                    .await
                    .ok();

                let status = match result {
                    Some(Ok(())) => "API key: not set".to_string(),
                    Some(Err(e)) => format!("API key: error ({e})"),
                    None => "API key: error".to_string(),
                };

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = weak_remove.upgrade() {
                        w.set_key_status(status.into());
                        w.set_api_key_input("".into());
                    }
                });
            });
        });
    }

    // Chat handler using the event bridge for UI updates
    {
        let event_tx = event_sender.clone();
        window.on_send_chat(move |message: slint::SharedString| {
            let message: String = message.into();
            let ai_service = ai_service.clone();
            let weak = weak.clone();
            let event_tx = event_tx.clone();

            let mut model = default_model();

            if let Some(w) = weak.upgrade() {
                let current = w.get_chat_output().to_string();
                w.set_chat_output(format!("{current}You: {message}\n\nAssistant: ").into());
                w.set_chat_input("".into());
                model = w.get_model_id().to_string();
            }

            handle.spawn(async move {
                let request = ai::ChatCompletionsRequest {
                    model,
                    messages: vec![ai::ChatMessage {
                        role: "user".to_string(),
                        content: message,
                    }],
                    temperature: None,
                    max_tokens: None,
                    stream: Some(true),
                };

                let mut rx = match ai_service.send_chat_stream(request, 128).await {
                    Ok(rx) => rx,
                    Err(e) => {
                        let _ = event_tx.send(UiEvent::ChatError {
                            message: e.to_string(),
                        }).await;
                        return;
                    }
                };

                while let Some(item) = rx.recv().await {
                    match item {
                        Ok(delta) => {
                            let _ = event_tx.send(UiEvent::ChatResponseChunk {
                                content: delta,
                            }).await;
                        }
                        Err(e) => {
                            let _ = event_tx.send(UiEvent::ChatError {
                                message: e.to_string(),
                            }).await;
                            return;
                        }
                    }
                }

                let _ = event_tx.send(UiEvent::ChatResponseComplete).await;
            });
        });
    }

    window.run()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    model: String,
}

fn default_model() -> String {
    "openai/gpt-4o-mini".to_string()
}

fn config_path() -> Option<PathBuf> {
    let dirs = ProjectDirs::from("dev", "text_editor", "ai_code_editor")?;
    Some(dirs.config_dir().join("config.json"))
}

fn load_config() -> AppConfig {
    let Some(path) = config_path() else {
        return AppConfig {
            model: default_model(),
        };
    };

    let data = std::fs::read_to_string(path);
    match data {
        Ok(s) => serde_json::from_str::<AppConfig>(&s)
            .ok()
            .unwrap_or(AppConfig {
                model: default_model(),
            }),
        Err(_) => AppConfig {
            model: default_model(),
        },
    }
}

fn save_config(cfg: &AppConfig) -> Result<(), String> {
    let path = config_path().ok_or("no config directory")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

/// Handle UI events from the event bridge.
/// This function is called on the UI thread via invoke_from_event_loop.
fn handle_ui_event(window: &AppWindow, event: UiEvent) {
    match event {
        UiEvent::EditorContentChanged { start_line, end_line } => {
            // Editor content updates are handled via update_editor_ui
            let _ = (start_line, end_line);
        }
        UiEvent::CursorMoved { line, column } => {
            window.set_cursor_position(format!("Ln {line}, Col {column}").into());
        }
        UiEvent::FileLoaded { filename, language } => {
            window.set_status_message(format!("Loaded: {filename}").into());
            window.set_language(language.into());
        }
        UiEvent::FileSaveStatus { filename, is_dirty } => {
            let status = if is_dirty { "Modified" } else { "Saved" };
            window.set_status_message(format!("{filename}: {status}").into());
        }
        UiEvent::ExplorerRefresh => {
            // Explorer refresh is handled directly by workspace watcher
        }
        UiEvent::ChatResponseChunk { content } => {
            let current = window.get_chat_output().to_string();
            window.set_chat_output(format!("{current}{content}").into());
        }
        UiEvent::ChatResponseComplete => {
            let current = window.get_chat_output().to_string();
            window.set_chat_output(format!("{current}\n\n").into());
        }
        UiEvent::ChatError { message } => {
            let current = window.get_chat_output().to_string();
            window.set_chat_output(format!("{current}\nError: {message}\n\n").into());
        }
        UiEvent::DiffAvailable { hunk_count } => {
            window.set_status_message(format!("{hunk_count} diff hunks available").into());
        }
        UiEvent::DiffHunkResolved { remaining } => {
            window.set_status_message(format!("{remaining} diff hunks remaining").into());
        }
        UiEvent::StatusUpdate { message } => {
            window.set_status_message(message.into());
        }
    }
}

/// Update editor UI with current tabs and content.
fn update_editor_ui(window: &AppWindow, editor: &EditorState) {
    // Update tabs model
    let tabs: Vec<TabData> = editor.tabs.iter().map(|tab| {
        TabData {
            filename: tab.filename.clone().into(),
            path: tab.path.to_string_lossy().to_string().into(),
            dirty: tab.dirty,
        }
    }).collect();
    let tabs_model = std::rc::Rc::new(slint::VecModel::from(tabs));
    window.set_tabs(tabs_model.into());

    // Update active tab index
    window.set_active_tab(editor.active_index.map(|i| i as i32).unwrap_or(-1));

    // Update editor lines for active tab
    if let Some(tab) = editor.active_tab() {
        let lines: Vec<EditorLineData> = tab.content
            .lines()
            .enumerate()
            .map(|(i, line)| EditorLineData {
                line_num: (i + 1) as i32,
                content: line.to_string().into(),
                is_current: false,
            })
            .collect();
        let lines_model = std::rc::Rc::new(slint::VecModel::from(lines));
        window.set_editor_lines(lines_model.into());
        window.set_language(tab.language.clone().into());
        window.set_cursor_position("Ln 1, Col 1".into());
    } else {
        // No active tab - clear editor
        let empty: Vec<EditorLineData> = Vec::new();
        let empty_model = std::rc::Rc::new(slint::VecModel::from(empty));
        window.set_editor_lines(empty_model.into());
        window.set_language("Plain Text".into());
        window.set_cursor_position("".into());
    }
}

/// Convert workspace file tree to Slint model and update UI.
fn update_file_tree(window: &AppWindow, ws: &workspace::WorkspaceService) {
    let flat_items = ws.flat_tree();
    let model: Vec<FileEntry> = flat_items
        .into_iter()
        .map(|item| {
            let icon = get_file_icon(&item.node);
            FileEntry {
                name: item.node.name.clone().into(),
                path: item.node.path.to_string_lossy().to_string().into(),
                icon: icon.into(),
                indent: item.depth as i32,
                is_folder: item.node.is_directory(),
                expanded: item.node.expanded,
                visible: item.visible,
            }
        })
        .collect();
    let model_rc = std::rc::Rc::new(slint::VecModel::from(model));
    window.set_files(model_rc.into());
}

/// Get appropriate icon for a file based on extension.
fn get_file_icon(node: &workspace::TreeNode) -> &'static str {
    if node.is_directory() {
        return "📁";
    }
    match node.extension() {
        Some("rs") => "🦀",
        Some("toml") => "📦",
        Some("md") => "📝",
        Some("json") => "📋",
        Some("js" | "ts" | "jsx" | "tsx") => "📜",
        Some("html" | "htm") => "🌐",
        Some("css" | "scss" | "sass") => "🎨",
        Some("py") => "🐍",
        Some("go") => "🐹",
        Some("c" | "cpp" | "h" | "hpp") => "⚙️",
        Some("java" | "kt") => "☕",
        Some("rb") => "💎",
        Some("sh" | "bash" | "zsh") => "🖥️",
        Some("yml" | "yaml") => "⚙️",
        Some("xml") => "📰",
        Some("sql") => "🗄️",
        Some("png" | "jpg" | "jpeg" | "gif" | "svg" | "ico") => "🖼️",
        Some("zip" | "tar" | "gz" | "7z" | "rar") => "📦",
        Some("pdf") => "📕",
        Some("txt") => "📄",
        Some("lock") => "🔒",
        Some("gitignore") => "🙈",
        _ => "📄",
    }
}
